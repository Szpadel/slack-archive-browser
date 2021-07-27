#![deny(clippy::all)]
#![deny(clippy::pedantic)]
// Exceptions from pedantic set:
#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::default_trait_access,
    clippy::redundant_closure_for_method_calls, // this looks less readable
    clippy::filter_map, // sometimes it is more readable to do it in 2 steps
    clippy::wildcard_imports,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless, // disable for now, maybe re-enable later
)]

mod error;
mod reader;
mod ui;
mod simple_cache;

use actix_web::{get, http::StatusCode, web, App, HttpServer};
use error::{Result, WebError};
use lazy_static::lazy_static;
use reader::MessagesReader;
use std::{sync::Arc, path::Path};

lazy_static! {
    pub static ref READER: Arc<MessagesReader> = Arc::new(MessagesReader::new(Path::new("./data").into()));
}

pub type DataMessagesReader = web::Data<Arc<MessagesReader>>;

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

#[get("/test/{chanel}")]
async fn list_dates(reader: DataMessagesReader, path: web::Path<(String,)>) -> Result<String> {
    let channel = path.into_inner().0;
    let dates = {
        reader.list_dates(&channel)
    }
    .map_err(|_| WebError::new(StatusCode::NOT_FOUND, "Channel not found"))?;

    Ok(serde_json::to_string(&dates).unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    setup_logger().unwrap();
    HttpServer::new(|| {
        App::new()
            .data(READER.clone())
            .service(list_dates)
            .service(ui::routes())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
