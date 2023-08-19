
use std::string::FromUtf8Error;

use embedded_svc::http::client::Client;
use embedded_svc::io::Write;
use esp_idf_svc::errors::EspIOError;
use esp_idf_svc::http::client::EspHttpConnection;
use rand::Rng;
use serde::{Serialize};

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

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StationMeasurement {
    on: String,
    battery_voltage: f64,
    temperature: f64,
    humidity: f64,
    lux: f64,
    soil_pf: f64,
    tank_pf: f64
}

impl StationMeasurement {
    pub fn random(on: String) -> StationMeasurement {
        let mut rng = rand::thread_rng();

        StationMeasurement {
            on,
            battery_voltage: rng.gen_range(2.2f64..3.3f64),
            temperature: rng.gen_range(10f64..45f64),
            humidity: rng.gen_range(5f64..100f64),
            lux: rng.gen_range(0f64..300_000f64),
            soil_pf: rng.gen_range(0f64..2000f64),
            tank_pf: rng.gen_range(0f64..2000f64),
        }
    }
}

pub fn check_in(client: &mut Client<EspHttpConnection>, access_token: &heapless::String<756>, station_id: &Uuid, measurements: Vec<StationMeasurement>) -> Result<(), MyceliumError> {
    let payload_vec = serde_json::to_vec(&measurements)?;
    let payload = payload_vec.as_slice();
    let payload_length = format!("{}", payload.len());
    let bearer = format!("Bearer {}", access_token);
    let headers = [
        ("content-type", "application/json"),
        ("authorization", bearer.as_str()),
        ("content-length", &*payload_length),
    ];
    let base_url = option_env!("MYCELIUM_BASE_URL").unwrap_or("http://reindeer-liked-lamprey.ngrok-free.app");
    let url = format!("{}/api/stations/{}/checkin", base_url, station_id);
    let mut request = client.put(url.as_str(), &headers)?;

    request.write_all(payload)?;
    request.flush()?;

    let response = &mut request.submit()?;

    if response.status() == 200 {
        Ok(())
    } else {
        Err(MyceliumError::UnexpectedResponse { status: response.status() })
    }
}

pub fn insert_plant(client: &mut Client<EspHttpConnection>, access_token: &heapless::String<756>, insert: &StationInsert) -> Result<Uuid, MyceliumError> {

    let payload_vec = serde_json::to_vec(&insert)?;
    let payload = payload_vec.as_slice();
    let payload_length = format!("{}", payload.len());
    let bearer = format!("Bearer {}", access_token);
    let headers = [
        ("content-type", "application/json"),
        ("authorization", bearer.as_str()),
        ("content-length", &*payload_length),
    ];

    let base_url = option_env!("MYCELIUM_BASE_URL").unwrap_or("http://reindeer-liked-lamprey.ngrok-free.app");
    let url = format!("{}/api/stations", base_url);

    let mut request = client.post(url.as_str(), &headers)?;

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