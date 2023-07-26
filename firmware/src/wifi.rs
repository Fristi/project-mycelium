use std::sync::{Arc, Mutex, MutexGuard};
// based on https://github.com/ferrous-systems/espressif-trainings/blob/1ec7fd78660c58739019b4c146634077a08e3d5e/common/lib/esp32-c3-dkc02-bsc/src/wifi.rs
// based on https://github.com/ivmarkov/rust-esp32-std-demo/blob/main/src/main.rs
use embedded_svc::wifi::{AccessPointInfo, AuthMethod, ClientConfiguration, Configuration};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi, NonBlocking, WifiDeviceId, WifiDriver};
use esp_idf_svc::wifi::config::ScanConfig;
use heapless::String;
use log::info;
use esp_idf_sys::EspError;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MyceliumWifiSettings {
    pub ssid: String<32>,
    pub password: String<64>,
    pub channel: Option<u8>,
    pub bssid: Option<[u8; 6]>,
}

impl MyceliumWifiSettings {
    pub fn basic(ssid: String<32>, password: String<64>) -> MyceliumWifiSettings {
        MyceliumWifiSettings { ssid, password, channel: None, bssid: None }
    }
}

pub trait MyceliumWifi : Send + Sync + Clone {
    fn connect(&self, settings: MyceliumWifiSettings) -> Result<MyceliumWifiSettings, EspError>;
}

pub struct EspMyceliumWifi {
    esp_wifi: Arc<Mutex<EspWifi<'static>>>,
    sysloop: Arc<Mutex<EspSystemEventLoop>>
}

impl EspMyceliumWifi {
    pub fn new(sysloop: EspSystemEventLoop, wifi: EspWifi<'static>) -> EspMyceliumWifi {
        EspMyceliumWifi { esp_wifi: Arc::new(Mutex::new(wifi)), sysloop: Arc::new(Mutex::new(sysloop)) }
    }
}

impl Clone for EspMyceliumWifi {
    fn clone(&self) -> Self {
        EspMyceliumWifi { esp_wifi: self.esp_wifi.clone(), sysloop: self.sysloop.clone() }
    }
}

impl MyceliumWifi for EspMyceliumWifi {

    fn connect(&self, settings: MyceliumWifiSettings) -> Result<MyceliumWifiSettings, EspError> {
        let sysloop = self.sysloop.lock().unwrap();
        let esp_wifi = &mut (*self.esp_wifi.lock().unwrap());
        let wifi = &mut BlockingWifi::wrap(esp_wifi, sysloop.clone())?;

        let mut auth_method = AuthMethod::WPA2Personal;

        if settings.password.is_empty() {
            auth_method = AuthMethod::None;
        }


        wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;
        wifi.start()?;

        let enriched_settings = if settings.channel.is_none() && settings.bssid.is_none() {
            info!("Searching for WiFi network {}", settings.ssid);

            let ap_infos = wifi.scan()?;
            let ours = ap_infos.into_iter().find(|a| a.ssid.eq(&settings.ssid));

            if let Some(ours) = ours {
                info!("Found configured access point {} on channel {}", settings.ssid, ours.channel);
                Ok(MyceliumWifiSettings { channel: Some(ours.channel), bssid: Some(ours.bssid), ..settings })
            } else {
                info!("Configured access point {} not found during scanning, will go with unknown channel", settings.ssid);
                Ok(settings)
            }
        } else {
            Ok(settings)
        }?;
        let conf = Configuration::Client(ClientConfiguration {
            ssid: enriched_settings.ssid.clone(),
            password: enriched_settings.password.clone(),
            channel: enriched_settings.channel,
            bssid: enriched_settings.bssid,
            auth_method,
            ..Default::default()
        });

        wifi.set_configuration(&conf)?;
        wifi.connect()?;
        wifi.wait_netif_up()?;

        Ok(enriched_settings.clone())
    }
}

unsafe impl Send for EspMyceliumWifi { }
unsafe impl Sync for EspMyceliumWifi { }