use config;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Settings {
    pub amqp_address: String,
    pub auth_address: String,
    pub amqp_queue: String,
    pub bind_address: std::net::SocketAddr,
    // pub tmp_storage: std::path::PathBuf,
    pub batch_size: usize,
}

impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
        let mut s = config::Config::new();
        s.merge(config::Environment::with_prefix("import"))?;
        s.try_into()
    }
}
