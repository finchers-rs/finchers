use std::net::IpAddr;
use structopt::StructOpt;

/// The inner type representing all arguments from command-line.
#[derive(Debug, StructOpt)]
#[structopt(name = "finchers")]
struct Cli {
    #[structopt(short = "h", long = "host", default_value = "127.0.0.1")]
    host: IpAddr,

    #[structopt(short = "p", long = "port", default_value = "5000")]
    port: u16,

    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
}

/// The configuration
#[derive(Debug)]
pub struct Config {
    host: IpAddr,
    port: u16,
    verbose: bool,
    cli: Option<Cli>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            host: [127, 0, 0, 1].into(),
            port: 5000,
            verbose: false,
            cli: None,
        }
    }
}

impl Config {
    pub fn from_env() -> Config {
        let mut config = Config::default();
        config.overwite_cli(Cli::from_args());
        config
    }

    fn overwite_cli(&mut self, cli: Cli) {
        self.host = cli.host;
        self.port = cli.port;
        self.verbose = cli.verbose;
        self.cli = Some(cli);
    }

    pub fn host(&self) -> IpAddr {
        self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn verbose(&self) -> bool {
        self.verbose
    }
}
