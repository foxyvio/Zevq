use crate::{
    parser::parse_rust_source,
    solver::{report_to_json, verify_findings},
};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn zevq_audit_source(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return CString::new("{\"status\":\"Crash\",\"error\":\"null input\"}")
            .unwrap()
            .into_raw();
    }

    let source = unsafe { CStr::from_ptr(input) }.to_string_lossy();
    let report = verify_findings(parse_rust_source(&source));
    let json = report_to_json(&report)
        .unwrap_or_else(|error| format!("{{\"status\":\"Crash\",\"error\":\"{error}\"}}"));
    CString::new(json).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn zevq_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}
