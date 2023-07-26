mod ble;
mod wifi;
mod kv;
mod onboarding;
mod auth0;

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported


use std::time::Duration;
use esp_idf_hal::prelude::*;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::{EspDefaultNvs, EspDefaultNvsPartition};
use esp_idf_svc::wifi::EspWifi;
use esp_idf_sys::{esp_get_free_heap_size, esp_get_free_internal_heap_size};


use crate::auth0::{Auth0};
use crate::ble::onboarding;
use crate::wifi::{EspMyceliumWifi};
use crate::kv::{NvsKvStore};


fn main() -> ! {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let nvs_partition = EspDefaultNvsPartition::take().unwrap();
    let nvs = EspDefaultNvs::new(nvs_partition, "mycelium", true).unwrap();
    let kv = NvsKvStore::new(nvs);

    // let _sntp = EspSntp::new_default().unwrap();
    let sysloop = EspSystemEventLoop::take().unwrap();
    let peripherals = Peripherals::take().unwrap();
    let modem = peripherals.modem;
    let wifi = EspWifi::new(modem, sysloop.clone(), None).unwrap();
    let mycelium_wifi = EspMyceliumWifi::new(sysloop, wifi);
    let auth = Auth0 { };

    onboarding(mycelium_wifi, kv, auth);

    loop {
        std::thread::sleep(Duration::from_secs(5));
        unsafe {
            let x = esp_get_free_heap_size();
            let y = esp_get_free_internal_heap_size();
            log::info!("Free heap: {} bytes, free internal heap: {} bytes", x, y);
        }
    }

}