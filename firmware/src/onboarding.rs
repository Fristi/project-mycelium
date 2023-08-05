use std::sync::{Arc, PoisonError, RwLock, RwLockWriteGuard};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use esp_idf_sys::EspError;
use serde::{Deserialize, Serialize};
use serde_json::de::from_slice;
use heapless::String;
use thingbuf::mpsc::blocking::*;
use thingbuf::recycling::DefaultRecycle;

use crate::auth0::{Auth, Auth0, AuthError, TokenResult};
use crate::kv::{KvStore, KvStoreError, NvsKvStore};
use crate::wifi::{EspMyceliumWifi, MyceliumWifi, MyceliumWifiSettings};

#[derive(Deserialize, Clone, Default, Debug)]
pub struct OnboardingSettings {
    name: String<255>,
    location: String<255>,
    description: String<255>,
    wifi_ssid: String<32>,
    wifi_password: String<64>
}

impl OnboardingSettings {
    pub fn wifi_settings(self) -> MyceliumWifiSettings {
        MyceliumWifiSettings::basic(self.wifi_ssid, self.wifi_password)
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "_type")]
pub enum OnboardingState {
    AwaitingSettings,
    ProvisioningWifi,
    Failed { error: String<256> },
    AwaitingAuthorization { url: String<255> },
    Complete
}

#[derive(Debug)]
pub enum OnboardingError {
    RwLock,
    Kv(KvStoreError),
    Auth(AuthError),
    Json(serde_json::Error),
    Esp(EspError)
}

pub struct OnboardingHandler{
    tx: Sender<OnboardingSettings, DefaultRecycle>,
    handle: JoinHandle<()>
}

impl OnboardingHandler {

    // pub fn new(wifi: EspMyceliumWifi, kv: NvsKvStore, auth: Auth0, state: Arc<RwLock<OnboardingState>>) -> OnboardingHandler {
    pub fn new(tx: Sender<OnboardingSettings>, rx: Receiver<OnboardingSettings>) -> OnboardingHandler {

        let handle = thread::spawn(move || {
            if let Some(settings) = rx.recv() {
                println!("Settings {:?}", settings);
            }
        });

        // let _ = thread::spawn(move || {
        //     if let Some(settings) = rx.recv() {
        //         *state.write().unwrap() = OnboardingState::ProvisioningWifi;
        //
        //         let enriched_settings = wifi.connect(settings.wifi_settings()).unwrap();
        //
        //         kv.set("wifi_settings", enriched_settings).unwrap();
        //
        //         let resp = auth.request_device_code().unwrap();
        //
        //         *state.write().unwrap() = OnboardingState::AwaitingAuthorization { url: resp.verification_uri_complete };
        //
        //         // println!("Got response: {:?}", &resp);
        //
        //         let mut authenticated = false;
        //
        //         while authenticated == false {
        //             match auth.poll_token(&resp.device_code) {
        //                 Ok(TokenResult::Error { error }) => println!("Auth0 error {:?}", error),
        //                 Ok(TokenResult::AccessToken { .. }) => println!("Skipping!"),
        //                 Ok(TokenResult::Full { access_token, refresh_token, expires_in }) => {
        //                     kv.set("refresh_token", refresh_token).unwrap();
        //                     kv.set("access_token", access_token).unwrap();
        //                     kv.set("expires_in", expires_in).unwrap();
        //
        //                     *state.write().unwrap() = OnboardingState::Complete;
        //                     authenticated = true;
        //                 },
        //                 Err(err) => println!("Auth0 error {:?}", err),
        //             }
        //
        //             std::thread::sleep(Duration::from_secs(5))
        //         }
        //     }
        // });

        return OnboardingHandler { tx, handle }
    }


    pub fn handle(&self, bytes: &Vec<u8>) {
        let msg = from_slice(bytes).unwrap();
        self.tx.send(msg).unwrap();
    }
}

impl From<EspError> for OnboardingError {
    fn from(value: EspError) -> Self {
        OnboardingError::Esp(value)
    }
}

impl From<serde_json::Error> for OnboardingError {
    fn from(value: serde_json::Error) -> Self {
        OnboardingError::Json(value)
    }
}

impl From<AuthError> for OnboardingError {
    fn from(value: AuthError) -> Self {
        OnboardingError::Auth(value)
    }
}

impl From<KvStoreError> for OnboardingError {
    fn from(value: KvStoreError) -> Self {
        OnboardingError::Kv(value)
    }
}

impl From<PoisonError<RwLockWriteGuard<'_, OnboardingState>>> for OnboardingError {
    fn from(_value: PoisonError<RwLockWriteGuard<'_, OnboardingState>>) -> Self {
        OnboardingError::RwLock
    }
}