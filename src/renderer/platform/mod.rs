#![allow(dead_code)]
pub mod atlas;
pub mod shader;
pub use std::ffi::CString;

pub fn to_c_str(s: &str) -> CString {
    CString::new(s).unwrap()
}

pub const MAX_TEXTURES: usize = 8;

pub fn get_version() -> String {
    let mut major_version: i32 = 0;
    let mut minor_version: i32 = 0;

    unsafe {
        gl::GetIntegerv(gl::MAJOR_VERSION, &mut major_version);
        gl::GetIntegerv(gl::MINOR_VERSION, &mut minor_version);
    }

    format!("{}.{}", major_version, minor_version).to_string()
}

// 06815484
// 619689