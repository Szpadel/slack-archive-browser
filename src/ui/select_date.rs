use actix_web::{get, HttpResponse};
use chrono::NaiveDate;
use serde::Serialize;

use crate::error::WebError;

use super::*;

#[derive(Serialize)]
struct SelectDateContext<'a> {
    dates_available: Vec<NaiveDate>,
    channel_id: &'a str,
}

#[get("/{channel}")]
async fn select_date(reader: DataMessagesReader, parts: web::Path<(String,)>) -> HttpResponse {
    let channel_id = parts.into_inner().0;
    let context: Result<_, WebError> = (|| {
        Ok(SelectDateContext {
            dates_available: reader.list_dates(&channel_id)?,
            channel_id: &channel_id,
        })
    })();

    if let Ok(context) = context {
        render_response("select_date.tera", &context)
    } else {
        render_page_not_found()
    }
}
