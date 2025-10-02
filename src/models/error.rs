use actix_web::{HttpResponse, ResponseError, body::BoxBody, http::StatusCode};
use serde::Serialize;
use tokio::task::JoinError;
use utoipa::ToSchema;

#[derive(Debug, Default, Serialize, ToSchema, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    #[default]
    Unknown = 0,
    // BadRequest,
    Forbidden,
    ForbiddenSelfEdit,
    BadAuth,
    NotFound,
    NotUnique,
    ServerError,
    DatabaseError,
    RateLimited,
    IndexOutOfBounds,

    SendFailed,
    FileTooBig,
}

impl ErrorCode {
    fn status(&self) -> u16 {
        match self {
            Self::NotUnique => 400,
            Self::FileTooBig => 400,

            Self::IndexOutOfBounds => 400,

            Self::NotFound => 404,

            Self::Forbidden | Self::BadAuth | Self::ForbiddenSelfEdit => 403,

            Self::RateLimited => 429,

            Self::SendFailed => 500,
            Self::Unknown => 500,
            Self::ServerError | Self::DatabaseError => 500,
        }
    }
}

#[derive(Serialize, Debug, ToSchema, Clone)]
#[schema(title = "AppErr")]
pub struct AppErr {
    status: u16,
    code: ErrorCode,
    debug: Option<String>,
}

impl AppErr {
    // pub fn new(status: u16, code: ErrorCode) -> Self {
    //     Self { status, code, debug: None }
    // }

    pub fn server_error() -> Self {
        Self { status: 500, code: ErrorCode::ServerError, debug: None }
    }

    pub fn debug(mut self, debug: &str) -> Self {
        self.debug = Some(debug.to_string());
        self
    }

    // pub fn code<C: Into<u16>>(mut self, code: C) -> Self {
    //     self.code = code.into();
    //     self
    // }
}

impl std::error::Error for AppErr {}

impl std::fmt::Display for AppErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// impl From<sqlx::Error> for AppErr {
//     fn from(value: sqlx::Error) -> Self {
//         log::error!("sqlx error: {value:?}");
//         match value {
//             sqlx::Error::RowNotFound => ErrorCode::NotFound,
//             sqlx::Error::Database(e) => match e.code() {
//                 Some(c) if c == "2067" => ErrorCode::NotUnique,
//                 Some(c) if c == "787" => ErrorCode::NotFound,
//                 _ => ErrorCode::DatabaseError,
//             },
//             _ => ErrorCode::DatabaseError,
//         }
//         .into()
//     }
// }

impl From<ErrorCode> for AppErr {
    fn from(value: ErrorCode) -> Self {
        Self { status: value.status(), debug: None, code: value }
    }
}

// impl From<image::ImageError> for AppErr {
//     fn from(value: image::ImageError) -> Self {
//         Self::from(ErrorCode::BadImage).debug(&value.to_string())
//     }
// }

impl From<JoinError> for AppErr {
    fn from(value: JoinError) -> Self {
        Self::from(ErrorCode::ServerError)
            .debug(&format!("failed to join tokio task: {value:#?}"))
    }
}

impl ResponseError for AppErr {
    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.status)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code()).json(self)
    }
}

macro_rules! fse {
    ($ty:path) => {
        impl From<$ty> for AppErr {
            fn from(value: $ty) -> Self {
                log::error!("_500_: [{}] {value:?}", stringify!($ty));
                Self::server_error().debug(stringify!($ty))
            }
        }
    };
}

fse!(reqwest::Error);
fse!(std::io::Error);
fse!(serde_json::Error);

#[macro_export]
macro_rules! err {
    (r, $code:ident) => {
        $crate::AppErr::from($crate::ErrorCode::$code)
    };

    (r,$code:ident, $debug:literal) => {
        $crate::AppErr::from($crate::ErrorCode::$code).debug($debug)
    };

    (r,$code:ident, $debug:expr) => {
        $crate::AppErr::from($crate::ErrorCode::$code).debug(&$debug)
    };

    ($code:ident) => {
        Err($crate::AppErr::from($crate::ErrorCode::$code))
    };

    ($code:ident, $debug:literal) => {
        Err($crate::AppErr::from($crate::ErrorCode::$code).debug($debug))
    };

    ($code:ident, $debug:expr) => {
        Err($crate::AppErr::from($crate::ErrorCode::$code).debug(&$debug))
    };
}
