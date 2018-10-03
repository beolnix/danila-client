extern crate clap;

use self::clap::{Arg, App};

pub struct ClientConfig {
    pub country: String,
    pub config: String,
    pub question_file: String
}

pub fn init_client_config() -> ClientConfig {
    let app = define_args();
    let client_config = parse_args(app);

    return client_config;
}

fn define_args() -> App<'static, 'static> {
    let app = App::new("Danila client")
        .version("1.0")
        .author("Danila A. <beolnix@gmail.com>")
        .about("Customised alexa client.")
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Sets configuration file")
             .takes_value(true)
             .required(true))
        .arg(Arg::with_name("question")
             .short("q")
             .long("question")
             .value_name("FILE")
             .help("Sets wav file for the notification delivery question of a given country")
             .takes_value(true)
             .required(true))
        .arg(Arg::with_name("COUNTRY")
             .help("Sets the country this device belongs to")
             .required(true)
             .index(1));

    return app;
}

fn parse_args(app: App) -> ClientConfig {
    let matches = app.get_matches();
    let config = matches.value_of("config").expect("config path must be provided");
    let country = matches.value_of("COUNTRY").expect("country must be provided");
    let question_file = matches.value_of("question").expect("path to audio file with question to deliver notifications must be provided");

    return ClientConfig {
        country: country.to_string(),
        config: config.to_string(),
        question_file: question_file.to_string()
    }
}

