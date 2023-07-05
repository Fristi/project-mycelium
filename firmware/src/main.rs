mod improv;
mod wifi;
mod kv;
mod ha;

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;
use improv::*;
use bluedroid::gatt_server::{Characteristic, GLOBAL_GATT_SERVER, Profile, Service};
use bluedroid::utilities::{AttributePermissions, BleUuid, CharacteristicProperties};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use embedded_svc::mqtt::client::QoS;
use embedded_svc::wifi::Wifi;
use esp_idf_hal::prelude::*;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::mqtt::client::{EspMqttClient, MqttClientConfiguration};
use esp_idf_svc::nvs::{EspDefaultNvs, EspDefaultNvsPartition};
use esp_idf_svc::wifi::EspWifi;
use esp_idf_sys::{esp_deep_sleep, esp_deep_sleep_start, esp_restart};
use crate::wifi::{EspMyceliumWifi, MyceliumWifiError, MyceliumWifi, MyceliumWifiSettings};
use heapless::String;
use crate::ha::HaConfig;
use crate::kv::{KvStore, NvsKvsStore};

fn main() -> ! {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let sysloop = EspSystemEventLoop::take().unwrap();
    let nvs_partition = EspDefaultNvsPartition::take().unwrap();
    let nvs = EspDefaultNvs::new(nvs_partition, "mycelium", true).unwrap();
    let peripherals = Peripherals::take().unwrap();
    let modem = peripherals.modem;
    let wifi = EspWifi::new(modem, sysloop.clone(), None).unwrap();

    let mut kv = NvsKvsStore::new(nvs);
    let mut wifi_connector = EspMyceliumWifi::new(sysloop.clone(), wifi);


    let wifi_settings: Option<MyceliumWifiSettings> = kv.get("wifi_settings").unwrap();
    let wifi_is_setup = wifi_settings.is_some();
    let mut elapsed = 0;

    match wifi_settings {
        Some(s) => {
            let _ = wifi_connector.connect(s).unwrap();

            let config = MqttClientConfiguration { username: Some("mycelium"), password: Some("!PlantsNeedToBeWater3d!"), ..MqttClientConfiguration::default() };

            let mut client = EspMqttClient::new("mqtt://192.168.1.200:1883", &config, |c| ()).unwrap();
            let topic = "homeassistant/sensor/plant_sensor_1/temperature/config";
            let mut buf = &mut [0u8; 256];
            let ha_config = HaConfig {
                name: String::from("Plant sensor 1"),
                stat_t: String::from(topic),
                unit_of_meas: String::from("%"),
                dev_class: Some(String::from("humidity")),
                frc_upd: true,
                val_tpl: String::from("{{ value_json.humidity|default(0) }}")
            };

            ha_config.as_json(buf).unwrap();

            client.publish(topic, QoS::AtLeastOnce, false, buf).unwrap();
        },
        None => {
            wifi_setup(wifi_connector, kv)
        }
    }

    loop {
        std::thread::sleep(Duration::from_millis(100));
        elapsed += 100;

        if wifi_is_setup && elapsed >= 60000 {
            // 60 seconds
            unsafe {
                esp_deep_sleep(60000000000);
                esp_deep_sleep_start();
            }
        }
    }
}

