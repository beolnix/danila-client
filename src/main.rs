#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate libc;
extern crate tokio;

extern crate futures;
extern crate bytes;
extern crate tokio_timer;


mod client;
mod args;
use self::client::DanilaClient;

extern crate rust_gpiozero;
use rust_gpiozero::*;

use std::thread;


fn main() {
    let client_config = args::init_client_config();
    let client = DanilaClient::init(client_config);

    let future = client.make_future_for_status_checks();

    client.run();
    // thread::spawn(move || {
    //     let button = Button::new(17);
    //     loop {
    //         button.wait_for_press();
    //         println!("button pressed");
    //         client.tap_to_talk();
    //     }
    // });

    // tokio::run(future);
}





