use std::ffi::{CStr, CString};
use std::str;
use libloading::{Symbol, Library};
use libc::c_char;
use libc::c_int;
use once_cell::sync::OnceCell;

pub struct ValidationError {
    pub reported_by: String,
    pub r#type     : String,
    pub code       : i32,
    pub title      : String,
    pub detail     : String,
    // pub Position: *mut ErrorPosition,
    // pub Trace: *mut *mut ::std::os::raw::c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CErrorPosition {
    pub filepath: *mut c_char,
    pub index   : *mut c_int,
    pub line    : *mut c_int,
    pub col     : *mut c_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CHeader {
    pub name: *mut ::std::os::raw::c_char,
    pub value: *mut ::std::os::raw::c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CValidationError {
    pub reported_by: *mut c_char,
    pub r#type     : *mut c_char,
    pub code       : c_int,
    pub title      : *mut c_char,
    pub detail     : *mut c_char,
    pub position   : *mut CErrorPosition,
    pub trace      : *mut *mut c_char,
}

static LIB_CELL: OnceCell<Library> = OnceCell::new();

static JSIGHT_STAT_SYMBOL_CELL                 : OnceCell<Symbol<unsafe extern fn() -> *const c_char>> = OnceCell::new();
static JSIGHT_VALIDATE_HTTP_REQUEST_SYMBOL_CELL: OnceCell<Symbol<unsafe extern fn (apiSpecFilePath: *const c_char, requestMethod: *const c_char, requestURI: *const c_char, requestHeaders: *const *const CHeader, requestBody: *const c_char) -> *const CValidationError>> = OnceCell::new();
// static JSIGHT_FREE

pub fn init(lib_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        LIB_CELL.set(Library::new(lib_path)?).unwrap();

        let jsight_stat_symbol                  = LIB_CELL.get().unwrap().get(b"JSightStat")?;
        let jsight_validate_http_request_symbol = LIB_CELL.get().unwrap().get(b"JSightValidateHttpRequest")?;

        JSIGHT_STAT_SYMBOL_CELL                 .set(jsight_stat_symbol                 ).unwrap();
        JSIGHT_VALIDATE_HTTP_REQUEST_SYMBOL_CELL.set(jsight_validate_http_request_symbol).unwrap();

        Ok(())
    }
}

pub fn stat() -> Result<&'static str, Box<dyn std::error::Error>> {
    unsafe {
        let func = JSIGHT_STAT_SYMBOL_CELL.get().expect(&format!("The jsight::{} function was not initialized! Call jsight::init() first.", "stat()"));
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
    unsafe{
        let func = JSIGHT_VALIDATE_HTTP_REQUEST_SYMBOL_CELL.get().expect(&format!("The jsight::{} function was not initialized! Call jsight::init() first.", "validate_http_request()"));
        let c_api_spec_path = CString::new(api_spec_path).expect("CString conversion failed");
        let c_method        = CString::new(method       ).expect("CString conversion failed");
        let c_uri           = CString::new(uri          ).expect("CString conversion failed");
        let c_error = func(
            c_api_spec_path.as_ptr(),
            c_method       .as_ptr(),
            c_uri          .as_ptr(),
            std::ptr::null(),
            std::ptr::null()
        );

        if ! c_error.is_null() {
            let reported_by = str::from_utf8(CStr::from_ptr((*c_error).reported_by).to_bytes()).expect("Invalid UTF-8 string").to_owned();
            let _type       = str::from_utf8(CStr::from_ptr((*c_error).r#type     ).to_bytes()).expect("Invalid UTF-8 string").to_owned();
            let title       = str::from_utf8(CStr::from_ptr((*c_error).title      ).to_bytes()).expect("Invalid UTF-8 string").to_owned();
            let detail      = str::from_utf8(CStr::from_ptr((*c_error).detail     ).to_bytes()).expect("Invalid UTF-8 string").to_owned();

            println!("reported_by: {}", reported_by);
            println!("type: {}"       , _type);
            println!("title: {}"      , title);
            println!("detail: {}"     , detail);

            let error = ValidationError {
                reported_by: reported_by,
                r#type     : _type,
                code       : 123,
                title      : title,
                detail     : detail,
            };
            return Err(error);
        }
    }

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

*/