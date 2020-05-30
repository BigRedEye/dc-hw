mod config;

use prost::Message;

use errors::prelude::*;

struct SmsSender {
    api_token: String,
}

impl amqp_consumer::Consumer for SmsSender {
    fn consume(&self, delivery: lapin::message::Delivery) -> Result<()> {
        let mut buf = &*delivery.data;
        let c = pb::Confirmation::decode(&mut buf)?;

        let text = format!("Visit {} to confirm your phone", c.url);
        // let query = format!("https://sms.ru/sms/send?api_id={}&to={}")

        let encoded: String = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("to", c.login)
            .finish();

        let _ = reqwest::get().await?.text().await?;
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
