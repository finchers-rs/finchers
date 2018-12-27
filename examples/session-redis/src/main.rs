use finchers::prelude::*;
use finchers::{path, routes};

use finchers_session::redis::RedisBackend;
use finchers_session::Session;

use failure::Fallible;
use http::{Response, StatusCode};
use redis::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Deserialize, Serialize)]
struct Login {
    username: String,
}

fn main() -> Fallible<()> {
    pretty_env_logger::init();

    let client = Client::open("redis://127.0.0.1/")?;
    let backend = RedisBackend::new(client)
        .key_prefix("my-app-name")
        .cookie_name("my-session-id")
        .timeout(Duration::from_secs(60 * 3));
    let session = Arc::new(backend);

    let greet = path!(@get /)
        .and(session.clone())
        .and_then(|session: Session<_>| {
            session.with(
                |session| match session.get().map(serde_json::from_str::<Login>) {
                    Some(Ok(login)) => Ok(html(format!(
                        "Hello, {}! <br />\n\
                         <form method=\"post\" action=\"/logout\">\n\
                         <input type=\"submit\" value=\"Log out\" />\n\
                         </form>\
                         ",
                        login.username
                    ))),
                    _ => Ok(Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .header("content-type", "text/html; charset=utf-8")
                        .body("<a href=\"/login\">Log in</a>".into())
                        .unwrap()),
                },
            )
        });

    let login = path!(@get /"login"/)
        .and(session.clone())
        .and_then(|session: Session<_>| {
            session.with(
                |session| match session.get().map(serde_json::from_str::<Login>) {
                    Some(Ok(_login)) => Ok(redirect_to("/").map(|_| "")),
                    _ => Ok(html(
                        "login form\n\
                         <form method=\"post\">\n\
                         <input type=\"text\" name=\"username\">\n\
                         <input type=\"submit\">\n\
                         </form>",
                    )),
                },
            )
        });

    let login_post = {
        #[derive(Debug, Deserialize)]
        struct Form {
            username: String,
        }

        path!(@post /"login"/)
            .and(session.clone())
            .and(endpoints::body::urlencoded())
            .and_then(|session: Session<_>, form: Form| {
                session.with(|session| {
                    let value = serde_json::to_string(&Login {
                        username: form.username,
                    }).map_err(finchers::error::fail)?;
                    session.set(value);
                    Ok(redirect_to("/"))
                })
            })
    };

    let logout = path!(@post /"logout"/)
        .and(session.clone())
        .and_then(|session: Session<_>| {
            session.with(|session| {
                session.remove();
                Ok(redirect_to("/"))
            })
        });

    let endpoint = endpoint::EndpointObj::new(routes![greet, login, login_post, logout,]);

    log::info!("Listening on http://127.0.0.1:4000");
    finchers::server::start(endpoint).serve("127.0.0.1:4000")?;

    Ok(())
}

fn redirect_to(location: &str) -> Response<()> {
    Response::builder()
        .status(StatusCode::FOUND)
        .header("location", location)
        .body(())
        .unwrap()
}

fn html<T>(body: T) -> Response<T> {
    Response::builder()
        .header("content-type", "text/html; charset=utf-8")
        .body(body)
        .unwrap()
}
