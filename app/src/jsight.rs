use std::ffi::{CStr, CString};
use std::str;
use libloading::Symbol;
use libc::c_char;

unsafe extern "C" fn jsight_stat_mock() -> *const c_char {
    println!("JSight stat() function is not initialized. Use jsight::init() first.");

    let my_string: &str = "Hello, World!";
    let c_string: CString = CString::new(my_string).expect("CString conversion failed");
    let c_string_ptr: *const c_char = c_string.as_ptr();
    
    c_string_ptr
}

static jsight_stat_symbol: Symbol<unsafe extern fn() -> *const c_char> = unsafe { Symbol::new(jsight_stat_mock as *const _) };

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let lib = libloading::Library::new("/opt/lib/libjsight.so")?;
        jsight_stat_symbol = lib.get(b"JSightStat")?;
        Ok(())
    }
}

pub fn stat() -> Result<&'static str, Box<dyn std::error::Error>> {
    unsafe {
        let c_str = jsight_stat_symbol();
        let rust_str = CStr::from_ptr(c_str).to_bytes();
        let rust_str = str::from_utf8(rust_str).expect("Invalid UTF-8 string");
        Ok(rust_str)
    }
}    