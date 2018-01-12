use finchers::{ErrorResponder, Responder};
use finchers::contrib::json::Json;
use finchers::http::{FromBody, FromBodyError, StatusCode};
use serde_json;
use service::Todo;

pub enum ApiResponse {
    TheTodo(Todo),
    NewTodo(Todo),
    Todos(Vec<Todo>),
    Deleted,
}
pub use self::ApiResponse::*;

impl Responder for ApiResponse {
    type Body = serde_json::Value;

    fn status(&self) -> StatusCode {
        match *self {
            ApiResponse::NewTodo(..) => StatusCode::Created,
            ApiResponse::Deleted => StatusCode::NoContent,
            _ => StatusCode::Ok,
        }
    }

    fn body(&mut self) -> Option<Self::Body> {
        match *self {
            ApiResponse::TheTodo(ref entry) => Some(serde_json::to_value(entry).unwrap()),
            ApiResponse::NewTodo(ref entry) => Some(serde_json::to_value(entry).unwrap()),
            ApiResponse::Todos(ref entries) => Some(serde_json::to_value(entries).unwrap()),
            ApiResponse::Deleted => None,
        }
    }
}

error_chain! {
    types { ApiError, ApiErrorKind, ResultExt, ApiResult; }
    errors {
        NotFound {
            description("not found"),
            display("The content is not found")
        }
    }
    foreign_links {
        ParseInt(::std::num::ParseIntError);
        ParseJson(FromBodyError<<Json as FromBody>::Error>);
        Service(::service::Error);
    }
}

impl ErrorResponder for ApiError {
    fn status(&self) -> StatusCode {
        match *self.kind() {
            ApiErrorKind::Service(..) => StatusCode::InternalServerError,
            _ => StatusCode::BadRequest,
        }
    }
}

// TODO: use impl Trait
#[macro_export]
macro_rules! build_endpoint {
    () => {{
        use finchers::Endpoint;
        use finchers::contrib::json::Json;
        use finchers::endpoint::{body, path};
        use finchers::endpoint::method::{delete, get, patch, post};
        use api::*;

        e!("todos").with(choice![
            get(()).and_then(|_| -> ApiResult<_> {
                let entries = service::get_todos()?;
                Ok(Todos(entries))
            }),
            post(body().map_err(Into::into)).and_then(|Json(entry)| -> ApiResult<_> {
                let entry = service::add_new_todo(entry)?;
                Ok(NewTodo(entry))
            }),
            get(path().map_err(Into::into)).and_then(|id| -> ApiResult<_> {
                let entry = service::find_todo(id)?.ok_or_else(|| ApiErrorKind::NotFound)?;
                Ok(TheTodo(entry))
            }),
            patch(path().map_err(Into::into))
                .join(body().map_err(Into::into))
                .and_then(|(id, Json(patch))| -> ApiResult<_> {
                    let entry = service::update_todo(id, patch)?.ok_or_else(|| ApiErrorKind::NotFound)?;
                    Ok(TheTodo(entry))
                }),
            delete(path().map_err(Into::into)).and_then(|id| -> ApiResult<_> {
                service::remove_todo(id)?;
                Ok(Deleted)
            }),
        ])
    }};
}
