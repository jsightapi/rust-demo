use std::ffi::{CStr, CString};
use std::str;
use libloading::{Symbol, Library};
use libc::c_char;
use libc::c_int;
use once_cell::sync::OnceCell;
use http::HeaderMap;

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
struct CHeader {
    pub name : *const c_char,
    pub value: *const c_char,
}

#[derive(Debug)]
struct CStringHeader {
    pub name : CString,
    pub value: CString
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
    api_spec_path   : &str, 
    method          : &str, 
    uri             : &str, 
    request_headers : &HeaderMap, 
    request_body    : &[u8]
) -> Result<(), ValidationError> {
  
    let func = JSIGHT_VALIDATE_HTTP_REQUEST_SYMBOL_CELL.get().expect(&format!("The jsight::{} function was not initialized! Call jsight::init() first.", "validate_http_request()"));
    let c_api_spec_path = CString::new(api_spec_path).expect("CString conversion failed");
    let c_method        = CString::new(method       ).expect("CString conversion failed");
    let c_uri           = CString::new(uri          ).expect("CString conversion failed");
    let c_body          = CString::new(request_body ).expect("Failed to create CString");

    let c_string_headers = get_c_string_headers( request_headers ).unwrap();
    let c_headers        = get_c_headers       (&c_string_headers).unwrap();
    let c_header_ptrs    = get_c_header_ptrs   (&c_headers       ).unwrap();

    unsafe{

        let c_error = func(
            c_api_spec_path.as_ptr(),
            c_method       .as_ptr(),
            c_uri          .as_ptr(),
            c_header_ptrs  .as_ptr(),
            c_body         .as_ptr()
        );

        if ! c_error.is_null() {
            let reported_by = from_c_str((*c_error).reported_by).unwrap();
            let _type       = from_c_str((*c_error).r#type     ).unwrap();
            let title       = from_c_str((*c_error).title      ).unwrap();
            let detail      = from_c_str((*c_error).detail     ).unwrap();

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

unsafe fn from_c_str(c_str: *mut c_char) -> Result<String, String> {
    let string = str::from_utf8(CStr::from_ptr(c_str).to_bytes()).expect("Invalid UTF-8 string").to_owned();
    Ok(string)
}

fn get_c_string_headers(rust_headers: &HeaderMap) -> Result<Vec<CStringHeader>, String> {
    let mut c_string_headers = Vec::new();

    for (k, v) in rust_headers.iter() {
        let c_string_header = CStringHeader {
            name : CString::new(k.as_str())         .expect("CString conversion failed"),
            value: CString::new(v.to_str().unwrap()).expect("CString conversion failed")
        };
        c_string_headers.push(c_string_header);
    }

    Ok(c_string_headers)
}

fn get_c_headers(c_string_headers: &Vec<CStringHeader>) -> Result<Vec<CHeader>, String> {
    let mut c_headers = Vec::new();

    for c_string_header in c_string_headers.iter() {
        let c_header = CHeader {
            name : c_string_header.name.as_ptr(),
            value: c_string_header.value.as_ptr()
        };
        c_headers.push(c_header);
    }
    Ok(c_headers)
}

fn get_c_header_ptrs(c_headers: &Vec<CHeader>) -> Result<Vec<*const CHeader>, String> {
    let mut c_header_ptrs = Vec::new();
    for c_header in c_headers.iter() {
        let c_header_ptr: *const CHeader = c_header;
        c_header_ptrs.push(c_header_ptr);
    }
    c_header_ptrs.push(std::ptr::null());
    Ok(c_header_ptrs)
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