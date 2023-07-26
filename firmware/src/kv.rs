use std::sync::{Arc, Mutex};
use esp_idf_svc::nvs::EspDefaultNvs;
use serde::{Serialize};
use serde::de::DeserializeOwned;

use serde_json::ser::{to_string};
use serde_json::{from_str};

#[derive(Debug)]
pub enum KvStoreError {
    Esp(esp_idf_sys::EspError),
    Json(serde_json::Error),
    StringConversionError
}

pub trait KvStore : Send + Sync + Clone {
    fn get<T : DeserializeOwned>(&self, key: &str) -> Result<Option<T>, KvStoreError>;
    fn set<T : Serialize>(&self, key: &str, value: T) -> Result<(), KvStoreError>;
    fn contains(&self, key: &str) -> Result<bool, KvStoreError>;
    fn remove(&self, key: &str) -> Result<(), KvStoreError>;
}

pub struct NvsKvStore {
    pub nvs: Arc<Mutex<EspDefaultNvs>>
}

impl NvsKvStore {
    pub fn new(nvs: EspDefaultNvs) -> NvsKvStore { NvsKvStore { nvs: Arc::new(Mutex::new(nvs)) } }
}

impl Clone for NvsKvStore {
    fn clone(&self) -> Self {
        NvsKvStore { nvs: self.nvs.clone() }
    }
}

impl KvStore for NvsKvStore {
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, KvStoreError> {
        let buf: &mut [u8; 2048] = &mut [0u8;2048];
        let nvs = self.nvs.lock().unwrap();

        if nvs.get_raw(key, buf)?.is_some() {
            let contents = String::from_utf8(buf.to_vec()).map_err(|_| KvStoreError::StringConversionError)?;
            let res = from_str::<T>(contents.trim_matches(char::from(0)))?;
            Ok(Some(res))
        } else {
            Ok(None)
        }
    }

    fn set<T: Serialize>(&self, key: &str, value: T) -> Result<(), KvStoreError> {
        let str = to_string(&value)?;
        let nvs = &mut self.nvs.lock().unwrap();
        nvs.set_raw(key, str.as_bytes())?;
        Ok(())
    }

    fn contains(&self, key: &str) -> Result<bool, KvStoreError> {
        let nvs = self.nvs.lock().unwrap();
        let res = nvs.contains(key)?;
        Ok(res)
    }

    fn remove(&self, key: &str) -> Result<(), KvStoreError> {
        let nvs = &mut self.nvs.lock().unwrap();
        nvs.remove(key).map_err(KvStoreError::Esp)?;
        Ok(())
    }
}

impl From<esp_idf_sys::EspError> for KvStoreError {
    fn from(value: esp_idf_sys::EspError) -> Self {
        KvStoreError::Esp(value)
    }
}

impl From<serde_json::Error> for KvStoreError {
    fn from(value: serde_json::Error) -> Self {
        KvStoreError::Json(value)
    }
}

unsafe impl Send for NvsKvStore { }
unsafe impl Sync for NvsKvStore { }