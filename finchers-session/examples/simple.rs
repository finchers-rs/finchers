#[macro_use]
extern crate finchers;
extern crate finchers_session;
extern crate http;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
#[macro_use]
extern crate serde;
extern crate serde_json;

use finchers::prelude::*;
use finchers_session::in_memory::{InMemoryBackend, InMemorySession};

use http::Response;

type Session = finchers_session::Session<InMemorySession>;

#[derive(Debug, Deserialize, Serialize, Default)]
struct SessionValue {
    text: String,
}

fn main() {
    pretty_env_logger::init();

    let session_endpoint = InMemoryBackend::default();

    let endpoint = path!(@get /)
        .and(session_endpoint)
        .and_then(|session: Session| {
            session.with(|session| {
                // Retrieve the value of session.
                //
                // Note that the session value are stored as a UTF-8 string,
                // which means that the user it is necessary for the user to
                // deserialize/serialize the session data.
                let mut session_value: SessionValue = {
                    let s = session.get().unwrap_or(r#"{ "text": "" }"#);
                    serde_json::from_str(s).map_err(|err| {
                        finchers::error::bad_request(format!(
                            "failed to parse session value (input = {:?}): {}",
                            s, err
                        ))
                    })?
                };

                let response = Response::builder()
                    .header("content-type", "text/html; charset=utf-8")
                    .body(format!("{:?}", session_value))
                    .expect("should be a valid response");

                session_value.text += "a";

                // Stores session data to the store.
                let s = serde_json::to_string(&session_value).map_err(finchers::error::fail)?;
                session.set(s);

                Ok(response)
            })
        });

    info!("Listening on http://127.0.0.1:4000");
    finchers::server::start(endpoint)
        .serve("127.0.0.1:4000")
        .unwrap_or_else(|err| error!("{}", err));
}
