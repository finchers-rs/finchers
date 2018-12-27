#[macro_use]
extern crate finchers;
extern crate finchers_template;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
#[macro_use]
extern crate horrorshow;

use finchers::prelude::*;

use horrorshow::helper::doctype;

fn main() {
    std::env::set_var("RUST_LOG", "horrorshow=info");
    pretty_env_logger::init();

    let endpoint = path!(@get /)
        .map(|| {
            html! {
                : doctype::HTML;
                html {
                    head {
                        meta(charset="utf-8");
                        title: "Greeting";
                    }
                    body {
                        p: format!("Hello, {}", "Alice");
                    }
                }
            }
        }).wrap(finchers_template::horrorshow());

    info!("Listening on http://127.0.0.1:4000");
    finchers::server::start(endpoint)
        .serve("127.0.0.1:4000")
        .unwrap_or_else(|e| error!("{}", e));
}
