use std::fs::OpenOptions;
use std::path::Path;
use egg_mode::{self, KeyPair, Token};
use serde_json;
use tokio_core::reactor::Core;

pub fn retrieve_bearer_token<P: AsRef<Path>>(config_path: P) -> Token {
    #[derive(Deserialize)]
    struct RawConfig {
        consumer_key: String,
        consumer_secret: String,
    }
    let f = OpenOptions::new().read(true).open(config_path).unwrap();
    let RawConfig {
        consumer_key,
        consumer_secret,
    } = serde_json::from_reader(f).unwrap();
    let consumer_token = KeyPair::new(consumer_key, consumer_secret);

    let mut core = Core::new().unwrap();
    let handle = core.handle();
    core.run(egg_mode::bearer_token(&consumer_token, &handle))
        .unwrap()
}
