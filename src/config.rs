// use std::path::{Path, PathBuf};
use std::{collections::HashMap, str::FromStr, sync::OnceLock};

mod config_toml {
    use std::{collections::HashMap, path::PathBuf};

    #[derive(Debug, serde::Deserialize)]
    pub struct Channel {
        pub chat: String,
        pub thread: Option<String>,
        pub pass: String,
    }

    #[derive(Debug, serde::Deserialize)]
    pub struct ConfigToml {
        pub tel_token: String,
        pub channels: HashMap<String, Channel>,
    }

    fn path() -> PathBuf {
        #[cfg(debug_assertions)]
        const DEFAULT: &str = "config.local.toml";

        #[cfg(not(debug_assertions))]
        const DEFAULT: &str = "config.toml";

        let mut args = std::env::args();
        let path = loop {
            let Some(arg) = args.next() else { break None };
            if arg == "-c" || arg == "--config" {
                break args.next();
            }
        }
        .unwrap_or(String::from(DEFAULT));

        PathBuf::from(path)
    }

    pub fn get() -> ConfigToml {
        let path = path();
        log::info!("reading config at: {path:?}");
        let data = match std::fs::read_to_string(&path) {
            Ok(v) => v,
            Err(e) => panic!("could not read config at: {path:?}\n{e:#?}"),
        };

        match toml::from_str(&data) {
            Ok(v) => v,
            Err(e) => panic!("invalid toml config file: {path:?}\n{e:#?}"),
        }
    }
}

#[derive(Debug)]
/// `Iris` Config
pub struct Config {
    pub tc: reqwest::Client,
    pub tb: String,
    pub channels: HashMap<String, config_toml::Channel>,
    pub send_message: reqwest::Url,
    pub send_document: reqwest::Url,
}

impl Config {
    // pub const RMBGU: &str = "https://api.remove.bg/v1.0/removebg";
    // pub const TOKEN_LIFE: i64 = 30 * 24 * 3600;
    pub const API_VERSION: &str = "0.1.0";
    // pub const RECORD_DIR: &str = "record";

    // pub const HTML_HEAD: &str = "./app/html/head.html";
    // pub const HTML_SCRIPTS: &str = "./app/dist/html/clean.html";

    // pub const CODE_ABC: &[u8] = b"0123456789";
    // pub const TOKEN_ABC: &[u8] = b"!@#$%^&*_+abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*_+";
    // pub const SALT_ABC: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    // pub const USERNAME_ABC: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_";

    fn create_dirs() -> std::io::Result<()> {
        // let path = Path::new(Self::RECORD_DIR);
        // std::fs::create_dir_all(path)?;

        Ok(())
    }

    // pub fn record(kind: &str, id: i64, salt: &str) -> PathBuf {
    //     Path::new(Self::RECORD_DIR).join(format!("{kind}-{id}-{salt}.webp"))
    // }

    fn tc_client() -> reqwest::Client {
        // use reqwest::header as hh;
        // let mut hm = hh::HeaderMap::new();
        // let av = hh::HeaderValue::from_str(token).expect("invalid hc token");
        // hm.insert("X-Api-Key", av);

        reqwest::ClientBuilder::new()
            // .default_headers(hm)
            .timeout(std::time::Duration::from_secs(500))
            .connection_verbose(false)
            .build()
            .expect("could not build telegram client")
    }

    fn init() -> Self {
        let ct = config_toml::get();

        Self::create_dirs().expect("failed to create required directories");

        Self {
            tc: Self::tc_client(),
            channels: ct.channels,
            send_message: reqwest::Url::from_str(&format!(
                "https://api.telegram.org/bot{}/sendMessage",
                ct.tel_token
            ))
            .expect("url err"),
            send_document: reqwest::Url::from_str(&format!(
                "https://api.telegram.org/bot{}/sendDocument",
                ct.tel_token
            ))
            .expect("url err"),
            tb: ct.tel_token,
        }
    }

    pub fn get() -> &'static Self {
        static STATE: OnceLock<Config> = OnceLock::new();
        STATE.get_or_init(Self::init)
    }
}
