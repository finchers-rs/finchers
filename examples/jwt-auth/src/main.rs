use finchers::error;
use finchers::input::Cookies;
use finchers::output::Redirect;
use finchers::prelude::*;
use finchers::server::middleware::log::stdlog;

use cookie::Cookie;
use either::Either;
use http::{Response, StatusCode};
use jsonwebtoken::TokenData;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

const SECRET_KEY: &[u8] = b"this-is-a-very-very-secret-key";

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    user_id: i32,
    iat: i64,
    exp: i64,
    nbf: i64,
}

fn parse_token(token_str: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    jsonwebtoken::decode(token_str, SECRET_KEY, &Default::default())
}

fn generate_token() -> Result<String, jsonwebtoken::errors::Error> {
    let now = time::now_utc().to_timespec();
    let claims = Claims {
        user_id: 0,
        iat: now.sec,
        nbf: now.sec,
        exp: (now + time::Duration::days(1)).sec,
    };
    jsonwebtoken::encode(&Default::default(), &claims, SECRET_KEY)
}

fn html<T>(body: T) -> Response<T> {
    Response::builder()
        .header("content-type", "text/html; charset=utf-8")
        .body(body)
        .expect("should be a valid response")
}

fn main() {
    let login = {
        #[derive(Debug, Deserialize)]
        struct FormData {
            username: String,
            password: String,
        }
        finchers::path!(@post / "login" /)
            .and(endpoints::body::urlencoded())
            .and(endpoints::cookie::cookies())
            .and_then(|form: FormData, mut cookies: Cookies| {
                if form.username == "user1" && form.password == "user1" {
                    let token = generate_token().map_err(error::fail)?;
                    cookies.add(Cookie::new("token", token));
                    Ok(Redirect::found("/"))
                } else {
                    Err(error::err_msg(StatusCode::UNAUTHORIZED, "invalid user"))
                }
            })
    };

    let login_page = finchers::path!(@get / "login" /).map(|| {
        const FORM_HTML: &str = "<form method=post>\n
            <input type=text name=username />\n
            <input type=password name=password />\n
            <input type=submit value=\"Log in \" />\n
        </form>";
        html(FORM_HTML)
    });

    let logout = finchers::path!(@get / "logout" /)
        .and(endpoints::cookie::cookies())
        .map(|mut cookies: Cookies| {
            cookies.remove(Cookie::named("token"));
            Redirect::see_other("/login")
        });

    let index = finchers::path!(@get /)
        .and(endpoints::cookie::cookies())
        .and_then(|cookies: Cookies| match cookies.get("token") {
            Some(cookie) => {
                let token = parse_token(cookie.value()).map_err(error::bad_request)?;
                Ok(Either::Left(html(format!(
                    "<p>logged in (used_id = {})</p>",
                    token.claims.user_id
                ))))
            }
            None => Ok(Either::Right(Redirect::see_other("/login"))),
        });

    let endpoint = index.or(login_page).or(login).or(logout);

    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();

    let addr: SocketAddr = ([127, 0, 0, 1], 4000).into();

    log::info!("Listening on {}", addr);
    finchers::server::start(endpoint)
        .with_middleware(stdlog(log::Level::Info, module_path!()))
        .serve("127.0.0.1:4000")
        .unwrap_or_else(|e| log::error!("{}", e));
}
