
mod wifi;
mod kv;
mod onboarding;
mod auth0;
mod mycelium;
mod settings;
mod tokens;

use std::sync::{Arc, RwLock};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported


use std::time::{Duration};
use bluedroid::gatt_server::{Characteristic, GLOBAL_GATT_SERVER, Profile, Service};
use bluedroid::utilities::{AttributePermissions, BleUuid, CharacteristicProperties};
use chrono::{NaiveDateTime, SecondsFormat, TimeZone, Utc};
use embedded_svc::http::client::Client;


use esp_idf_hal::prelude::*;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::http::client::EspHttpConnection;
use esp_idf_svc::netif::{EspNetif, NetifStack};
use esp_idf_svc::nvs::{EspDefaultNvs, EspDefaultNvsPartition};

use esp_idf_svc::systime::EspSystemTime;
use esp_idf_svc::wifi::EspWifi;
use esp_idf_sys::*;
use log::{error, info, warn};
use retry::delay::Fixed;
use retry::retry;
use serde_json::{from_slice, to_vec};
use thingbuf::mpsc::blocking::channel;

use crate::auth0::{TokenResult};
use crate::wifi::{EspMyceliumWifi, MyceliumWifi, MyceliumWifiSettings};
use crate::kv::{NvsKvStore};
use crate::onboarding::{AppError, OnboardingCommand, OnboardingState, OnboardingSettings};
use crate::mycelium::{StationInsert, StationMeasurement, WateringSchedule};
use crate::settings::FlashState;
use crate::tokens::{TokenWallet, TokenWalletError};

fn main() -> ! {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let nvs_partition = EspDefaultNvsPartition::take().unwrap();
    let nvs = EspDefaultNvs::new(nvs_partition, "mycelium", true).unwrap();
    let kv = NvsKvStore::new(nvs);
    let flash_state = FlashState::new(kv);

    if flash_state.has_station_id().unwrap() {
        operational(&flash_state)
    } else {
        onboarding(&flash_state)
    }
}

fn extract_wallet(client: &mut Client<EspHttpConnection>, flash_state: &FlashState) -> Result<TokenWallet, AppError>  {
    let wallet = flash_state.get_token_wallet()?;
    let needs_refresh = wallet.needs_refresh()?;

    if needs_refresh {
        let resp = auth0::refresh_token(client, &wallet.refresh_token)?;

        match resp {
            TokenResult::Error { error } => error!("Token error: {:?}", error),
            TokenResult::AccessToken { access_token, expires_in } => {
                let new_wallet = wallet.update(access_token, expires_in)?;
                flash_state.set_token_wallet(new_wallet.clone())?;
                return Ok(new_wallet.clone())
            }
            _ => info!("Ignoring")
        }
    }

    Ok(wallet)
}

fn measure(flash_state: &FlashState, wifi: &EspMyceliumWifi) -> Result<(), AppError> {
    let connection = EspHttpConnection::new(&esp_idf_svc::http::client::Configuration {
        use_global_ca_store: true,
        buffer_size_tx: Some(1536),
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        ..Default::default()
    })?;
    let client = &mut Client::wrap(connection);

    let wifi_settings = flash_state.get_wifi_settings()?;
    wifi.connect(wifi_settings)?;
    let wallet = extract_wallet(client, &flash_state)?;
    let station_id = flash_state.get_station_id()?;
    let now = EspSystemTime{}.now().as_secs();
    let rfc3339 = timestamp_to_rfc3389(now).ok_or(AppError::TokenWallet(TokenWalletError::TimeSyncTimeout))?;

    mycelium::check_in(client, &wallet.access_token, &station_id, vec![StationMeasurement::random(rfc3339)])?;

    Ok(())
}

fn operational(flash_state: &FlashState) -> ! {

    let peripherals = Peripherals::take().unwrap();
    let modem = peripherals.modem;
    let sysloop = EspSystemEventLoop::take().unwrap();
    let esp_wifi = EspWifi::new(modem, sysloop.clone(), None).unwrap();
    let wifi = EspMyceliumWifi::new(sysloop, esp_wifi);

    let result = retry(Fixed::from_millis(1000).take(2), || {
        measure(&flash_state, &wifi)
    });

    match result {
        Ok(_) => {
            flash_state.reset_errors().unwrap();
        },
        Err(err) => {
            error!("Error: {:?}", err);
            flash_state.increment_errors().unwrap()
        }
    }

    if flash_state.get_num_errors().unwrap() == 10 {
        flash_state.erase_settings().unwrap();
    }

    unsafe {
        let second = 1000000;
        let minute = 60 * second;
        esp_sleep_enable_timer_wakeup(5 * minute);
        esp_sleep_pd_config(esp_sleep_pd_domain_t_ESP_PD_DOMAIN_RTC_PERIPH, esp_sleep_pd_option_t_ESP_PD_OPTION_OFF);
        esp_sleep_pd_config(esp_sleep_pd_domain_t_ESP_PD_DOMAIN_XTAL, esp_sleep_pd_option_t_ESP_PD_OPTION_OFF);
        esp_deep_sleep_disable_rom_logging();
        esp_deep_sleep_start();
    }
}

