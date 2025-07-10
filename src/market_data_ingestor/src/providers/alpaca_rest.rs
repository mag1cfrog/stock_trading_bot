use reqwest::{header, Client};
use secrecy::{ExposeSecret, SecretString};
use shared_utils::env::get_env_var;

use crate::providers::ProviderInitError;


pub struct AlpacaProvider {
    client: Client,
    _api_key: SecretString,
    _secret_key: SecretString,
}

impl AlpacaProvider {
    /// Creates a new Alpaca provider.
    ///
    /// Reads API keys from the `APCA_API_KEY_ID` and `APCA_API_SECRET_KEY`
    /// environment variables.
    pub fn new() -> Result<Self, ProviderInitError> {
        let api_key = SecretString::new(get_env_var("APCA_API_KEY_ID")?.into());
        let secret_key = SecretString::new(get_env_var("APCA_API_SECRET_KEY")?.into());

        let mut headers = header::HeaderMap::new();
        headers.insert(
            "APCA-API-KEY-ID",
            header::HeaderValue::from_str(api_key.expose_secret()).unwrap(),
        );
        headers.insert(
            "APCA_API_SECRET_KEY",
            header::HeaderValue::from_str(secret_key.expose_secret()).unwrap(),
        );

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            client,
            _api_key: api_key,
            _secret_key: secret_key,
        })
    }
}