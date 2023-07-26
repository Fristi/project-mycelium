use std::str::Utf8Error;

use embedded_svc::http::client::Client;
use embedded_svc::io::Write;
use esp_idf_svc::errors::EspIOError;
use esp_idf_svc::http::client::EspHttpConnection;
use heapless::String;
use serde::{Deserialize};
use serde::de::DeserializeOwned;
use serde_json::{from_str};

pub trait Auth : Send + Sync + Clone {
    fn refresh_token(&self, refresh_token: &str) -> Result<TokenResult, AuthError>;
    fn poll_token(&self, device_code: &str) -> Result<TokenResult, AuthError>;
    fn request_device_code(&self) -> Result<DeviceCodeResponse, AuthError>;
}

#[derive(Deserialize, Debug)]
pub struct DeviceCodeResponse {
    pub device_code: String<128>,
    pub user_code: String<128>,
    pub verification_uri: String<255>,
    pub verification_uri_complete: String<255>,
    pub expires_in: u32,
    pub interval: u64
}

#[derive(Debug)]
pub enum AuthError {
    Json(serde_json::Error),
    String(Utf8Error),
    IO(EspIOError)
}

#[derive(Deserialize, Debug)]
pub enum TokenStatus {
    #[serde(rename = "authorization_pending")]
    AuthorizationPending,
    #[serde(rename = "slow_down")]
    SlowDown,
    #[serde(rename = "expired_token")]
    ExpiredToken,
    #[serde(rename = "access_denied")]
    AccessDenied,
    #[serde(rename = "invalid_grant")]
    InvalidGrant,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum TokenResult {
    Full { access_token: String<756>, refresh_token: String<128>, expires_in: i64 },
    AccessToken { access_token: String<756>, expires_in: i64 },
    Error { error: TokenStatus }
}

pub struct Auth0 {}

unsafe impl Send for Auth0 { }
unsafe impl Sync for Auth0 { }

const DOMAIN: &str = "dev-plq6-asi.eu.auth0.com";
const CLIENT_ID: &str = "5nYFEjhKlvTPheFxEDIEo97wLx3auwB7";
const CLIENT_SECRET: &str = "zp-7XzX4rP-ihysBSPoF2fXLfQRAxv2WnJEw-dp4f2LEa_rN8T2gU4fU-OqxWg4I";
const SCOPE: &str = "offline_access";
const AUDIENCE: &str = "https://mycelium.co";

impl Auth0{

    fn post_form<T, const N : usize>(&self, url: &str, payload: [(&str, &str); N]) -> Result<T, AuthError> where T : DeserializeOwned {

        let connection = EspHttpConnection::new(&esp_idf_svc::http::client::Configuration {
            use_global_ca_store: true,
            crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
            ..Default::default()
        }).unwrap();

        let client = &mut Client::wrap(connection);

        let payload_str = payload.map(|(k, v)| format!("{}={}", k, v)).join("&");
        let payload = payload_str.as_bytes();
        let payload_length = format!("{}", payload.len());

        let headers = [
            ("content-type", "application/x-www-form-urlencoded"),
            ("content-length", &*payload_length),
        ];

        let mut request = client.post(url, &headers)?;
        request.write_all(payload)?;
        request.flush()?;

        let response = &mut request.submit()?;
        let (_, body) = response.split();

        let mut buf = [0u8; 1536];
        embedded_svc::io::Read::read(body, &mut buf)?;
        let str = std::str::from_utf8(&buf)?.trim_matches(char::from(0));
        let res = from_str(str)?;

        Ok(res)
    }
}

impl Clone for Auth0 {
    fn clone(&self) -> Self {
        Auth0 { }
    }
}

impl Auth for Auth0 {
    fn refresh_token(&self, refresh_token: &str) -> Result<TokenResult, AuthError> {
        self.post_form(&format!("https://{}/oauth/token", DOMAIN), [("client_id", CLIENT_ID), ("client_secret", CLIENT_SECRET), ("grant_type", "refresh_token"), ("refresh_token", refresh_token)])
    }

    fn poll_token(&self, device_code: &str) -> Result<TokenResult, AuthError> {
        self.post_form(&format!("https://{}/oauth/token", DOMAIN), [("client_id", CLIENT_ID), ("device_code", device_code), ("grant_type", "urn:ietf:params:oauth:grant-type:device_code")])
    }

    fn request_device_code(&self) -> Result<DeviceCodeResponse, AuthError> {
        self.post_form(&format!("https://{}/oauth/device/code", DOMAIN), [("client_id", CLIENT_ID), ("scope", SCOPE), ("audience", AUDIENCE)])
    }
}

impl From<Utf8Error> for AuthError {
    fn from(value: Utf8Error) -> Self {
        AuthError::String(value)
    }
}

impl From<EspIOError> for AuthError {
    fn from(value: EspIOError) -> Self {
        AuthError::IO(value)
    }
}

impl From<serde_json::Error> for AuthError {
    fn from(value: serde_json::Error) -> Self {
        AuthError::Json(value)
    }
}