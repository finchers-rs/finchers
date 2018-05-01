use base64::decode;
use finchers::endpoint::header::FromHeader;
use finchers::output::ResponseBody;
use finchers::{Endpoint, HttpError, Input};
use http::{Response, StatusCode};
use std::{error, fmt};

pub fn basic_auth() -> impl Endpoint<Output = BasicAuth> + Send + Sync + 'static {
    use finchers::endpoint::header::header;
    use finchers::endpoint::prelude::*;
    header().ok_or_else(|| Unauthorized).unwrap_ok()
}

#[derive(Debug)]
pub struct BasicAuth {
    pub username: String,
    pub password: Option<String>,
}

impl FromHeader for BasicAuth {
    type Error = Unauthorized;

    fn header_name() -> &'static str {
        "Authorization"
    }

    fn from_header(input: &[u8]) -> Result<Self, Self::Error> {
        if input.get(0..6) != Some(b"Basic ") {
            return Err(Unauthorized);
        }
        let decoded = decode(&input[6..]).map_err(|_| Unauthorized)?;
        let decoded_str = String::from_utf8(decoded).map_err(|_| Unauthorized)?;
        let mut elems = decoded_str.splitn(2, ':');
        let username = elems.next().map(ToOwned::to_owned).ok_or_else(|| Unauthorized)?;
        let password = elems.next().map(ToOwned::to_owned);
        Ok(BasicAuth { username, password })
    }
}

#[derive(Debug)]
pub struct Unauthorized;

impl fmt::Display for Unauthorized {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("unauthorized")
    }
}

impl error::Error for Unauthorized {
    fn description(&self) -> &str {
        "unauthorized"
    }
}

impl HttpError for Unauthorized {
    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }

    fn to_response(&self, _: &Input) -> Option<Response<ResponseBody>> {
        let www_authenticate = format!("Basic realm=\"\"");
        Response::builder()
            .header("WWW-Authenticate", www_authenticate.as_str())
            .body(Default::default())
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::encode;

    #[test]
    fn from_header_case1() {
        let input = format!("Basic {}", encode("alice:wonderland"));
        let auth = BasicAuth::from_header(input.as_ref()).unwrap();
        assert_eq!(auth.username, "alice");
        assert_eq!(auth.password, Some("wonderland".into()));
    }

    #[test]
    fn from_header_case2() {
        let input = format!("Basic {}", encode("alice"));
        let auth = BasicAuth::from_header(input.as_ref()).unwrap();
        assert_eq!(auth.username, "alice");
        assert_eq!(auth.password, None);
    }

    #[test]
    fn from_header_empty() {
        assert!(BasicAuth::from_header("".as_ref()).is_err());
    }

    #[test]
    fn from_header_invalid() {
        assert!(BasicAuth::from_header("🍣".as_ref()).is_err());
    }
}
