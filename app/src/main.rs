use std::ffi::CStr;
use std::str;

// #[link(name = "jsight")]
// extern "C" {
//     fn JSightStat() -> *const libc::c_char;
// }

fn main() {
    println!("Hello, world!");
    
    let stat = call_dynamic().unwrap();
    println!("JSight stat: {}", stat);

    // unsafe {
        // let c_str = JSightStat();
        // let rust_str = CStr::from_ptr(c_str).to_bytes();
        // let rust_str = str::from_utf8(rust_str).expect("Invalid UTF-8 string");
        // println!("Sight Stat: {}", rust_str);
    // }    
}


fn call_dynamic() -> Result<&'static str, Box<dyn std::error::Error>> {
    unsafe {
        let lib = libloading::Library::new("/opt/lib/libjsight.so")?;
        let jsight_stat: libloading::Symbol<unsafe extern fn() -> *const libc::c_char> = lib.get(b"JSightStat")?;
        let c_str = jsight_stat();
        let rust_str = CStr::from_ptr(c_str).to_bytes();
        let rust_str = str::from_utf8(rust_str).expect("Invalid UTF-8 string");
        println!("Sight Stat: {}", rust_str);
        println!("here");
        Ok(rust_str)
    }
}    

