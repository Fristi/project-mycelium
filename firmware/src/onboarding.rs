use std::sync::{Arc, PoisonError, RwLock, RwLockWriteGuard};
use std::time::Duration;
use esp_idf_sys::EspError;
use serde::{Deserialize, Serialize};
use serde_json::de::from_slice;
use heapless::String;

use crate::auth0::{Auth, AuthError, TokenResult};
use crate::kv::{KvStore, KvStoreError};
use crate::wifi::{MyceliumWifi, MyceliumWifiSettings};

#[derive(Deserialize, Debug)]
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

pub struct OnboardingHandler<W, N, A> {
    wifi: W,
    kv: N,
    auth: A
}

impl <W : 'static, N: 'static, A: 'static> OnboardingHandler<W, N, A> {

    pub fn new(wifi: W, kv: N, auth: A) -> OnboardingHandler<W, N, A> where W : MyceliumWifi, N : KvStore, A : Auth {
        OnboardingHandler { wifi, kv, auth }
    }


    pub fn handle(this: Arc<Self>, bytes: &Vec<u8>, state: Arc<RwLock<OnboardingState>>) where W : MyceliumWifi, N : KvStore, A : Auth {
        let b = bytes.clone();
        let s = state.clone();
        let wifi = this.wifi.clone();
        let kv = this.kv.clone();
        let auth = this.auth.clone();

        let _ = std::thread::spawn(move || {
            let settings = from_slice::<OnboardingSettings>(b.as_slice()).unwrap();
            *state.write().unwrap() = OnboardingState::ProvisioningWifi;

            let enriched_settings = wifi.connect(settings.wifi_settings()).unwrap();

            kv.set("wifi_settings", enriched_settings).unwrap();

            let resp = auth.request_device_code().unwrap();

            *state.write().unwrap() = OnboardingState::AwaitingAuthorization { url: resp.verification_uri_complete };

            // println!("Got response: {:?}", &resp);

            let mut authenticated = false;

            while authenticated == false {
                match auth.poll_token(&resp.device_code) {
                    Ok(TokenResult::Error { error }) => println!("Auth0 error {:?}", error),
                    Ok(TokenResult::AccessToken { .. }) => println!("Skipping!"),
                    Ok(TokenResult::Full { access_token, refresh_token, expires_in }) => {
                        kv.set("refresh_token", refresh_token).unwrap();
                        kv.set("access_token", access_token).unwrap();
                        kv.set("expires_in", expires_in).unwrap();

                        *state.write().unwrap() = OnboardingState::Complete;
                        authenticated = true;
                    },
                    Err(err) => println!("Auth0 error {:?}", err),
                }

                std::thread::sleep(Duration::from_secs(5))
            }
        });
    }
}

impl <W : Clone, N : Clone, A : Clone> Clone for OnboardingHandler<W, N, A> {
    fn clone(&self) -> Self {
        OnboardingHandler { wifi: self.wifi.clone(), auth: self.auth.clone(), kv: self.kv.clone() }
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