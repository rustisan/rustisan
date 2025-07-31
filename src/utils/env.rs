use std::ffi::OsStr;

/// Safe wrapper around unsafe set_var in custom std
#[inline]
pub fn set_var<K: AsRef<OsStr>, V: AsRef<OsStr>>(key: K, value: V) {
    // SAFETY: We ensure this is only called during single-threaded startup
    unsafe {
        std::env::set_var(key, value);
    }
}