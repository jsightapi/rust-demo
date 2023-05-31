use std::ffi::{CStr, CString};
use std::{str, ptr};
use std::error::Error;
use libloading::{Symbol, Library};
use libc::{c_char, c_int};
use once_cell::sync::OnceCell;
use http::HeaderMap;

#[derive(Debug, Clone)]
pub struct ErrorPosition {
    pub filepath: Option<String>,
    pub index   : Option<i32>,
    pub line    : Option<i32>,
    pub col     : Option<i32>,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub reported_by: String,
    pub r#type     : String,
    pub code       : i32,
    pub title      : String,
    pub detail     : String,
    pub position   : ErrorPosition,
    pub trace      : Vec<String>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CErrorPosition {
    pub filepath: *const c_char,
    pub index   : *const c_int,
    pub line    : *const c_int,
    pub col     : *const c_int,
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
    pub reported_by: *const c_char,
    pub r#type     : *const c_char,
    pub code       : c_int,
    pub title      : *const c_char,
    pub detail     : *const c_char,
    pub position   : *const CErrorPosition,
    pub trace      : *const *const c_char,
}

static LIB_CELL: OnceCell<Library> = OnceCell::new();

static JSIGHT_STAT_SYMBOL_CELL                  : OnceCell<Symbol<unsafe extern fn() -> *const c_char>> = OnceCell::new();
static JSIGHT_VALIDATE_HTTP_REQUEST_SYMBOL_CELL : OnceCell<Symbol<unsafe extern fn (apiSpecFilePath: *const c_char, requestMethod: *const c_char, requestURI: *const c_char, requestHeaders: *const *const CHeader, requestBody: *const c_char) -> *mut CValidationError>> = OnceCell::new();
static JSIGHT_VALIDATE_HTTP_RESPONSE_SYMBOL_CELL: OnceCell<Symbol<unsafe extern fn (apiSpecFilePath: *const c_char, requestMethod: *const c_char, requestURI: *const c_char, responseStatusCode: c_int, responseHeaders: *const *const CHeader, responseBody: *const c_char) -> *mut CValidationError>> = OnceCell::new();
static JSIGHT_FREE_VALIDATION_ERROR_SYMBOL_CELL : OnceCell<Symbol<unsafe extern fn (error: *const CValidationError)>> = OnceCell::new();
static JSIGHT_SERIALIZE_ERROR_SYMBOL_CELL       : OnceCell<Symbol<unsafe extern fn (format: *const c_char, error: *const CValidationError,) -> *const c_char>> = OnceCell::new();

pub fn init(lib_path: &str) -> Result<(), Box<dyn Error>> {
    unsafe {
        LIB_CELL.set(Library::new(lib_path)?).unwrap();

        let jsight_stat_symbol                   = LIB_CELL.get().unwrap().get(b"JSightStat")?;
        let jsight_validate_http_request_symbol  = LIB_CELL.get().unwrap().get(b"JSightValidateHttpRequest")?;
        let jsight_validate_http_response_symbol = LIB_CELL.get().unwrap().get(b"JSightValidateHttpResponse")?;
        let jsight_free_validation_error_symbol  = LIB_CELL.get().unwrap().get(b"freeValidationError")?;
        let jsight_serialize_error_symbol        = LIB_CELL.get().unwrap().get(b"JSightSerializeError")?;

        JSIGHT_STAT_SYMBOL_CELL                  .set(jsight_stat_symbol                  ).unwrap();
        JSIGHT_VALIDATE_HTTP_REQUEST_SYMBOL_CELL .set(jsight_validate_http_request_symbol ).unwrap();
        JSIGHT_VALIDATE_HTTP_RESPONSE_SYMBOL_CELL.set(jsight_validate_http_response_symbol).unwrap();
        JSIGHT_FREE_VALIDATION_ERROR_SYMBOL_CELL .set(jsight_free_validation_error_symbol ).unwrap();
        JSIGHT_SERIALIZE_ERROR_SYMBOL_CELL       .set(jsight_serialize_error_symbol       ).unwrap();

        Ok(())
    }
}

pub fn stat() -> Result<String, Box<dyn Error>> {
    unsafe {
        let func = JSIGHT_STAT_SYMBOL_CELL.get().expect(&format!("The jsight::{} function was not initialized! Call jsight::init() first.", "stat()"));
        let c_str = func();
        let rust_str = from_c_str(c_str).unwrap();
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
  
    let validate_func = JSIGHT_VALIDATE_HTTP_REQUEST_SYMBOL_CELL.get().expect(&format!("The jsight::{} function was not initialized! Call jsight::init() first.", "validate_http_request()"));
    let c_api_spec_path = CString::new(api_spec_path).expect("CString conversion failed");
    let c_method        = CString::new(method       ).expect("CString conversion failed");
    let c_uri           = CString::new(uri          ).expect("CString conversion failed");
    let c_body          = CString::new(request_body ).expect("Failed to create CString");

    let c_string_headers = get_c_string_headers( request_headers ).unwrap();
    let c_headers        = get_c_headers       (&c_string_headers).unwrap();
    let c_header_ptrs    = get_c_header_ptrs   (&c_headers       ).unwrap();

    unsafe{
        let c_error = validate_func(
            c_api_spec_path.as_ptr(),
            c_method       .as_ptr(),
            c_uri          .as_ptr(),
            c_header_ptrs  .as_ptr(),
            c_body         .as_ptr()
        );

        if ! c_error.is_null() {
            let error = get_validation_error( &(*c_error) ).unwrap();
            let free_error_func = JSIGHT_FREE_VALIDATION_ERROR_SYMBOL_CELL.get().expect(&format!("The jsight::{} function was not initialized! Call jsight::init() first.", "freeValidationError()"));
            free_error_func(c_error);
            return Err(error);
        }
    }

    Ok(())
}

pub fn validate_http_response(
    api_spec_path    : &str, 
    method           : &str, 
    uri              : &str,
    status_code      : i32,
    response_headers : &HeaderMap, 
    response_body    : &[u8]
) -> Result<(), ValidationError> {
  
    let validate_func = JSIGHT_VALIDATE_HTTP_RESPONSE_SYMBOL_CELL.get().expect(&format!("The jsight::{} function was not initialized! Call jsight::init() first.", "validate_http_response()"));
    let c_api_spec_path = CString::new(api_spec_path).expect("CString conversion failed");
    let c_method        = CString::new(method       ).expect("CString conversion failed");
    let c_uri           = CString::new(uri          ).expect("CString conversion failed");
    let c_body          = CString::new(response_body).expect("Failed to create CString");

    let c_string_headers = get_c_string_headers( response_headers).unwrap();
    let c_headers        = get_c_headers       (&c_string_headers).unwrap();
    let c_header_ptrs    = get_c_header_ptrs   (&c_headers       ).unwrap();

    unsafe{
        let c_error = validate_func(
            c_api_spec_path.as_ptr(),
            c_method       .as_ptr(),
            c_uri          .as_ptr(),
            status_code,
            c_header_ptrs  .as_ptr(),
            c_body         .as_ptr()
        );

        if ! c_error.is_null() {
            let error = get_validation_error( &(*c_error) ).unwrap();
            let free_error_func = JSIGHT_FREE_VALIDATION_ERROR_SYMBOL_CELL.get().expect(&format!("The jsight::{} function was not initialized! Call jsight::init() first.", "freeValidationError()"));
            free_error_func(c_error);
            return Err(error);
        }
    }

    Ok(())
}

pub fn serialize_error(format: &str, error: ValidationError) -> Result<String, Box<dyn Error>> {
    let c_format      = CString::new(format                    ).expect("CString conversion failed");
    let c_reported_by = CString::new(error.reported_by.as_str()).expect("CString conversion failed");
    let c_type        = CString::new(error.r#type     .as_str()).expect("CString conversion failed");
    let c_title       = CString::new(error.title      .as_str()).expect("CString conversion failed");
    let c_detail      = CString::new(error.detail     .as_str()).expect("CString conversion failed");

    let mut c_position = CErrorPosition {
        filepath : ptr::null(),
        index    : ptr::null(),
        line     : ptr::null(),
        col      : ptr::null()
    };

    if error.position.filepath.is_some() {
        let filepath = CString::new(error.position.filepath.unwrap().as_str()).expect("CString conversion failed");
        c_position.filepath = filepath.as_ptr();
    }

    if error.position.index.is_some() {
        c_position.index = &error.position.index.unwrap();
    }

    if error.position.line.is_some() {
        c_position.line = &error.position.line.unwrap();
    }

    if error.position.col.is_some() {
        c_position.col = &error.position.col.unwrap();
    }

    let c_strings     = get_c_strings  (&error.trace).unwrap();
    let c_string_ptrs = get_c_string_ptrs(&c_strings).unwrap();

    let c_error = CValidationError {
        reported_by : c_reported_by.as_ptr(),
        r#type      : c_type.as_ptr(),
        code        : error.code,
        title       : c_title.as_ptr(),
        detail      : c_detail.as_ptr(),
        position    : &c_position,
        trace       : c_string_ptrs.as_ptr(),        
    };

    unsafe {
        let func = JSIGHT_SERIALIZE_ERROR_SYMBOL_CELL.get().expect(&format!("The jsight::{} function was not initialized! Call jsight::init() first.", "serialize_error()"));
        let c_str = func(c_format.as_ptr(), &c_error);
        let rust_str = from_c_str(c_str).unwrap();
        Ok(rust_str)
    }
}

unsafe fn get_validation_error(c_error: &CValidationError) -> Result<ValidationError, Box<dyn Error>> {
    let reported_by = from_c_str(c_error.reported_by).unwrap();
    let _type       = from_c_str(c_error.r#type     ).unwrap();
    let title       = from_c_str(c_error.title      ).unwrap();
    let detail      = from_c_str(c_error.detail     ).unwrap();

    let mut position = ErrorPosition {
        filepath : None,
        index    : None,
        line     : None,
        col      : None
    };

    if ! (*c_error).position.is_null() {
        let c_position = (*c_error).position;

        if ! (*c_position).filepath.is_null() {
            position.filepath = Some(from_c_str((*c_position).filepath).unwrap());
        }
        if ! (*c_position).index.is_null() {
            position.index = Some(*(*c_position).index);
        }
        if ! (*c_position).line.is_null() {
            position.line  = Some(*(*c_position).line);
        }                
        if ! (*c_position).col.is_null() {
            position.col   = Some(*(*c_position).col);
        }
    }

    let mut trace : Vec<String> = Vec::new();
    if ! c_error.trace.is_null() {
        let c_trace = c_error.trace;
        let mut i = 0;
        while !(*c_trace.offset(i)).is_null() {
            let c_string = CStr::from_ptr(*c_trace.offset(i));
            let string = c_string.to_string_lossy().into_owned();
            trace.push(string);
            i += 1;
        }                
    }

    let error = ValidationError {
        reported_by: reported_by,
        r#type     : _type,
        code       : c_error.code,
        title      : title,
        detail     : detail,
        position   : position,
        trace      : trace,
    };    

    Ok(error)
}

unsafe fn from_c_str(c_str: *const c_char) -> Result<String, Box<dyn Error>> {
    let string = str::from_utf8(CStr::from_ptr(c_str).to_bytes()).expect("Invalid UTF-8 string").to_owned();
    Ok(string)
}

fn get_c_string_headers(rust_headers: &HeaderMap) -> Result<Vec<CStringHeader>, Box<dyn Error>> {
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

fn get_c_headers(c_string_headers: &Vec<CStringHeader>) -> Result<Vec<CHeader>, Box<dyn Error>> {
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

fn get_c_header_ptrs(c_headers: &Vec<CHeader>) -> Result<Vec<*const CHeader>, Box<dyn Error>> {
    let mut c_header_ptrs = Vec::new();
    for c_header in c_headers.iter() {
        let c_header_ptr: *const CHeader = c_header;
        c_header_ptrs.push(c_header_ptr);
    }
    c_header_ptrs.push(ptr::null());
    Ok(c_header_ptrs)
}

fn get_c_strings(strings: &Vec<String>) -> Result<Vec<CString>, Box<dyn Error>> {
    let mut c_strings : Vec<CString> = Vec::new();
    for string in strings.iter() {
        let c_string = CString::new(string.as_str()).expect("Failed to create CString");
        c_strings.push(c_string);
    }
    Ok(c_strings)
}

fn get_c_string_ptrs(c_strings: &Vec<CString>) -> Result<Vec<*const c_char>, Box<dyn Error>> {
    let mut c_string_ptrs : Vec<*const c_char> = Vec::new();
    for c_string in c_strings.iter() {
        c_string_ptrs.push(c_string.as_ptr());
    }
    // Add null pointer as the last element
    c_string_ptrs.push(ptr::null());

    Ok(c_string_ptrs)
}
