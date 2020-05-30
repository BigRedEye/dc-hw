use errors::prelude::*;
use log::debug;

use tokio_amqp::*;

#[derive(Clone)]
pub struct Receiver {
    address: String,
    queue_name: String,
    next_sleep_time: std::time::Duration,
}

pub trait Consumer {
    fn consume(&self, c: lapin::message::Delivery) -> Result<()>;
}

impl Receiver {
    pub fn new(address: &str, queue_name: &str) -> Result<Self> {
        Ok(Receiver {
            address: address.into(),
            queue_name: queue_name.into(),
            next_sleep_time: std::time::Duration::new(0, 0),
        })
    }

    pub async fn process<T: Consumer>(&mut self, cb: &T) -> Result<()> {
        tokio::time::delay_for(self.next_sleep_time).await;

        let res = self.process_impl(cb).await;

        // Incresase backoff time only for ampq errors
        match &res {
            Err(errors::Error::AmqpError(_)) => self.next_sleep_time = self.select_next_timeout(),
            _ => (),
        }

        res
    }

    async fn process_impl<T: Consumer>(&self, cb: &T) -> Result<()> {
        let conn = lapin::Connection::connect(
            self.address.as_str(),
            lapin::ConnectionProperties::default().with_tokio(),
        )
        .await?;
        let channel = conn.create_channel().await?;

        let _ = channel
            .queue_declare(
                self.queue_name.as_str(),
                lapin::options::QueueDeclareOptions::default(),
                lapin::types::FieldTable::default(),
            )
            .await?;

        let consumer = channel
            .basic_consume(
                self.queue_name.as_str(),
                "consumer",
                lapin::options::BasicConsumeOptions::default(),
                lapin::types::FieldTable::default(),
            )
            .await?;

        for delivery in consumer {
            debug!("Received message: {:?}", delivery);
            if let Ok((channel, delivery)) = delivery {
                let tag = delivery.delivery_tag;
                cb.consume(delivery)?;
                channel
                    .basic_ack(tag, lapin::options::BasicAckOptions::default())
                    .await?;
            }
        }

        Ok(())
    }

    fn select_next_timeout(&self) -> std::time::Duration {
        if self.next_sleep_time.as_nanos() == 0u128 {
            return std::time::Duration::from_millis(125);
        } else {
            return self.next_sleep_time * 2;
        }
    }
}
