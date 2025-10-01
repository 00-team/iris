use actix_web::{HttpResponse, web::Json};

pub type Horp = Result<HttpResponse, super::AppErr>;
pub type Jorp<T> = Result<Json<T>, super::AppErr>;

// #[derive(serde::Deserialize, utoipa::IntoParams)]
// pub struct ListParams {
//     #[param(example = 0)]
//     pub page: u32,
// }

#[derive(Debug, Clone, Default, Copy, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    #[default]
    Desc,
    Asc,
}

impl std::fmt::Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        })
    }
}

// macro_rules! sql_enum {
//     ($name:ident) => {
//         impl sqlx::Type<sqlx::Sqlite> for $name {
//             fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
//                 <i64 as sqlx::Type<sqlx::Sqlite>>::type_info()
//             }
//         }
//
//         impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for $name {
//             fn encode_by_ref(
//                 &self, buf: &mut <sqlx::Sqlite as sqlx::Database>::ArgumentBuffer<'q>,
//             ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
//                 buf.push(sqlx::sqlite::SqliteArgumentValue::Int(self.clone() as i32));
//                 Ok(sqlx::encode::IsNull::No)
//             }
//         }
//
//
//         impl sqlx::Decode<'_, sqlx::Sqlite> for $name {
//             fn decode(value: <sqlx::Sqlite as sqlx::Database>::ValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
//                 Ok(Self::from(<i64 as sqlx::Decode::<sqlx::Sqlite>>::decode(value)?))
//             }
//         }
//     };
// }
// pub(crate) use sql_enum;
