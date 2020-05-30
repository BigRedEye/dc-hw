use config;

#[derive(Debug, serde::Deserialize)]
pub struct Settings {
    pub amqp_address: String,
    pub amqp_queue: String,
    pub smtp_server: String,
}

impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
        let mut s = config::Config::new();
        s.merge(config::Environment::with_prefix("email"))?;
        s.try_into()
    }
}
