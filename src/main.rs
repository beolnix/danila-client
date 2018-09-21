extern crate libc;

use std::ffi::CString;
use libc::c_void;
use std::ptr;
use std::env;

extern {
    fn da_initialize(danilaApp: &DanilaApp) -> libc::c_void;
    fn da_run(danilaApp: &DanilaApp);
    fn da_joke(danilaApp: DanilaApp);
}

#[repr(C)]
struct DanilaApp {
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut app = DanilaApp{};
    unsafe {
        app = std::mem::uninitialized();
        da_initialize(&app);
        da_run(&app);
    }

}
