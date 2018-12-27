#[macro_use]
extern crate finchers;
extern crate finchers_template;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
#[macro_use]
extern crate serde;
extern crate handlebars;

use finchers::prelude::*;

use handlebars::Handlebars;

#[derive(Debug, Serialize)]
struct UserInfo {
    name: String,
}

impl UserInfo {
    const TEMPLATE_NAME: &'static str = "index.html";

    const TEMPLATE_STR: &'static str = "\
    <!doctype html>
    <html>
    <head>
        <meta charset=\"utf-8\" />
        <title>Greeting</title>
    </head>
    <body>
        Hello, {{ name }}.
    </body>
    </html>";
}

fn main() {
    pretty_env_logger::init();

    let mut engine = Handlebars::new();
    engine
        .register_template_string(UserInfo::TEMPLATE_NAME, UserInfo::TEMPLATE_STR)
        .unwrap();

    let endpoint = path!(@get /)
        .map(|| UserInfo {
            name: "Alice".into(),
        }).wrap(finchers_template::handlebars(
            engine,
            UserInfo::TEMPLATE_NAME,
        ));

    info!("Listening on http://127.0.0.1:4000");
    finchers::server::start(endpoint)
        .serve("127.0.0.1:4000")
        .unwrap_or_else(|e| error!("{}", e));
}
