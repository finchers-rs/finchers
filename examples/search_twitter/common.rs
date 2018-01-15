use std::io;
use egg_mode;
use tokio_core::reactor::Core;

fn read_line(message: &str) -> String {
    print!("\n{}:\n", message);
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    line
}

pub fn retrieve_access_token(core: &mut Core) -> egg_mode::Token {
    let consumer_key = read_line("Consumer Key").trim().to_string();
    let consumer_secret = read_line("Consume Secret").trim().to_string();
    let consumer_token = egg_mode::KeyPair::new(consumer_key, consumer_secret);

    let handle = core.handle();
    let request_token = core.run(egg_mode::request_token(&consumer_token, "oob", &handle))
        .unwrap();

    println!();
    println!("Open the following URL and retrieve the PIN");
    println!("{}", egg_mode::authorize_url(&request_token));
    println!();

    let pin = read_line("PIN");

    let (access_token, _user_id, username) = core.run(egg_mode::access_token(
        consumer_token,
        &request_token,
        pin,
        &handle,
    )).unwrap();

    println!();
    println!("Logged in as @{}", username);

    access_token
}
