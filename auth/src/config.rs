use config;

#[derive(Debug, serde::Deserialize)]
pub struct Settings {
    pub database_url: String,
    pub bind_address: std::net::SocketAddr,
    pub amqp_address: String,
    pub session_timeout: u32,
}

impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
        let mut s = config::Config::new();
        s.merge(config::Environment::with_prefix("auth"))?;
        s.try_into()
    }
}
