use std::ffi::{CStr};
use std::str;
use libloading::{Symbol, Library};
use libc::c_char;

use once_cell::sync::OnceCell;

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
