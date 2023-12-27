#[macro_export]
macro_rules! syscall {
    ($fun:ident, $($arg:expr),* $(,)?) => {
        unsafe {
            let res = libc::$fun($($arg),*);
            if let -1 = res  {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(res)
            }
        }
    }
}
