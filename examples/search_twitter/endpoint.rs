use std::error::Error;
use finchers::errors::StdErrorResponseBuilder;
use finchers::http::{IntoResponse, Response};

pub struct EndpointError(Box<Error>);

impl<E: Error + 'static> From<E> for EndpointError {
    fn from(error: E) -> Self {
        EndpointError(Box::new(error))
    }
}

impl IntoResponse for EndpointError {
    fn into_response(self) -> Response {
        StdErrorResponseBuilder::bad_request(self.0).finish()
    }
}

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
