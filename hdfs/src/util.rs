use libc::c_char;
use std::ffi::{CStr, CString};
use std::str;

/// Memory may be leaking there
pub fn str_to_chars(s: &str) -> *const c_char {
    let cs = CString::new(s).unwrap();
    let p = cs.as_ptr();

    // We need to forget cstring variable as at the end of the scope the value will be freed.
    std::mem::forget(cs);
    p
}

pub fn chars_to_str<'a>(chars: *const c_char) -> &'a str {
    unsafe { CStr::from_ptr(chars).to_str().unwrap() }
}

#[cfg(test)]
mod test {
    use util;
    #[test]
    fn test_str_to_chars() {
        let blah = util::str_to_chars("blah");
        let result = util::chars_to_str(blah);
        assert_eq!("blah", result);
    }
}
