use std::alloc::System;
use std::time::{Duration, SystemTime};
use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use esp_idf_svc::systime::EspSystemTime;
use esp_idf_sys::EspError;
use heapless::String;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct TokenWallet {
    pub access_token: String<756>,
    pub refresh_token: String<128>,
    expires_at: u64
}

impl TokenWallet {
    pub fn needs_refresh(&self) -> Result<bool, TokenWalletError> {
        let now = get_time()?;
        info!("now: {}, expires_at: {}", now, self.expires_at);
        Ok(now > self.expires_at)
    }

    pub fn update(self, access_token: String<756>, expires_in: u64)  -> Result<TokenWallet, TokenWalletError> {
        TokenWallet::new(access_token, self.refresh_token, expires_in)
    }

    pub fn new(access_token: String<756>, refresh_token: String<128>, expires_in: u64) -> Result<TokenWallet, TokenWalletError> {
        let now = get_time()?;
        Ok(TokenWallet { access_token, refresh_token, expires_at: now + expires_in })
    }
}

fn get_time() -> Result<u64, TokenWalletError> {
    let sntp = EspSntp::new_default()?;
    let mut counter = 0;

    while sntp.get_sync_status() != SyncStatus::Completed {
        if counter == 300 {
            return Err(TokenWalletError::TimeSyncTimeout)
        }
        std::thread::sleep(Duration::from_millis(100));
        counter += 1;
    }

    return Ok(EspSystemTime{}.now().as_secs())
}

#[derive(Debug)]
pub enum TokenWalletError {
    Esp(EspError),
    TimeSyncTimeout
}

impl From<EspError> for TokenWalletError {
    fn from(value: EspError) -> Self {
        TokenWalletError::Esp(value)
    }
}