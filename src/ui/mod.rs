mod index;
mod select_date;
mod view_day;

use super::DataMessagesReader;
use crate::{
    reader::{ChannelInfo, MessagesReader},
    simple_cache::{OptimisticLockCache, TimeCache},
};
use crate::{simple_cache::OptimisticLRU, READER};
use actix_web::{
    get,
    web::{self},
    HttpRequest, HttpResponse, Scope,
};
use chrono::NaiveDate;
use include_dir::{include_dir, Dir};
use lazy_static::lazy_static;
use serde::Serialize;
use std::collections::HashMap;
use tera::{Context, Error as TeraError, Result as TeraResult, Tera, Value};

static TEMPLATES: Dir = include_dir!("templates");
static STATIC: Dir = include_dir!("static");

lazy_static! {
    static ref USERNAME_CACHE: OptimisticLRU<String, String> = OptimisticLRU::new(256);
    static ref CHANNELS_LIST_CACHE: OptimisticLRU<Option<NaiveDate>, String> =
        OptimisticLRU::new(256);
    pub static ref TPL: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_templates(
            TEMPLATES
                .files()
                .iter()
                .map(|file| (file.path, file.contents_utf8().unwrap())),
        )
        .unwrap();
        tera.autoescape_on(vec![".tera"]);
        tera.register_function("render_username", RenderUsername);
        tera.register_function("render_channels", RenderChannels);
        tera
    };
}

#[derive(Serialize)]
pub struct ChannelsContext<'a> {
    channels: Vec<&'a ChannelInfo>,
    date: &'a Option<NaiveDate>,
}

struct RenderChannels;

impl tera::Function for RenderChannels {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let date = args
            .get("date")
            .map(|v| serde_json::from_value(v.to_owned()).unwrap());
        Ok(serde_json::to_value(
            CHANNELS_LIST_CACHE
                .get_or_update(date, |date| {
                    let context = Context::from_serialize(ChannelsContext {
                        channels: READER.list_channels(),
                        date,
                    })
                    .unwrap();
                    TPL.render("channels.tera", &context).unwrap()
                })
                .as_ref(),
        )
        .unwrap())
    }

    fn is_safe(&self) -> bool {
        true
    }
}
struct RenderUsername;


impl tera::Function for RenderUsername {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let user_id = match args.get("user_id") {
            Some(id) => match serde_json::from_value::<String>(id.clone()) {
                Ok(id) => id,
                Err(_) => {
                    return Err(TeraError::msg(format!(
                        "Function `render_username` `need user_id` passed as string"
                    )))
                }
            },
            None => {
                return Err(TeraError::msg(
                    "Function `render_username require parameter `user_id``",
                ))
            }
        };

        Ok(serde_json::to_value(
            USERNAME_CACHE
                .get_or_update(user_id, |user_id| {
                    if let Ok(user_info) = READER.get_user_info(user_id) {
                        TPL.render("username.tera", &Context::from_serialize(user_info).unwrap())
                            .unwrap()
                    } else {
                        "Unknown".to_string()
                    }
                })
                .as_ref(),
        )
        .unwrap())
    }

    fn is_safe(&self) -> bool {
        true
    }
}

#[derive(Serialize)]
pub struct LayoutContext<'a> {
    channels: Vec<&'a ChannelInfo>,
}

pub fn layout_context<'a>(reader: &'a MessagesReader) -> LayoutContext<'a> {
    LayoutContext {
        channels: reader.list_channels(),
    }
}

fn content_type_for_ext(ext: &str) -> &str {
    match ext {
        "css" => "text/css",
        _ => "application/octet-stream",
    }
}

#[get("/static/{file:.*}")]
async fn serve_static(req: HttpRequest) -> HttpResponse {
    let file_name = req.match_info().query("file");
    let file = STATIC.get_file(file_name);
    if let Some(file) = file {
        HttpResponse::Ok()
            .content_type(content_type_for_ext(
                &file
                    .path()
                    .extension()
                    .unwrap_or_default()
                    .to_string_lossy(),
            ))
            .body(file.contents())
    } else {
        HttpResponse::NotFound().body("File not found")
    }
}

pub fn render_page_not_found() -> HttpResponse {
    HttpResponse::NotFound().body("Page not found")
}

fn render_response<C>(template_name: &str, data: &C) -> HttpResponse
where
    C: Serialize,
{
    let context = Context::from_serialize(data).unwrap();
    let result = TPL.render(template_name, &context);
    match result {
        Ok(data) => HttpResponse::Ok().content_type("text/html").body(data),
        Err(err) => {
            log::error!("Failed to render page: {:#}", err);
            HttpResponse::InternalServerError().body("Failed to render the page")
        }
    }
}

pub fn routes() -> Scope {
    web::scope("")
        .service(serve_static)
        .service(index::index)
        .service(select_date::select_date)
        .service(view_day::view_day)
}
