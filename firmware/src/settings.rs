use uuid::Uuid;
use crate::kv::{KvStore, KvStoreError, NvsKvStore};
use crate::tokens::TokenWallet;
use crate::wifi::MyceliumWifiSettings;

pub struct FlashState {
    kv: NvsKvStore
}

impl FlashState {
    pub fn new(kv: NvsKvStore) -> FlashState { FlashState { kv } }

    pub fn set_wifi_settings(&self, s: MyceliumWifiSettings) -> Result<(), KvStoreError> {
        self.kv.set("wifi", s)
    }
    pub fn get_wifi_settings(&self) -> Result<MyceliumWifiSettings, KvStoreError> {
        self.kv.get("wifi")
    }
    pub fn get_opt_wifi_settings(&self) -> Result<Option<MyceliumWifiSettings>, KvStoreError> {
        self.kv.get_opt("wifi")
    }

    pub fn set_token_wallet(&self, wallet: TokenWallet) -> Result<(), KvStoreError> {
        self.kv.set("token_wallet", wallet)
    }
    pub fn get_token_wallet(&self) -> Result<TokenWallet, KvStoreError> {
        self.kv.get("token_wallet")
    }

    pub fn set_station_id(&self, id: Uuid) -> Result<(), KvStoreError> {
        self.kv.set("station_id", id)
    }
    pub fn get_station_id(&self) -> Result<Uuid, KvStoreError> {
        self.kv.get("station_id")
    }
    pub fn has_station_id(&self) -> Result<bool, KvStoreError> {
        self.kv.contains("station_id")
    }

    pub fn reset_errors(&self) -> Result<(), KvStoreError> {
        self.kv.set("num_errors", 0u32)
    }

    pub fn increment_errors(&self) -> Result<(), KvStoreError> {
        let current = self.get_num_errors()?;
        self.kv.set("num_errors", current + 1)
    }
    pub fn get_num_errors(&self) -> Result<u32, KvStoreError> {
        Ok(self.kv.get_opt("num_errors")?.unwrap_or(0u32))
    }

    pub fn erase_settings(&self) -> Result<(), KvStoreError> {
        self.kv.remove("num_errors")?;
        self.kv.remove("station_id")?;
        self.kv.remove("wifi")?;
        self.kv.remove("token_wallet")?;

        Ok(())
    }
}