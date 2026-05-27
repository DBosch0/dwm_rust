use std::{
    ffi::CStr,
    io::{Write, stderr},
    process::exit,
};

pub(crate) fn die_impl(msg: &str) -> ! {
    let saved_errno = unsafe { *libc::__errno_location() };

    let mut stderr = stderr().lock();
    let _ = write!(stderr, "{}", msg);

    if msg.ends_with(':') {
        let error_string = unsafe { CStr::from_ptr(libc::strerror(saved_errno)) };
        let _ = write!(stderr, " {}", error_string.to_string_lossy());
    }
    let _ = writeln!(stderr);
    exit(1)
}

#[macro_export]
macro_rules! die {
    ($fmt:literal $(, $arg:expr)*) => {
        $crate::util::die_impl(&format!($fmt $(, $arg)*))
    };
}
