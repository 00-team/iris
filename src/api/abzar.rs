use crate::models::Horp;
use crate::{config::Config, docs::UpdatePaths};

use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::{MultipartForm, text::Text};
use actix_web::{HttpResponse, Scope, post, web::Json};

#[derive(utoipa::OpenApi)]
#[openapi(
    tags((name = "api::abzar")),
    paths(r_send, r_send_file),
    servers((url = "/abzar")),
    modifiers(&UpdatePaths)
)]
pub struct ApiDoc;

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
struct AbzarSendBody {
    channel: String,
    pass: String,
    text: String,
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

    #[derive(serde::Serialize)]
    struct SendMessageBody<'a> {
        chat_id: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        message_thread_id: Option<&'a str>,
        text: String,
        parse_mode: &'static str,
    }

    let bd = SendMessageBody {
        chat_id: &ch.chat,
        message_thread_id: ch.thread.as_ref().map(|v| v.as_str()),
        text: body.text.clone(),
        parse_mode: "MarkdownV2",
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
    // #[schema(value_type = AbzarSendBody)]
    // x: MpJson<AbzarSendBody>,
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
        .text("caption", form.text.0.clone())
        .text("parse_mode", "MarkdownV2");

    if let Some(tid) = &ch.thread {
        sf = sf.text("message_thread_id", tid.clone());
    }

    let r = conf.tc.post(url).multipart(sf).send().await?;
    if r.status() != 200 {
        log::error!("[tel_err]: {:#?}", r.text().await);
        return crate::err!(SendFailed, "sending file to telegram failed");
    }

    Ok(HttpResponse::Ok().finish())
}

pub fn router() -> Scope {
    Scope::new("/abzar").service(r_send).service(r_send_file)
}
