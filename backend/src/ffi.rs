use crate::audit::audit_payload_json;
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
    let json = audit_payload_json(&source)
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
