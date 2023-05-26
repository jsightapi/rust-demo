use std::ffi::{CStr};
use std::str;
use libloading::{Symbol, Library};
use libc::c_char;
use once_cell::sync::OnceCell;

pub struct ValidationError {
    pub reported_by: String,
    pub r#type     : String,
    pub code       : String,
    pub title      : String,
    pub detail     : String,
    // pub Position: *mut ErrorPosition,
    // pub Trace: *mut *mut ::std::os::raw::c_char,
}



static JSIGHT_STAT_SYMBOL_CELL: OnceCell<Symbol<unsafe extern fn() -> *const c_char>> = OnceCell::new();
static LIB_CELL: OnceCell<Library> = OnceCell::new();

pub fn init(lib_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        LIB_CELL.set(Library::new(lib_path)?).unwrap();
        let jsight_stat_symbol = LIB_CELL.get().unwrap().get(b"JSightStat")?;
        JSIGHT_STAT_SYMBOL_CELL.set(jsight_stat_symbol).unwrap();
        Ok(())
    }
}

pub fn stat() -> Result<&'static str, Box<dyn std::error::Error>> {
    unsafe {
        let func = JSIGHT_STAT_SYMBOL_CELL.get().expect("The jsight::stat() function was not initialized! Call jsight::init() first.");
        let c_str = func();
        let rust_str = CStr::from_ptr(c_str).to_bytes();
        let rust_str = str::from_utf8(rust_str).expect("Invalid UTF-8 string");
        Ok(rust_str)
    }
}

pub fn validate_http_request(
    api_spec_path  : &str, 
    method         : &str, 
    uri            : &str, 
    request_headers: i32, 
    request_body   : &[u8]
) -> Result<(), ValidationError> {
    Ok(())
}

/*
pub fn JSightValidateHttpRequest(
    apiSpecFilePath: *mut ::std::os::raw::c_char,
    requestMethod: *mut ::std::os::raw::c_char,
    requestURI: *mut ::std::os::raw::c_char,
    requestHeaders: *mut *mut Header,
    requestBody: *mut ::std::os::raw::c_char,
) -> *mut ValidationError;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ValidationError {
    pub ReportedBy: *mut ::std::os::raw::c_char,
    pub Type: *mut ::std::os::raw::c_char,
    pub Code: ::std::os::raw::c_int,
    pub Title: *mut ::std::os::raw::c_char,
    pub Detail: *mut ::std::os::raw::c_char,
    pub Position: *mut ErrorPosition,
    pub Trace: *mut *mut ::std::os::raw::c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ErrorPosition {
    pub Filepath: *mut ::std::os::raw::c_char,
    pub Index: *mut ::std::os::raw::c_int,
    pub Line: *mut ::std::os::raw::c_int,
    pub Col: *mut ::std::os::raw::c_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Header {
    pub Name: *mut ::std::os::raw::c_char,
    pub Value: *mut ::std::os::raw::c_char,
}

*/