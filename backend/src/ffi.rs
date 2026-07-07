use crate::assessment::assess_payload_json;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn zevq_assess_startup(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return safe_c_string("{\"verdict\":\"REJECT_FOR_NOW\",\"error\":\"null input\"}");
    }

    let payload = unsafe { CStr::from_ptr(input) }.to_string_lossy();
    safe_c_string(&assess_payload_json(&payload))
}

#[no_mangle]
pub extern "C" fn zevq_audit_source(input: *const c_char) -> *mut c_char {
    zevq_assess_startup(input)
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

fn safe_c_string(value: &str) -> *mut c_char {
    let sanitized = value.replace('\0', "");
    CString::new(sanitized)
        .unwrap_or_else(|_| {
            CString::new("{\"verdict\":\"REJECT_FOR_NOW\",\"error\":\"invalid native string\"}")
                .expect("static fallback has no nul")
        })
        .into_raw()
}
