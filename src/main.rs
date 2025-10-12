use crate::config::Config;
use actix_files as af;
use actix_multipart::form::MultipartFormConfig;
use actix_web::{
    App, HttpServer, middleware,
    web::{ServiceConfig, scope},
};
pub use models::{AppErr, ErrorCode};

mod api;
mod config;
mod docs;
mod logger;
mod models;
mod utils;

fn config_app(app: &mut ServiceConfig) {
    if cfg!(debug_assertions) {
        app.service(af::Files::new("/static", "static"));
        // app.service(af::Files::new("/xp-assets", "xp/dist/xp-assets"));
        // app.service(af::Files::new("/record", Config::RECORD_DIR));
    }

    app.service(docs::openapi_json).service(docs::rapidoc);
    app.service(scope("/api").service(api::abzar::router()));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    logger::setup();
    Config::get();

    // let cpt = SqliteConnectOptions::from_str("sqlite://main.db")
    //     .expect("could not init sqlite connection options")
    //     .journal_mode(SqliteJournalMode::Off);
    //
    // let pool = SqlitePool::connect_with(cpt).await.expect("sqlite connection");

    // let app_state = Data::new(AppState::new().expect("app state"));

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::new("%s %r %Ts"))
            .app_data(
                MultipartFormConfig::default()
                    .total_limit(200 * 1024 * 1024)
                    .memory_limit(200 * 1024 * 1024)
                    .error_handler(|e, _rq| {
                        log::error!("mpf: {e:#?}");
                        err!(r, FileTooBig, e.to_string()).into()
                    }),
            )
            // .wrap(
            //     actix_cors::Cors::default()
            //         .allowed_origin("https://sky.gooje.app")
            //         .allowed_origin("https://gooje.app")
            //         .allowed_origin("http://localhost:8008")
            //         .allowed_methods(["GET", "POST"]),
            // )
            // .app_data(app_state.clone())
            .configure(config_app)
    });

    // server.bind(("127.0.0.1", 7023)).unwrap().run().await

    let server = if cfg!(debug_assertions) {
        server.bind(("127.0.0.1", 7023)).unwrap()
    } else {
        use std::os::unix::fs::PermissionsExt;
        const PATH: &str = "/usr/share/nginx/socks/iris.sock";
        let server = server.bind_uds(PATH).unwrap();
        std::fs::set_permissions(PATH, std::fs::Permissions::from_mode(0o777))?;
        server
    };

    server.run().await
}
