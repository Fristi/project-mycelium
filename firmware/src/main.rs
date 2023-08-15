
mod wifi;
mod kv;
mod onboarding;
mod auth0;
mod mycelium;

use std::sync::{Arc, PoisonError, RwLock, RwLockWriteGuard};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported


use std::time::{Duration};
use bluedroid::gatt_server::{Characteristic, GLOBAL_GATT_SERVER, Profile, Service};
use bluedroid::utilities::{AttributePermissions, BleUuid, CharacteristicProperties};
use embedded_svc::http::client::Client;
use embedded_svc::http::headers;
use esp_idf_hal::prelude::*;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::http::client::EspHttpConnection;
use esp_idf_svc::netif::{EspNetif, NetifStack};
use esp_idf_svc::nvs::{EspDefaultNvs, EspDefaultNvsPartition};
use esp_idf_svc::sntp::{EspSntp, SntpConf, SyncMode, SyncStatus};
use esp_idf_svc::systime::EspSystemTime;
use esp_idf_svc::wifi::EspWifi;
use esp_idf_sys::*;
use retry::delay::Fixed;
use retry::retry;
use serde_json::{from_slice, to_vec};
use thingbuf::mpsc::blocking::channel;


use crate::auth0::{AuthError, TokenResult};
use crate::wifi::{EspMyceliumWifi, MyceliumWifi, MyceliumWifiSettings};
use crate::kv::{KvStore, KvStoreError, NvsKvStore};
use crate::onboarding::{OnboardingError, OnboardingSettings, OnboardingState};
use crate::mycelium::{StationInsert, WateringSchedule};

const SERVICE_UUID: &str = "00467768-6228-2272-4663-277478269000";
const STATUS_UUID: &str = "00467768-6228-2272-4663-277478269001";
const RPC_COMMAND_UUID: &str = "00467768-6228-2272-4663-277478269002";

fn main() -> ! {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let nvs_partition = EspDefaultNvsPartition::take().unwrap();
    let nvs = EspDefaultNvs::new(nvs_partition, "mycelium", true).unwrap();
    let kv = NvsKvStore::new(nvs);

    let state = Arc::new(RwLock::new(OnboardingState::AwaitingSettings));
    let state_write = state.clone();
    let connection = EspHttpConnection::new(&esp_idf_svc::http::client::Configuration {
        use_global_ca_store: true,
        buffer_size_tx: Some(1536),
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        ..Default::default()
    }).unwrap();
    let client = &mut Client::wrap(connection);
    let sysloop = EspSystemEventLoop::take().unwrap();
    let peripherals = Peripherals::take().unwrap();
    let modem = peripherals.modem;
    let esp_wifi = EspWifi::new(modem, sysloop.clone(), None).unwrap();
    let wifi = EspMyceliumWifi::new(sysloop, esp_wifi);

    // unsafe {
    //     esp_sleep_enable_timer_wakeup(10000000000);
    //     esp_sleep_pd_config(esp_sleep_pd_domain_t_ESP_PD_DOMAIN_RTC_PERIPH, esp_sleep_pd_option_t_ESP_PD_OPTION_OFF);
    //     esp_sleep_pd_config(esp_sleep_pd_domain_t_ESP_PD_DOMAIN_XTAL, esp_sleep_pd_option_t_ESP_PD_OPTION_OFF);
    //     esp_deep_sleep_disable_rom_logging();
    //     esp_deep_sleep_start();
    // }

    let (tx, rx) = channel::<Vec<u8>>(4);

    let current_state = Characteristic::new(BleUuid::from_uuid128_string(STATUS_UUID))
        .name("Current state")
        .permissions(AttributePermissions::new().read())
        .properties(CharacteristicProperties::new().read().notify())
        .show_name()
        .on_read(move |_| {
            let s = state.read().unwrap().clone();
            return to_vec(&s).unwrap();
        })
        .build();

    let rpc_command = Characteristic::new(BleUuid::from_uuid128_string(RPC_COMMAND_UUID))
        .name("RPC command handler")
        .permissions(AttributePermissions::new().write().read())
        .properties(CharacteristicProperties::new().write().read())
        .on_write(move |bytes, _| { tx.send(bytes).unwrap() })
        .on_read(|_| vec![])
        .show_name()
        .build();

    let service = Service::new(BleUuid::from_uuid128_string(SERVICE_UUID))
        .name("Mycelium onboarding service")
        .primary()
        .characteristic(&rpc_command)
        .characteristic(&current_state)
        .build();

    let profile = Profile::new(0x0001)
        .name("Default Profile")
        .service(&service)
        .build();

    GLOBAL_GATT_SERVER
        .lock()
        .unwrap()
        .profile(profile)
        .device_name("Mycelium onboarding")
        .appearance(bluedroid::utilities::Appearance::GenericComputer)
        .advertise_service(&service)
        .start();

    loop {
        if let Some(bytes) = rx.recv() {


            let result = retry(Fixed::from_millis(10).take(5), || {
                process_message(client, &wifi, &kv, &state_write, &bytes)
            });

            match result {
                Ok(_) => (),
                Err(err) => {
                    let error = format!("{:?}", err);
                    *state_write.write().unwrap() = OnboardingState::Failed { error: heapless::String::from(error.as_str()) }
                }
            }
        }

        std::thread::sleep(Duration::from_secs(5));
    }
}

