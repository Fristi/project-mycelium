use embedded_svc::storage::RawStorage;
use esp_idf_svc::nvs::EspDefaultNvs;
use esp_idf_sys::EspError;
use serde::{Deserialize, Serialize};
use rmp_serde::decode::{from_slice, from_read, Error as RmpDecodeError};
use rmp_serde::encode::{to_vec, Error as RmpEncodeError};
use rmp_serde::Serializer;
use serde::de::DeserializeOwned;

#[derive(Debug)]
pub enum KvStoreError {
    Esp(EspError),
    Encode(RmpEncodeError),
    Decode(RmpDecodeError)
}

pub trait KvStore {
    fn get<T : DeserializeOwned>(&self, key: &str) -> Result<Option<T>, KvStoreError>;
    fn set<T : Serialize>(&mut self, key: &str, value: T) -> Result<(), KvStoreError>;
    fn contains(&self, key: &str) -> Result<bool, KvStoreError>;
}

pub struct NvsKvsStore {
    nvs: EspDefaultNvs
}

impl NvsKvsStore {
    pub fn new(nvs: EspDefaultNvs) -> NvsKvsStore { NvsKvsStore { nvs } }
}

impl KvStore for NvsKvsStore {
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, KvStoreError> {
        let buf: &mut [u8; 255] = &mut [0u8;255];

        if self.nvs.get_raw(key, buf)?.is_some() {
            let res = from_slice(buf).map_err(|err| KvStoreError::Decode(err))?;
            Ok(Some(res))
        } else {
            Ok(None)
        }
    }

    fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), KvStoreError> {
        let bytes = to_vec(&value).map_err(|err| KvStoreError::Encode(err))?;
        self.nvs.set_raw(key, bytes.as_slice())?;
        Ok(())
    }

    fn contains(&self, key: &str) -> Result<bool, KvStoreError> {
        let res = self.nvs.contains(key)?;
        Ok(res)
    }
}

impl From<EspError> for KvStoreError {
    fn from(value: EspError) -> Self {
        KvStoreError::Esp(value)
    }
}