fn onboarding(flash_state: &FlashState) -> ! {
    let state = Arc::new(RwLock::new(OnboardingState::AwaitingSettings));
    let state_write = state.clone();
    let (tx, rx) = channel::<Vec<u8>>(4);
    let peripherals = Peripherals::take().unwrap();
    let modem = peripherals.modem;
    let sysloop = EspSystemEventLoop::take().unwrap();
    let esp_wifi = EspWifi::new(modem, sysloop.clone(), None).unwrap();
    let wifi = EspMyceliumWifi::new(sysloop, esp_wifi);

    let current_state = Characteristic::new(BleUuid::from_uuid128_string("00467768-6228-2272-4663-277478269001"))
        .name("Current state")
        .permissions(AttributePermissions::new().read())
        .properties(CharacteristicProperties::new().read().notify())
        .show_name()
        .on_read(move |_| {
            let s = state.read().unwrap().clone();
            return to_vec(&s).unwrap();
        })
        .build();

    let rpc_command = Characteristic::new(BleUuid::from_uuid128_string("00467768-6228-2272-4663-277478269002"))
        .name("RPC command handler")
        .permissions(AttributePermissions::new().write().read())
        .properties(CharacteristicProperties::new().write().read())
        .on_write(move |bytes, _| { tx.send(bytes).unwrap() })
        .on_read(|_| vec![])
        .show_name()
        .build();

    let service = Service::new(BleUuid::from_uuid128_string("00467768-6228-2272-4663-277478269000"))
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
            process_message(&flash_state, &state_write, &wifi, &bytes);
        }

        std::thread::sleep(Duration::from_secs(5));
    }
}

fn  process_initialize(flash_state: &FlashState, state_write: &Arc<RwLock<OnboardingState>>, wifi: &EspMyceliumWifi, settings: &OnboardingSettings) -> Result<(), AppError> {
    let connection = EspHttpConnection::new(&esp_idf_svc::http::client::Configuration {
        use_global_ca_store: true,
        buffer_size_tx: Some(1536),
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        ..Default::default()
    })?;
    let client = &mut Client::wrap(connection);

    *state_write.write()? = OnboardingState::ProvisioningWifi;

    let wifi_settings = flash_state.get_opt_wifi_settings()?.filter(|x: &MyceliumWifiSettings| x.ssid == settings.wifi_ssid).unwrap_or(settings.clone().wifi_settings());
    let enriched_settings = wifi.connect(wifi_settings)?;

    flash_state.set_wifi_settings(enriched_settings)?;

    let resp = auth0::request_device_code(client)?;

    info!("Got url: {:?}", resp.verification_uri_complete);

    *state_write.write()? = OnboardingState::AwaitingAuthorization { url: resp.verification_uri_complete };

    let mut authenticated = false;

    while authenticated == false {

        match auth0::poll_token(client, &resp.device_code) {
            Ok(TokenResult::Error { error }) => warn!("Auth0 error {:?}", error),
            Ok(TokenResult::AccessToken { .. }) => info!("Skipping!"),
            Ok(TokenResult::Full { access_token, refresh_token, expires_in }) => {
                let wallet = TokenWallet::new(access_token.clone(), refresh_token, expires_in)?;

                flash_state.set_token_wallet(wallet)?;

                let mac_addr_str = get_mac_addr()?;

                let station_id = mycelium::insert_plant(
                    client,
                    &access_token,
                    &StationInsert {
                        mac: mac_addr_str,
                        name: settings.name.clone(),
                        location: settings.location.clone(),
                        description: settings.description.clone(),
                        watering_schedule: WateringSchedule::Threshold { below_soil_pf: 500, period: heapless::String::from("5 seconds") }
                    }
                )?;

                flash_state.set_station_id(station_id)?;

                *state_write.write()? = OnboardingState::Complete;
                authenticated = true;
            }
            Err(err) => warn!("Auth0 error {:?}", err),
        }

        std::thread::sleep(Duration::from_secs(5))
    }

    Ok(())
}

fn process_message(flash_state: &FlashState, state_write: &Arc<RwLock<OnboardingState>>, wifi: &EspMyceliumWifi, bytes: &[u8]) {

    match from_slice::<OnboardingCommand>(&bytes)  {
        Ok(OnboardingCommand::Initialize { settings }) => {
            let result = retry(Fixed::from_millis(10).take(5), || {
                process_initialize(&flash_state, &state_write, &wifi, &settings).map_err(|err| {
                    error!("Error: {:?}", err);
                    err
                })
            });

            match result {
                Ok(_) => (),
                Err(err) => {
                    let error = format!("{:?}", err);
                    *state_write.write().unwrap() = OnboardingState::Failed { error: heapless::String::from(error.as_str()) }
                }
            }
        },
        Ok(OnboardingCommand::Reboot) => {
            unsafe {
                esp_restart();
            }
        },
        Err(err) => error!("Command not recognized! {:?}", err)
    }
}

fn get_mac_addr() -> Result<heapless::String<17>, AppError> {
    let netif = EspNetif::new(NetifStack::Eth)?;
    let mac = netif.get_mac()?;
    let mac_addr_str = heapless::String::from(format!("{:<02X}:{:<02X}:{:<02X}:{:<02X}:{:<02X}:{:<02X}", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]).as_str());

    Ok(mac_addr_str)
}

// returns a rfc3389 - 2018-01-26T18:30:09Z
fn timestamp_to_rfc3389(timestamp: u64) -> Option<String> {
    NaiveDateTime::from_timestamp_opt(timestamp as i64, 0)
        .map(|x| Utc.from_utc_datetime(&x).to_rfc3339_opts(SecondsFormat::Secs, true))
}