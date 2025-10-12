use crate::models::Horp;
use crate::{config::Config, docs::UpdatePaths};

use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::{MultipartForm, text::Text};
use actix_web::{HttpResponse, Scope, post, web::Json};

#[derive(utoipa::OpenApi)]
#[openapi(
    tags((name = "api::abzar")),
    paths(r_send, r_send_file, r_send_mp),
    servers((url = "/abzar")),
    modifiers(&UpdatePaths)
)]
pub struct ApiDoc;

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, Clone, Copy)]
enum ParseMode {
    Markdown,
    MarkdownV2,
    Html,
}

impl ParseMode {
    fn as_str(&self) -> &'static str {
        match self {
            Self::MarkdownV2 => "MarkdownV2",
            Self::Markdown => "Markdown",
            Self::Html => "HTML",
        }
    }
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
struct AbzarSendBody {
    channel: String,
    pass: String,
    text: String,
    parse_mode: Option<ParseMode>,
}

#[derive(serde::Serialize)]
struct LinkPreviewOptions {
    is_disabled: bool,
    prefer_small_media: bool,
}

impl Default for LinkPreviewOptions {
    fn default() -> Self {
        Self { is_disabled: false, prefer_small_media: true }
    }
}

#[derive(serde::Serialize)]
struct SendMessageBody<'a> {
    chat_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    message_thread_id: Option<&'a str>,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parse_mode: Option<&'static str>,
    link_preview_options: LinkPreviewOptions,
}

#[utoipa::path(
    post,
    request_body = AbzarSendBody,
    responses((status = 200))
)]
/// Send
#[post("/send/")]
async fn r_send(body: Json<AbzarSendBody>) -> Horp {
    let conf = Config::get();
    let Some(ch) = conf.channels.get(&body.channel) else {
        return crate::err!(NotFound, "no channel");
    };

    if ch.pass != body.pass {
        return crate::err!(NotFound, "no channel");
    }

    let url = conf.send_message.clone();

    let bd = SendMessageBody {
        chat_id: &ch.chat,
        message_thread_id: ch.thread.as_ref().map(|v| v.as_str()),
        text: body.text.clone(),
        parse_mode: body.parse_mode.map(|v| v.as_str()),
        link_preview_options: Default::default(),
    };

    let r = conf.tc.post(url).json(&bd).send().await?;
    if r.status() != 200 {
        log::error!("[tel_err]: {:#?}", r.text().await);
        return crate::err!(SendFailed, "sending message to telegram failed");
    }

    Ok(HttpResponse::Ok().finish())
}

// #[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
// struct AbzarSendFileBody {
//     channel: String,
//     pass: String,
//     text: String,
// }

#[derive(Debug, MultipartForm, utoipa::ToSchema)]
pub struct AbzarSendFileBody {
    #[schema(value_type = String, format = Binary)]
    #[multipart(limit = "50MB")]
    file: TempFile,
    #[schema(value_type = String)]
    channel: Text<String>,
    #[schema(value_type = String)]
    pass: Text<String>,
    #[schema(value_type = String)]
    text: Text<String>,
    #[schema(value_type = Option<ParseMode>)]
    parse_mode: Option<Text<ParseMode>>,
}

#[utoipa::path(
    post,
    request_body(
        content = AbzarSendFileBody,
        content_type = "multipart/form-data"
    ),
    responses((status = 200))
)]
/// Send File
#[post("/send-file/")]
async fn r_send_file(form: MultipartForm<AbzarSendFileBody>) -> Horp {
    if form.file.size >= 50_000_000 {
        return crate::err!(FileTooBig, "max file size is 50MB");
    }

    let conf = Config::get();
    let Some(ch) = conf.channels.get(&form.channel.0) else {
        return crate::err!(NotFound, "no channel");
    };

    if ch.pass != form.pass.0 {
        return crate::err!(NotFound, "no channel");
    }

    let url = conf.send_document.clone();

    let mut doc = reqwest::multipart::Part::file(form.file.file.path()).await?;
    if let Some(fname) = form.file.file_name.clone() {
        doc = doc.file_name(fname);
    }
    if let Some(mime) = &form.file.content_type {
        doc = doc.mime_str(&mime.to_string())?;
    }

    let mut sf = reqwest::multipart::Form::new()
        .part("document", doc)
        .text("chat_id", ch.chat.clone())
        .text("caption", form.text.0.clone());

    if let Some(pm) = &form.parse_mode {
        sf = sf.text("parse_mode", pm.as_str());
    }

    if let Some(tid) = &ch.thread {
        sf = sf.text("message_thread_id", tid.clone());
    }

    async fn send_gg(
        conf: &Config, url: reqwest::Url, sf: reqwest::multipart::Form,
    ) {
        let Ok(r) = conf.tc.post(url).multipart(sf).send().await else {
            return;
        };
        if r.status() != 200 {
            log::error!("[tel_err]: {:#?}", r.text().await);
            // return crate::err!(SendFailed, "sending file to telegram failed");
        }
    }

    tokio::task::spawn(send_gg(conf, url, sf));

    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, MultipartForm, utoipa::ToSchema)]
pub struct AbzarSendMpBody {
    #[schema(value_type = String)]
    channel: Text<String>,
    #[schema(value_type = String)]
    pass: Text<String>,
    #[schema(value_type = String)]
    text: Text<String>,
    #[schema(value_type = Option<ParseMode>)]
    parse_mode: Option<Text<ParseMode>>,
}

#[utoipa::path(
    post,
    request_body(
        content = AbzarSendMpBody,
        content_type = "multipart/form-data"
    ),
    responses((status = 200))
)]
/// Send Message Multipart
#[post("/send-mp/")]
async fn r_send_mp(form: MultipartForm<AbzarSendMpBody>) -> Horp {
    // if form.file.size >= 50_000_000 {
    //     return crate::err!(FileTooBig, "max file size is 50MB");
    // }

    let conf = Config::get();
    let Some(ch) = conf.channels.get(&form.channel.0) else {
        return crate::err!(NotFound, "no channel");
    };

    if ch.pass != form.pass.0 {
        return crate::err!(NotFound, "no channel");
    }

    let url = conf.send_message.clone();

    let bd = SendMessageBody {
        chat_id: &ch.chat,
        message_thread_id: ch.thread.as_ref().map(|v| v.as_str()),
        text: form.text.clone(),
        parse_mode: form.parse_mode.as_ref().map(|v| v.as_str()),
        link_preview_options: Default::default(),
    };

    let r = conf.tc.post(url).json(&bd).send().await?;
    if r.status() != 200 {
        log::error!("[tel_err mp]: {:#?}", r.text().await);
        return crate::err!(SendFailed, "sending message to telegram failed");
    }

    Ok(HttpResponse::Ok().finish())
}

pub fn router() -> Scope {
    Scope::new("/abzar").service(r_send).service(r_send_file).service(r_send_mp)
}
