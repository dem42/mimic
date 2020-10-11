use crate::util::result::Result;

use std::ffi::CStr;
use std::os::raw::c_char;

pub fn vk_to_string(raw_string_array: &[c_char]) -> Result<String> {
    let raw_string = unsafe {
        let pointer = raw_string_array.as_ptr();
        CStr::from_ptr(pointer)
    };

    Ok(raw_string
        .to_str()?
        .to_owned())
}