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
use self::client::DanilaClient;

fn main() {
    let client = DanilaClient::init();
    let future = client.make_future_for_status_checks();

    tokio::run(future);
}



