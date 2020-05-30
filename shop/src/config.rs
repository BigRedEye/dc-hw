use config;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Settings {
    pub database_url: String,
    pub bind_address: std::net::SocketAddr,
    pub auth_address: String,
    pub amqp_address: String,
    pub amqp_queue: String,
}

impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
        let mut s = config::Config::new();
        s.merge(config::Environment::with_prefix("shop"))?;
        s.try_into()
    }
}