fn process_message(client: &mut Client<EspHttpConnection>, wifi: &EspMyceliumWifi, kv: &NvsKvStore, state_write: &Arc<RwLock<OnboardingState>>, bytes: &[u8]) -> Result<(), OnboardingError> {
    let settings = from_slice::<OnboardingSettings>(&bytes)?;

    *state_write.write()? = OnboardingState::ProvisioningWifi;

    let wifi_settings = kv.get("wifi_settings")?.filter(|x: &MyceliumWifiSettings| x.ssid == settings.wifi_ssid).unwrap_or(settings.clone().wifi_settings());
    let enriched_settings = wifi.connect(wifi_settings)?;

    kv.set("wifi_settings", enriched_settings)?;

    *state_write.write()? = OnboardingState::SynchronizingTime;

    let _sntp = EspSntp::new_default()?;

    while _sntp.get_sync_status() != SyncStatus::Completed {
        std::thread::sleep(Duration::from_secs(1));
    }

    let resp = auth0::request_device_code(client)?;

    println!("Got url: {:?}", resp.verification_uri_complete);

    *state_write.write()? = OnboardingState::AwaitingAuthorization { url: resp.verification_uri_complete };

    let mut authenticated = false;

    while authenticated == false {

        match auth0::poll_token(client, &resp.device_code) {
            Ok(TokenResult::Error { error }) => println!("Auth0 error {:?}", error),
            Ok(TokenResult::AccessToken { .. }) => println!("Skipping!"),
            Ok(TokenResult::Full { access_token, refresh_token, expires_in }) => {

                let netif = EspNetif::new(NetifStack::Eth)?;
                let mac = netif.get_mac()?;
                let mac_addr_str = heapless::String::from(format!("{:<02X}:{:<02X}:{:<02X}:{:<02X}:{:<02X}:{:<02X}", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]).as_str());

                let station_id = mycelium::insert_plant(
                    client,
                    &access_token,
                    &StationInsert {
                        mac: mac_addr_str,
                        name: settings.name.clone(),
                        location: settings.location.clone(),
                        description: settings.description.clone(),
                        watering_schedule: WateringSchedule::Threshold { below_soil_pf: 1337, period: heapless::String::from("5 seconds") }
                    }
                )?;

                // let now = EspSystemTime {}.now();
                // let expires_at = now.as_secs() + expires_in;

                kv.set("station_id", station_id)?;
                kv.set("access_token", access_token.clone())?;
                kv.set("refresh_token", refresh_token)?;
                // kv.set("expires_at", expires_at)?;

                *state_write.write()? = OnboardingState::Complete;
                authenticated = true;
            }
            Err(err) => println!("Auth0 error {:?}", err),
        }

        std::thread::sleep(Duration::from_secs(5))
    }

    println!("Done !!!");

    Ok(())
}