fn wifi_setup(wifi_connector: EspMyceliumWifi, kv: NvsKvsStore) {
    let state = Arc::new(RwLock::new(ImprovState::Authorized));
    let state_read = state.clone();
    let error = Arc::new(RwLock::new(ImprovError::None));
    let error_read = error.clone();
    let improv_handler = Arc::new(Mutex::new(ImprovHandler::new(wifi_connector, kv, error, state)));

    let current_state = Characteristic::new(BleUuid::from_uuid128_string(IMPROV_STATUS_UUID))
        .name("Current state")
        .permissions(AttributePermissions::new().read())
        .properties(CharacteristicProperties::new().read().notify())
        .show_name()
        .on_read(move |_| {
            let s = state_read.read().unwrap();
            let ss = *s;
            vec![ss.into()]
        })
        .build();

    let error_state = Characteristic::new(BleUuid::from_uuid128_string(IMPROV_ERROR_UUID))
        .name("Error state")
        .permissions(AttributePermissions::new().read())
        .properties(CharacteristicProperties::new().read().notify())
        .show_name()
        .on_read(move |_| {
            let s = error_read.read().unwrap();
            let ss = *s;
            vec![ss.into()]
        })
        .build();

    let rpc_command = Characteristic::new(BleUuid::from_uuid128_string(IMPROV_RPC_COMMAND_UUID))
        .name("RPC command handler")
        .permissions(AttributePermissions::new().write())
        .properties(CharacteristicProperties::new().write())
        .on_write(move |bytes, _| {
            improv_handler.lock().unwrap().handle(&bytes);

        })
        .show_name()
        .build();


    let rpc_result = Characteristic::new(BleUuid::from_uuid128_string(IMPROV_RPC_RESULT_UUID))
        .name("RPC result")
        .permissions(AttributePermissions::new().read())
        .properties(CharacteristicProperties::new().read().notify())
        .show_name()
        .build();

    let capabilities = Characteristic::new(BleUuid::from_uuid128_string(IMPROV_CAPABILITIES_UUID))
        .name("Capabilities")
        .permissions(AttributePermissions::new().read())
        .properties(CharacteristicProperties::new().read())
        .show_name()
        .set_value([0x00])
        .build();


    let service = Service::new(BleUuid::from_uuid128_string(IMPROV_SERVICE_UUID))
        .name("Improv Service")
        .primary()
        .characteristic(&rpc_command)
        .characteristic(&rpc_result)
        .characteristic(&current_state)
        .characteristic(&error_state)
        .characteristic(&capabilities)
        .build();

    let profile = Profile::new(0x0001)
        .name("Default Profile")
        .service(&service)
        .build();

    GLOBAL_GATT_SERVER
        .lock()
        .unwrap()
        .profile(profile)
        .device_name("Improve onboarding")
        .appearance(bluedroid::utilities::Appearance::GenericComputer)
        .advertise_service(&service)
        .start();
}



struct ImprovHandler<W, N> {
    wifi: W,
    settings: N,
    error: Arc<RwLock<ImprovError>>,
    state: Arc<RwLock<ImprovState>>
}

enum ImprovCommandResult<R> {
    Connected(R)
}

impl <W, N> ImprovHandler<W, N> {

    fn new(esp_wifi: W, settings: N, error: Arc<RwLock<ImprovError>>, state: Arc<RwLock<ImprovState>>) -> ImprovHandler<W, N> where W : MyceliumWifi, N : KvStore {
        ImprovHandler { wifi: esp_wifi, settings, error, state }
    }

    fn handle(&mut self, bytes: &Vec<u8>) where W : MyceliumWifi, N : KvStore {
        if let Some(cmd) = ImprovCommand::from_bytes(&bytes.as_slice()).ok() {
            match cmd {
                ImprovCommand::WifiSettings { ssid, password } => {
                    *self.error.write().unwrap() = ImprovError::None;
                    *self.state.write().unwrap() = ImprovState::Provisioning;

                    match self.wifi.connect(MyceliumWifiSettings::basic(ssid, password)) {
                        Ok(res) => {
                            *self.state.write().unwrap() = ImprovState::Provisioned;
                            self.settings.set("wifi_settings", res).unwrap();
                            unsafe { esp_restart() }
                        }
                        Err(wifi_err) => {
                            let err = match wifi_err {
                                MyceliumWifiError::Esp(err) => ImprovError::Unknown,
                                MyceliumWifiError::Timeout => ImprovError::UnableToConnect,
                                MyceliumWifiError::NoIp => ImprovError::NotAuthorized,
                            };
                            *self.error.write().unwrap() = err;
                            *self.state.write().unwrap() = ImprovState::Authorized;
                        }
                    };
                },
                _ => {
                    *self.error.write().unwrap() = ImprovError::InvalidRpc;
                }
            }
        } else {
            *self.error.write().unwrap() = ImprovError::UnknownRpc;
        }
    }
}
