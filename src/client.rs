extern crate futures;
extern crate hyper;
extern crate libc;
extern crate rust_gpiozero;
extern crate serde_json;
extern crate tokio;
use rust_gpiozero::*;

use std::ffi::CString;
use std::os::raw::c_char;

use self::hyper::Client;
use self::hyper::{Body, Response};

use std::sync::{Arc, Mutex};

use self::futures::future::ok;
use self::futures::{Future, Stream};
use std::mem;
use std::thread::sleep_ms;

use super::args::ClientConfig;

extern "C" {
    pub fn da_initialize(danilaApp: &AlexaWrapper, config_path: *const c_char) -> libc::c_void;
    fn da_run(danilaApp: &AlexaWrapper);
    fn da_tap(danilaApp: &AlexaWrapper);
    fn da_joke(danilaApp: &AlexaWrapper);
    fn da_mock_question(danilaApp: &AlexaWrapper, question_sound_path: *const c_char);
}

#[repr(C)]
pub struct AlexaWrapper {}

pub struct DanilaClient {
    wrapper: Arc<&'static AlexaWrapper>,
    lock: Arc<Mutex<u32>>,
    country: String,
    question_file: String,
}

impl DanilaClient {
    pub fn init(client_config: ClientConfig) -> DanilaClient {
        println!(
            "initializing client with config {} and country {}",
            client_config.config, client_config.country
        );
        static mut wrapper: AlexaWrapper = AlexaWrapper {};

        let config_path = CString::new(client_config.config).unwrap();
        unsafe {
            wrapper = mem::uninitialized();
            da_initialize(&wrapper, config_path.as_ptr());
        }

        sleep_ms(5000);
        println!("init is done");

        let raw_ptr;
        unsafe {
            raw_ptr = &wrapper;
        }

        return DanilaClient {
            wrapper: Arc::new(raw_ptr),
            lock: Arc::new(Mutex::new(0)),
            country: client_config.country,
            question_file: client_config.question_file,
        };
    }

    pub fn make_future_for_status_checks(&self) -> Box<Future<Item = (), Error = ()> + Send> {
        return make_future_for_status_check(
            self.lock.clone(),
            self.wrapper.clone(),
            self.country.clone(),
            self.question_file.clone(),
        );
    }

    pub fn tap_to_talk(&self) {
        let mut result = self.lock.try_lock();
        while result.is_err() {
            result = self.lock.try_lock();
            println!("failed to aquiare lock, try again in 500ms");
            sleep_ms(500);
        }
        let data = result.unwrap();
        println!("got lock for tap to talk!");

        //        let mut led = LED::new(23);
        //        led.on();

        unsafe {
            da_tap(&self.wrapper);
        }
        sleep_ms(20000);
        //        led.off();
    }

    pub fn run(&self) -> ! {
        unsafe {
            da_run(&self.wrapper);
        }

        loop {
            sleep_ms(1000);
        }
    }
}

fn make_future_for_status_check(
    lock: Arc<Mutex<u32>>,
    wrapper: Arc<&'static AlexaWrapper>,
    country: String,
    question_file: String,
) -> Box<Future<Item = (), Error = ()> + Send> {
    let client = Client::new();
    let request_path = format!("http://auto1.danila.app/rest-api/status?city={}", country);
    let url = request_path.parse::<hyper::Uri>().unwrap();
    let local_ptr = wrapper.clone();
    let _lock = lock.clone();
    let future_response = client
        .get(url)
        .and_then(consume_body)
        .and_then(move |raw_body| {
            if is_notification_available(&raw_body) {
                println!("notifications available, ask to deliver.");
                deliver_notification(local_ptr.clone(), _lock.clone(), question_file.clone());
            } else {
                //                println!("no notifications found");
                sleep_ms(1000);
            }
            let task = make_future_for_status_check(
                _lock.clone(),
                local_ptr.clone(),
                country,
                question_file,
            );
            tokio::spawn(task);

            Ok(())
        })
        .map_err(move |e| {
            println!("error happened on requesting for the status: {:?}", e);
        });

    return Box::new(future_response);
}

fn consume_body(rsp: Response<Body>) -> Box<Future<Item = String, Error = hyper::Error> + Send> {
    let future_body = rsp
        .into_body()
        .fold(Vec::new(), |mut acc, chunk| {
            acc.extend_from_slice(&chunk);
            ok::<Vec<u8>, hyper::Error>(acc)
        })
        .and_then(move |acc| {
            let str_body = String::from_utf8(acc).unwrap();
            ok(str_body)
        });

    return Box::new(future_body);
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct StatusResponse {
    pub message_num: usize,
}

fn is_notification_available(body: &String) -> bool {
    let result = match serde_json::from_str::<StatusResponse>(body) {
        Ok(status) => status.message_num > 0,
        Err(err) => {
            println!("Failed to deserialise response: {:?}", err);
            false
        }
    };

    return result;
}

fn deliver_notification(wrapper: Arc<&AlexaWrapper>, lock: Arc<Mutex<u32>>, question_file: String) {
    let mut result = lock.try_lock();
    while result.is_err() {
        result = lock.try_lock();
        println!("failed to aquiare lock, try again in 1s");
        sleep_ms(1000);
    }
    let data = result.unwrap();
    println!("got lock for notification delivery!");
    //    let mut led = LED::new(23);
    //    led.on();
    let c_to_ask = CString::new(question_file).unwrap();
    unsafe {
        da_mock_question(&wrapper, c_to_ask.as_ptr());
    }
    sleep_ms(20000);

    //    led.off();
}
