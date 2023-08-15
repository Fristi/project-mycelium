use std::str::Utf8Error;
use std::string::FromUtf8Error;

use embedded_svc::http::client::Client;
use embedded_svc::io::Write;
use esp_idf_svc::errors::EspIOError;
use esp_idf_svc::http::client::EspHttpConnection;
use serde::{Serialize};
use serde::de::DeserializeOwned;
use serde_json::{from_str};
use uuid::Uuid;

#[derive(Debug)]
pub enum MyceliumError {
    Json(serde_json::Error),
    String(FromUtf8Error),
    IO(EspIOError),
    UnexpectedResponse { status: u16 }
}

#[derive(Serialize, Debug)]
#[serde(tag = "_type")]
pub enum WateringSchedule {
    #[serde(rename_all = "camelCase")]
    Interval { schedule: heapless::String<16>, period: heapless::String<30> },
    #[serde(rename_all = "camelCase")]
    Threshold { below_soil_pf: u32, period: heapless::String<30> },
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StationInsert {
    pub mac: heapless::String<17>,
    pub name: heapless::String<128>,
    pub location: heapless::String<128>,
    pub description: heapless::String<128>,
    pub watering_schedule: WateringSchedule
}

pub fn insert_plant(client: &mut Client<EspHttpConnection>, access_token: &heapless::String<756>, insert: &StationInsert) -> Result<Uuid, MyceliumError> {

    let payload_vec = serde_json::to_vec(&insert)?;
    let payload = payload_vec.as_slice();
    let payload_length = format!("{}", payload.len());
    let bearer = format!("bearer {}", access_token);
    let headers = [
        ("content-type", "application/json"),
        ("authorization", bearer.as_str()),
        ("content-length", &*payload_length),
    ];

    let url = "http://reindeer-liked-lamprey.ngrok-free.app/api/stations";

    let mut request = client.post(url, &headers)?;
    request.write_all(payload)?;
    request.flush()?;

    let response = &mut request.submit()?;

    if response.status() == 200 {
        let (_, body) = response.split();
        let mut buf = [0u8; 64];
        embedded_svc::io::Read::read(body, &mut buf)?;
        let contents = String::from_utf8(buf.to_vec())?;
        let uuid = from_str::<Uuid>(contents.trim_matches(char::from(0)))?;

        Ok(uuid)
    } else {
        Err(MyceliumError::UnexpectedResponse { status: response.status() })
    }
}

impl From<FromUtf8Error> for MyceliumError {
    fn from(value: FromUtf8Error) -> Self {
        MyceliumError::String(value)
    }
}
impl From<EspIOError> for MyceliumError {
    fn from(value: EspIOError) -> Self {
        MyceliumError::IO(value)
    }
}

impl From<serde_json::Error> for MyceliumError {
    fn from(value: serde_json::Error) -> Self {
        MyceliumError::Json(value)
    }
}