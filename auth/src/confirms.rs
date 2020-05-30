use crate::config;
use crate::models;
use crate::service;
use async_trait::async_trait;
use errors::prelude::*;
use prost::Message;

use tokio_amqp::*;

#[derive(Clone)]
pub struct ConfrimsSender {
    channel: lapin::Channel,
}

#[async_trait]
impl service::ConfirmationsSender for ConfrimsSender {
    async fn send(&self, login: models::Login, token: String) -> Result<()> {
        println!("Token: {}", token);

        let (login, queue) = match login {
            models::Login::Email(email) => (email, "confirmations_email"),
            models::Login::Phone(phone) => (phone, "confirmations_phone"),
        };

        // FIXME(sskvor)
        let url = format!("https://hw.sskvor.dev/v1/confirm?token={}", token);
        let c = pb::Confirmation { login, url };

        let mut buf = Vec::with_capacity(c.encoded_len());
        c.encode(&mut buf).unwrap();

        self.channel
            .basic_publish(
                "",
                queue,
                lapin::options::BasicPublishOptions::default(),
                buf,
                lapin::BasicProperties::default(),
            )
            .await?;

        Ok(())
    }
}

impl ConfrimsSender {
    pub async fn new(cfg: &config::Settings) -> Result<ConfrimsSender> {
        let conn = lapin::Connection::connect(
            &cfg.amqp_address,
            lapin::ConnectionProperties::default().with_tokio(),
        )
        .await?;
        let channel = conn.create_channel().await?;

        for queue in &["confirmations_email", "confirmations_phone"] {
            channel
                .queue_declare(
                    queue,
                    lapin::options::QueueDeclareOptions::default(),
                    lapin::types::FieldTable::default(),
                )
                .await?;
        }

        Ok(ConfrimsSender { channel })
    }
}
