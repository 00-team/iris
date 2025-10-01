use crate::models::Horp;
use crate::{config::Config, docs::UpdatePaths};

use actix_web::{HttpResponse, Scope, post, web::Json};

#[derive(utoipa::OpenApi)]
#[openapi(
    tags((name = "api::abzar")),
    paths(r_send),
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
    }

    let bd = SendMessageBody {
        chat_id: &ch.chat,
        message_thread_id: ch.thread.as_ref().map(|v| v.as_str()),
        text: body.text.clone(),
    };

    let r = conf.tc.post(url).json(&bd).send().await?;
    if r.status() != 200 {
        log::error!("[tel_err]: {:#?}", r.text().await);
        return crate::err!(SendFailed, "sending message to telegram failed");
    }

    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
struct AbzarSendFileBody {
    channel: String,
    pass: String,
    text: String,
}

#[utoipa::path(
    post,
    request_body = AbzarSendFileBody,
    responses((status = 200))
)]
/// Send File
#[post("/send-file/")]
async fn r_send_file(body: Json<AbzarSendBody>) -> Horp {
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
    }

    let bd = SendMessageBody {
        chat_id: &ch.chat,
        message_thread_id: ch.thread.as_ref().map(|v| v.as_str()),
        text: body.text.clone(),
    };

    let r = conf.tc.post(url).json(&bd).send().await?;
    if r.status() != 200 {
        log::error!("[tel_err]: {:#?}", r.text().await);
        return crate::err!(SendFailed, "sending message to telegram failed");
    }

    Ok(HttpResponse::Ok().finish())
}

pub fn router() -> Scope {
    Scope::new("/abzar").service(r_send).service(r_send_file)
}
