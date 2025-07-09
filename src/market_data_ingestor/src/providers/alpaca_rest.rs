use reqwest::Client;
use secrecy::SecretString;

pub struct AlpacaProvider {
    client: Client,
    _api_key: SecretString,
    _secret_key: SecretString,
}

