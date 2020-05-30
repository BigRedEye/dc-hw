mod config;

use lettre::{SmtpClient, Transport};
use lettre_email::EmailBuilder;
use prost::Message;

use errors::prelude::*;

struct EmailSender {
    smtp_server: String,
}

impl amqp_consumer::Consumer for EmailSender {
    fn consume(&self, delivery: lapin::message::Delivery) -> Result<()> {
        let mut buf = &*delivery.data;
        let c = pb::Confirmation::decode(&mut buf)?;

        let text = format!("Visit {} to confirm your email", c.url);

        let email = EmailBuilder::new()
            .to(c.login.clone())
            .from("noreply@sskvor.dev")
            .subject("Confirm your email address")
            .text(text)
            .build()
            .map_err(|e| {
                log::error!("Failed to parse email: {}", e.to_string());
            });

        let email = match email {
            Ok(email) => email,
            Err(()) => return Ok(()),
        };

        let mut mailer = SmtpClient::new_simple(&self.smtp_server).unwrap().transport();

        let result = mailer.send(email.into());

        if result.is_ok() {
            log::info!("Sucessfully sent email with url {} to address {}", c.url, c.login);
        } else {
            log::error!("Could not send email: {:?}", result);
        }

        result.map_err(|e| errors::Error::Internal(e.into())).map(|_| ())
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    env_logger::init();

    let config = config::Settings::new()?;

    log::info!("Opening amqp consumer");
    let mut receiver = amqp_consumer::Receiver::new(&config.amqp_address, &config.amqp_queue)?;
    let email = EmailSender{ smtp_server: config.smtp_server };

    log::info!("Starting main loop");
    loop {
        match receiver.process(&email).await {
            Ok(_) => (),
            Err(e) => log::error!("Consumer error: {}", e.to_string()),
        }
    }
}
