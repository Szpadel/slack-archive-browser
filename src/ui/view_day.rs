use crate::{error::WebError};
use actix_web::{get, HttpResponse};
use chrono::{NaiveDate};
use serde::Serialize;
use super::*;

#[derive(Serialize)]
struct ViewDayContext<'a> {
    layout: LayoutContext<'a>,
    messages: String,
}

#[derive(Serialize)]
struct MessagesContext<'a> {
    messages: Vec<Message<'a>>,
}

#[derive(Serialize)]
struct Message<'a> {
    time: String,
    user_id: &'a String,
    text: &'a String,
}

fn render_messages(
    reader: &MessagesReader,
    channel_id: &str,
    date: &NaiveDate,
) -> Result<String, WebError> {
    Ok(TPL
        .render(
            "messages.tera",
            &Context::from_serialize(&MessagesContext {
                messages: reader
                    .channel_messages_parse(channel_id, date)?
                    .messages
                    .iter()
                    .map(|msg| Message {
                        time: msg.timestamp.format("%H:%M:%S").to_string(),
                        user_id: &msg.user_id,
                        text: &msg.text,
                    })
                    .collect(),
            })
            .unwrap(),
        )
        .unwrap())
}

#[get("/{channel}/{date}")]
async fn view_day(
    reader: DataMessagesReader,
    parts: web::Path<(String, NaiveDate)>,
) -> HttpResponse {
    let parts = parts.into_inner();
    let channel_id = parts.0;
    let date = parts.1;

    let context = render_messages(&reader, &channel_id, &date).map(|messages| ViewDayContext {
        messages,
        layout: layout_context(&reader),
    });

    match context {
        Ok(context) => {
            render_response("view_day.tera", &context)
        }
        Err(err) => {
            log::info!("Error rendering {} for day: {}: {:#}", channel_id, date, err);
            render_page_not_found()
        }
    }
}
