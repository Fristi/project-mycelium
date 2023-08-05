mod ble;
mod wifi;
mod kv;
mod onboarding;
mod auth0;

use std::sync::{Arc, PoisonError, RwLock, RwLockWriteGuard};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported


use std::time::Duration;
use bluedroid::gatt_server::{Characteristic, GLOBAL_GATT_SERVER, Profile, Service};
use bluedroid::utilities::{AttributePermissions, BleUuid, CharacteristicProperties};
use embedded_svc::http::headers;
use esp_idf_hal::prelude::*;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::{EspDefaultNvs, EspDefaultNvsPartition};
use esp_idf_svc::wifi::EspWifi;
use esp_idf_sys::{esp_get_free_heap_size, esp_get_free_internal_heap_size, EspError};
use retry::delay::Fixed;
use retry::retry;
use serde_json::{from_slice, to_vec};
use thingbuf::mpsc::blocking::channel;


use crate::auth0::{Auth, Auth0, AuthError, TokenResult};
use crate::wifi::{EspMyceliumWifi, MyceliumWifi};
use crate::kv::{KvStore, KvStoreError, NvsKvStore};
use crate::onboarding::{OnboardingError, OnboardingHandler, OnboardingSettings, OnboardingState};

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
    //
    // // let _sntp = EspSntp::new_default().unwrap();
    let sysloop = EspSystemEventLoop::take().unwrap();
    let peripherals = Peripherals::take().unwrap();
    let modem = peripherals.modem;
    let esp_wifi = EspWifi::new(modem, sysloop.clone(), None).unwrap();
    let wifi = EspMyceliumWifi::new(sysloop, esp_wifi);
    let auth = Auth0 {};
    let state = Arc::new(RwLock::new(OnboardingState::AwaitingSettings));
    let state_write = state.clone();

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


    // let _ = std::thread::spawn(move || {
    //     println!("Starting handler");
    //
    // });


    loop {
        if let Some(bytes) = rx.recv() {

            let result = retry(Fixed::from_millis(10).take(5), || {
                process_message(&wifi, &kv, &auth, &state_write, &bytes)
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

fn process_message(wifi: &EspMyceliumWifi, kv: &NvsKvStore, auth: &Auth0, state_write: &Arc<RwLock<OnboardingState>>, bytes: &[u8]) -> Result<(), OnboardingError> {
    let settings = from_slice::<OnboardingSettings>(&bytes)?;

    *state_write.write()? = OnboardingState::ProvisioningWifi;

    let enriched_settings = wifi.connect(settings.wifi_settings())?;

    kv.set("wifi_settings", enriched_settings)?;

    let resp = auth.request_device_code()?;

    *state_write.write()? = OnboardingState::AwaitingAuthorization { url: resp.verification_uri_complete };

    let mut authenticated = false;

    while authenticated == false {
        match auth.poll_token(&resp.device_code) {
            Ok(TokenResult::Error { error }) => println!("Auth0 error {:?}", error),
            Ok(TokenResult::AccessToken { .. }) => println!("Skipping!"),
            Ok(TokenResult::Full { access_token, refresh_token, expires_in }) => {
                kv.set("refresh_token", refresh_token)?;
                kv.set("access_token", access_token)?;
                kv.set("expires_in", expires_in)?;

                *state_write.write()? = OnboardingState::Complete;
                authenticated = true;
            }
            Err(err) => println!("Auth0 error {:?}", err),
        }

        std::thread::sleep(Duration::from_secs(5))
    }

    Ok(())
}
