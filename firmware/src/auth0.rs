use std::str::Utf8Error;

use embedded_svc::http::client::Client;
use embedded_svc::io::Write;
use esp_idf_svc::errors::EspIOError;
use esp_idf_svc::http::client::EspHttpConnection;
use heapless::String;
use serde::{Deserialize};
use serde::de::DeserializeOwned;
use serde_json::{from_str};


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
    Full { access_token: String<756>, refresh_token: String<128>, expires_in: u64 },
    AccessToken { access_token: String<756>, expires_in: u64 },
    Error { error: TokenStatus }
}


fn post_form<T, const N : usize>(client: &mut Client<EspHttpConnection>, url: &str, payload: [(&str, &str); N]) -> Result<T, AuthError> where T : DeserializeOwned {

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

pub fn refresh_token(client: &mut Client<EspHttpConnection>, refresh_token: &String<128>) -> Result<TokenResult, AuthError> {
    post_form(client, &format!("https://{}/oauth/token", option_env!("AUTH0_DOMAIN").unwrap_or("dev-plq6-asi.eu.auth0.com")), [("client_id", option_env!("AUTH0_CLIENT_ID").unwrap_or("5nYFEjhKlvTPheFxEDIEo97wLx3auwB7")), ("client_secret", option_env!("AUTH0_CLIENT_SECRET").unwrap_or("zp-7XzX4rP-ihysBSPoF2fXLfQRAxv2WnJEw-dp4f2LEa_rN8T2gU4fU-OqxWg4I")), ("grant_type", "refresh_token"), ("refresh_token", refresh_token.as_str())])
}

pub fn poll_token(client: &mut Client<EspHttpConnection>, device_code: &str) -> Result<TokenResult, AuthError> {
    post_form(client, &format!("https://{}/oauth/token", option_env!("AUTH0_DOMAIN").unwrap_or("dev-plq6-asi.eu.auth0.com")), [("client_id", option_env!("AUTH0_CLIENT_ID").unwrap_or("5nYFEjhKlvTPheFxEDIEo97wLx3auwB7")), ("device_code", device_code), ("grant_type", "urn:ietf:params:oauth:grant-type:device_code")])
}

pub fn request_device_code(client: &mut Client<EspHttpConnection>) -> Result<DeviceCodeResponse, AuthError> {
    post_form(client, &format!("https://{}/oauth/device/code", option_env!("AUTH0_DOMAIN").unwrap_or("dev-plq6-asi.eu.auth0.com")), [("client_id", option_env!("AUTH0_CLIENT_ID").unwrap_or("5nYFEjhKlvTPheFxEDIEo97wLx3auwB7")), ("scope", option_env!("AUTH0_SCOPE").unwrap_or("offline_access")), ("audience", option_env!("AUTH0_AUDIENCE").unwrap_or("https://mycelium.co"))])
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