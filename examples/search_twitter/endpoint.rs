use finchers::contrib::urlencoded::{FromUrlEncoded, Parse, UrlDecodeError};
use std::error::Error;
use egg_mode::search::ResultType;

#[derive(Debug)]
pub struct SearchTwitterParam {
    pub query: String,
    pub result_type: Option<ResultType>,
    pub count: Option<u32>,
}

// TODO: custom derive
impl FromUrlEncoded for SearchTwitterParam {
    fn from_urlencoded(iter: Parse) -> Result<Self, UrlDecodeError> {
        let mut query = None;
        let mut result_type = None;
        let mut count = None;
        for (key, value) in iter {
            match &*key {
                "q" => query = Some(value.into_owned()),
                "type" => match &*value {
                    "recent" => result_type = Some(ResultType::Recent),
                    "popular" => result_type = Some(ResultType::Popular),
                    "mixed" => result_type = Some(ResultType::Mixed),
                    s => return Err(UrlDecodeError::invalid_value("type", s.to_owned())),
                },
                "count" => count = Some(value.parse().map_err(|e| UrlDecodeError::other(e))?),
                s => return Err(UrlDecodeError::invalid_key(s.to_owned())),
            }
        }
        Ok(SearchTwitterParam {
            query: query.ok_or_else(|| UrlDecodeError::missing_key("q"))?,
            result_type,
            count,
        })
    }
}

pub struct EndpointError(pub Box<Error>);

impl<E: Error + 'static> From<E> for EndpointError {
    fn from(error: E) -> Self {
        EndpointError(Box::new(error))
    }
}

// TODO: use impl Trait
#[macro_export]
macro_rules! build_endpoint {
    () => {{
        use endpoint::EndpointError;
        use finchers::Endpoint;
        use finchers::contrib::urlencoded::{queries, Form};
        use finchers::endpoint::body;
        use finchers::endpoint::method::{get, post};

        endpoint!("search").with(endpoint![
            get(queries().map_err(EndpointError::from)),
            post(body()).map(|Form(queries)| queries).map_err(EndpointError::from)
        ])
    }};
}
