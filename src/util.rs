use std::{
    ffi::{CStr, CString},
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

#[inline(always)]
pub(crate) const fn sptag(i: u32) -> u32 {
    (1 << crate::config::TAGS.len() as u32) << i
}

fn getparentprocess(p: libc::pid_t) -> libc::pid_t {
    let v = 0u32;

    let mut buf = [0i8; 256];
    let cstr = CString::new(format!("/proc/{}/stat", p as u32)).expect("valid CString");
    unsafe { libc::snprintf(buf.as_mut_ptr(), buf.len() - 1, cstr.as_ptr()) };

    let f = unsafe { libc::fopen(buf.as_ptr(), c"r".as_ptr()) };
    if f.is_null() {
        return 0;
    }

    unsafe { libc::fscanf(f, c"%*u %*s %*c %u".as_ptr(), &v) };
    unsafe { libc::fclose(f) };
    v as libc::pid_t
}

pub(crate) fn isdescprocess(p: libc::pid_t, mut c: libc::pid_t) -> i32 {
    while p != c && c != 0 {
        c = getparentprocess(c);
    }
    c
}

#[inline]
pub(crate) const fn shift(tag: u32, i: i32) -> u32 {
    if i > 0 {
        (tag << i as u32) | (tag >> (crate::config::TAGS.len() as u32 - i as u32))
    } else {
        (tag >> (-i) as u32) | (tag << (crate::config::TAGS.len() as u32 - (-i) as u32))
    }
}
