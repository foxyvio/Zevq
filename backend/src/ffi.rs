use crate::{assess_startup, assessment::assessment_to_json, parse_or_default_profile};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn zevq_assess_startup(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return CString::new("{\"verdict\":\"REJECT_FOR_NOW: null input\"}")
            .unwrap()
            .into_raw();
    }

    let payload = unsafe { CStr::from_ptr(input) }.to_string_lossy();
    let profile = parse_or_default_profile(&payload);
    let json = assessment_to_json(&assess_startup(profile))
        .unwrap_or_else(|error| format!("{{\"verdict\":\"REJECT_FOR_NOW: {error}\"}}"));
    CString::new(json).unwrap().into_raw()
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
