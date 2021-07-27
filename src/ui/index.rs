use actix_web::{get, HttpResponse};
use serde::Serialize;

use super::*;

#[derive(Serialize)]
struct IndexContext<'a> {
    layout: LayoutContext<'a>,
}

#[get("/")]
async fn index(reader: DataMessagesReader) -> HttpResponse {
    let context = IndexContext {
        layout: layout_context(&reader),
    };
    render_response("index.tera", &context)
}
