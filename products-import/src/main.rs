use actix_multipart::Multipart;
use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use futures::TryStreamExt;
use std::io::{Cursor, Read};
use prost::Message;

use tokio_amqp::*;

mod config;

#[derive(serde::Serialize)]
struct Response {
    status: String,
}

struct ImportService {
    config: config::Settings
}

struct IteratorAsRead<I>
where
    I: Iterator,
{
    iter: I,
    cursor: Option<Cursor<I::Item>>,
}

impl<I> IteratorAsRead<I>
where
    I: Iterator,
{
    pub fn new<T>(iter: T) -> Self
    where
        T: IntoIterator<IntoIter = I, Item = I::Item>,
    {
        let mut iter = iter.into_iter();
        let cursor = iter.next().map(Cursor::new);
        IteratorAsRead { iter, cursor }
    }
}

impl<I> Read for IteratorAsRead<I>
where
    I: Iterator,
    Cursor<I::Item>: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        while let Some(ref mut cursor) = self.cursor {
            let read = cursor.read(buf)?;
            if read > 0 {
                return Ok(read);
            }
            self.cursor = self.iter.next().map(Cursor::new);
        }
        Ok(0)
    }
}

struct BatchPublisher {
    address: String,
    queue_name: String,
    batch_size: usize,

    batch: pb::ProductsBatch,
}

impl BatchPublisher {
    fn new(cfg: &config::Settings) -> Self {
        BatchPublisher {
            address: cfg.amqp_address.clone(),
            queue_name: cfg.amqp_queue.clone(),
            batch_size: cfg.batch_size,
            batch: pb::ProductsBatch::default(),
        }
    }

    async fn submit(&mut self, record: Record) {
        self.batch.products.push(pb::Product {
            id: None,
            code: record.uniq_id,
            name: record.product_name,
            category: record.amazon_category_and_sub_category,
        });

        if self.batch.products.len() >= self.batch_size {
            log::info!("Start submit");
            self.send_batch().await.unwrap();
            log::info!("Finish submit");
        }
    }

    async fn finish(&mut self) {
        if self.batch.products.len() > 0 {
            log::info!("Start submit");
            self.send_batch().await.unwrap();
            log::info!("Finish submit");
        }
    }

    async fn send_batch(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Shrug
        let conn = lapin::Connection::connect(
            &self.address,
            lapin::ConnectionProperties::default().with_tokio(),
        )
        .await?;

        let channel = conn
            .create_channel()
            .await?;

        channel
            .queue_declare(
                &self.queue_name,
                lapin::options::QueueDeclareOptions::default(),
                lapin::types::FieldTable::default(),
            )
            .await?;

        let mut buf = Vec::with_capacity(self.batch.encoded_len());
        self.batch.encode(&mut buf).unwrap();
        self.batch.products.clear();

        channel
            .basic_publish(
                "",
                &self.queue_name,
                lapin::options::BasicPublishOptions::default(),
                buf,
                lapin::BasicProperties::default(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Debug, serde::Deserialize)]
struct Record {
    uniq_id: String,
    product_name: String,
    amazon_category_and_sub_category: String,
}

impl ImportService {
    fn new(config: config::Settings) -> Self {
        ImportService { config }
    }

    async fn import(&self, mut payload: Multipart) -> Result<HttpResponse, Error> {
        while let Ok(Some(field)) = payload.try_next().await {
            let content_type = field.content_disposition().unwrap();
            let filename = content_type.get_name().unwrap();
            if filename != "products" {
                log::error!("Invalid form data: {}", filename);
                return Ok(HttpResponse::BadRequest().json(Response{ status: "Invalid form data".into() }))
            }

            let stream = futures::executor::block_on_stream(field).map(|e| e.unwrap());
            let raw_reader = IteratorAsRead::new(stream);
            let mut reader = csv::Reader::from_reader(raw_reader);
            let mut batch = self.make_submitter();

            for result in reader.deserialize::<Record>() {
                match result {
                    Ok(record) => batch.submit(record).await,
                    Err(_) => (),
                }
            }

            batch.finish().await;
        }
        Ok(HttpResponse::Ok().into())
    }

    fn make_submitter(&self) -> BatchPublisher {
        BatchPublisher::new(&self.config)
    }
}

async fn process_file(svc: web::Data<ImportService>, payload: Multipart) -> Result<HttpResponse, Error> {
    svc.import(payload).await
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let config = config::Settings::new().unwrap();
    let bind_address = config.bind_address;

    HttpServer::new(move || {
        App::new()
            .data(ImportService::new(config.clone()))
            .wrap(middleware::Logger::default())
            .service(web::resource("/v1/import").route(web::post().to(process_file)))
    })
    .bind(bind_address)?
    .run()
    .await
}
