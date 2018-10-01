extern crate libc;
extern crate tokio;
extern crate futures;

use std::ffi::CString;
use std::os::raw::c_char;

use hyper::Client;
use hyper::{Response, Body};

use std::sync::Arc;

use crate::futures::{Future, Stream};
use crate::futures::future::ok;
use std::thread::sleep_ms;

extern "C" {
    pub fn da_initialize(danilaApp: &AlexaWrapper, config_path: *const c_char) -> libc::c_void;
    fn da_run(danilaApp: &AlexaWrapper);
    fn da_joke(danilaApp: &AlexaWrapper);
    fn da_mock_question(danilaApp: &AlexaWrapper, question_sound_path: *const c_char);
}

#[repr(C)]
pub struct AlexaWrapper {
}

pub struct DanilaClient {
    wrapper: Arc<&'static AlexaWrapper>
}

impl DanilaClient {
    pub fn init() -> DanilaClient {
        static mut wrapper: AlexaWrapper = AlexaWrapper{};

        let config_path = CString::new("/Users/dan.atmakin/dev/private/danila.app/AlexaClientSDKConfig.json").unwrap();
        unsafe {
            wrapper = std::mem::uninitialized();
            da_initialize(&wrapper, config_path.as_ptr());
        }

        sleep_ms(5000);
        println!("init is done");

        let raw_ptr;
        unsafe {
            raw_ptr = &wrapper;
        }

        return DanilaClient {
            wrapper: Arc::new(raw_ptr)
        }
    }

    pub fn make_future_for_status_checks(&self) -> Box<Future<Item=(), Error=()> + Send> {
        return make_future_for_status_check(self.wrapper.clone());

    }

}

fn make_future_for_status_check(wrapper: Arc<&'static AlexaWrapper>) -> Box<Future<Item=(), Error=()> + Send> {

    let client = Client::new();
    let url = "http://auto1.danila.app/rest-api/status?city=BERLIN".parse::<hyper::Uri>().unwrap();
    let local_ptr = wrapper.clone();
    let local_ptr_2 = wrapper.clone();
    let future_response = client
        .get(url)
        .and_then( consume_body )
        .and_then( move |raw_body| {
            if is_notification_available(&raw_body) {
                println!("notifications available, ask to deliver.");
                deliver_notification(local_ptr.clone());
                sleep_ms(20000);
            } else {
                println!("no notifications found");
                sleep_ms(1000);
            }

            Ok(())
        }).and_then(move |()| {
            let task = make_future_for_status_check(local_ptr_2.clone());
            tokio::spawn(task);

            Ok(())
        }).map_err(move |e| {
            println!("error happened on requesting for the status: {:?}", e);
        });

    return Box::new(future_response);

}


fn consume_body(rsp: Response<Body>) -> Box<Future<Item=String, Error=hyper::Error> + Send> {
     let future_body = rsp.into_body()
            .fold(Vec::new(), |mut acc, chunk| {
                acc.extend_from_slice(&chunk);
                ok::<Vec<u8>, hyper::Error>(acc)
            })
            .and_then( move |acc| {
                let str_body = String::from_utf8(acc).unwrap();
                ok(str_body)
            });

    return Box::new(future_body);
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct StatusResponse {
    pub message_num: usize
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

fn deliver_notification( wrapper: Arc<&AlexaWrapper>) {
    let c_to_ask = CString::new("/Users/dan.atmakin/dev/private/danila.app/info/ask_danila_to_deliver_notification-2.wav").unwrap();
    unsafe {
        da_mock_question(&wrapper, c_to_ask.as_ptr());
    }
}



