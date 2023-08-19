use std::sync::{PoisonError, RwLockWriteGuard};



use esp_idf_sys::EspError;
use serde::{Deserialize, Serialize};

use heapless::String;



use crate::auth0::{AuthError};
use crate::kv::{KvStoreError};
use crate::mycelium::MyceliumError;
use crate::tokens::TokenWalletError;
use crate::wifi::{MyceliumWifiSettings};

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

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "_type")]
pub enum OnboardingCommand {
    Initialize { settings: OnboardingSettings },
    Reboot
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
pub enum AppError {
    RwLock,
    Kv(KvStoreError),
    Auth(AuthError),
    TokenWallet(TokenWalletError),
    Mycelium(MyceliumError),
    Json(serde_json::Error),
    Esp(EspError)
}

impl From<EspError> for AppError {
    fn from(value: EspError) -> Self {
        AppError::Esp(value)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        AppError::Json(value)
    }
}

impl From<MyceliumError> for AppError {
    fn from(value: MyceliumError) -> Self {
        AppError::Mycelium(value)
    }
}

impl From<AuthError> for AppError {
    fn from(value: AuthError) -> Self {
        AppError::Auth(value)
    }
}

impl From<KvStoreError> for AppError {
    fn from(value: KvStoreError) -> Self {
        AppError::Kv(value)
    }
}

impl From<TokenWalletError> for AppError {
    fn from(value: TokenWalletError) -> Self { AppError::TokenWallet(value) }
}

impl From<PoisonError<RwLockWriteGuard<'_, OnboardingState>>> for AppError {
    fn from(_value: PoisonError<RwLockWriteGuard<'_, OnboardingState>>) -> Self {
        AppError::RwLock
    }
}