use std::{
    ffi::CString,
    io::{Write, stderr},
    process::exit,
};

pub(crate) fn die(fmt: &str) -> ! {
    let saved_errno = unsafe { *libc::__errno_location() };

    let mut stderr = stderr().lock();
    let _ = write!(stderr, "{}", fmt);

    if fmt.ends_with(':') {
        let error_string = unsafe { CString::from_raw(libc::strerror(saved_errno)) };
        let error_string = error_string.to_string_lossy();
        let _ = write!(stderr, "{}", error_string);
    }
    let _ = writeln!(stderr);
    exit(1)
}
