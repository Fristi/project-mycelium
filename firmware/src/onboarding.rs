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

use crate::auth0::{AuthError, TokenResult};
use crate::kv::{KvStore, KvStoreError, NvsKvStore};
use crate::mycelium::MyceliumError;
use crate::wifi::{EspMyceliumWifi, MyceliumWifi, MyceliumWifiSettings};

#[derive(Deserialize, Clone, Default, Debug)]
pub struct OnboardingSettings {
    pub name: String<128>,
    pub location: String<128>,
    pub description: String<128>,
    pub wifi_ssid: String<32>,
    pub wifi_password: String<64>
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
    Mycelium(MyceliumError),
    Json(serde_json::Error),
    Esp(EspError)
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

impl From<MyceliumError> for OnboardingError {
    fn from(value: MyceliumError) -> Self {
        OnboardingError::Mycelium(value)
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