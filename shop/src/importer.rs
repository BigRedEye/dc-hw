use crate::config;
use crate::repo;
use prost::Message;
use errors::prelude::*;

pub struct Importer {
    repo: repo::PgRepo,
    rc: amqp_consumer::Receiver,
}

impl Importer {
    pub fn new(repo: repo::PgRepo, config: config::Settings) -> Self {
        Importer {
            repo,
            rc: amqp_consumer::Receiver::new(&config.amqp_address, &config.amqp_queue).unwrap()
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            match self.rc.process(&self.make_handler()).await {
                Ok(_) => (),
                Err(e) => log::error!("Consumer error: {}", e.to_string()),
            }
        }
    }

    fn make_handler(&self) -> ImporterHandler {
        ImporterHandler { repo: self.repo.clone() }
    }
}

struct ImporterHandler {
    repo: repo::PgRepo,
}

impl amqp_consumer::Consumer for ImporterHandler {
    fn consume(&self, delivery: lapin::message::Delivery) -> Result<()> {
        let mut buf = &*delivery.data;
        let batch = pb::ProductsBatch::decode(&mut buf)?;
        log::info!("Start loading batch with {} products", batch.products.len());

        let _ = self.repo.add_products(&batch.products.into_iter().map(|e| e.into()).collect::<Vec<_>>());

        log::info!("Finish loading batch");

        Ok(())
    }
}
