use crate::{Arg, Client, Globals, Monitor};
// arrange, load_resource_bool, nexttiled, resize};
use std::ffi::CString;

#[allow(dead_code)]
pub(crate) fn setgaps(mut oh: i32, mut ov: i32, mut ih: i32, mut iv: i32, globals: &mut Globals) {
    if oh < 0 {
        oh = 0
    };
    if ov < 0 {
        ov = 0
    };
    if ih < 0 {
        ih = 0
    };
    if iv < 0 {
        iv = 0
    };

    unsafe { globals.selmon.as_mut() }.gappoh = oh;
    unsafe { globals.selmon.as_mut() }.gappov = ov;
    unsafe { globals.selmon.as_mut() }.gappih = ih;
    unsafe { globals.selmon.as_mut() }.gappiv = iv;
    Monitor::arrange(Some(globals.selmon), globals);
}

#[allow(dead_code)]
pub(crate) fn togglegaps(_arg: &Arg, globals: &mut Globals) {
    globals.enable_gaps = !globals.enable_gaps;
    Monitor::arrange(None, globals);
}

#[allow(dead_code)]
pub(crate) fn defaultgaps(_arg: &Arg, globals: &mut Globals) {
    setgaps(
        crate::load_resource!("GAPP_OH", globals, Integer) as i32,
        crate::load_resource!("GAPP_OV", globals, Integer) as i32,
        crate::load_resource!("GAPP_IH", globals, Integer) as i32,
        crate::load_resource!("GAPP_IV", globals, Integer) as i32,
        globals,
    );
}

#[allow(dead_code)]
pub(crate) fn incrgaps(arg: &Arg, globals: &mut Globals) {
    let Arg::I(i) = arg else {
        unreachable!("invalid value given to incrgaps")
    };
    setgaps(
        unsafe { globals.selmon.as_ref() }.gappoh + *i,
        unsafe { globals.selmon.as_ref() }.gappov + *i,
        unsafe { globals.selmon.as_ref() }.gappih + *i,
        unsafe { globals.selmon.as_ref() }.gappiv + *i,
        globals,
    );
}

#[allow(dead_code)]
pub(crate) fn incrigaps(arg: &Arg, globals: &mut Globals) {
    let Arg::I(i) = arg else {
        unreachable!("invalid value given to incrgaps")
    };
    setgaps(
        unsafe { globals.selmon.as_ref() }.gappoh,
        unsafe { globals.selmon.as_ref() }.gappov,
        unsafe { globals.selmon.as_ref() }.gappih + *i,
        unsafe { globals.selmon.as_ref() }.gappiv + *i,
        globals,
    );
}

#[allow(dead_code)]
pub(crate) fn incrogaps(arg: &Arg, globals: &mut Globals) {
    let Arg::I(i) = arg else {
        unreachable!("invalid value given to incrgaps")
    };
    setgaps(
        unsafe { globals.selmon.as_ref() }.gappoh + *i,
        unsafe { globals.selmon.as_ref() }.gappov + *i,
        unsafe { globals.selmon.as_ref() }.gappih,
        unsafe { globals.selmon.as_ref() }.gappiv,
        globals,
    );
}

#[allow(dead_code)]
pub(crate) fn incrovgaps(arg: &Arg, globals: &mut Globals) {
    let Arg::I(i) = arg else {
        unreachable!("invalid value given to incrgaps")
    };
    setgaps(
        unsafe { globals.selmon.as_ref() }.gappoh,
        unsafe { globals.selmon.as_ref() }.gappov + *i,
        unsafe { globals.selmon.as_ref() }.gappih,
        unsafe { globals.selmon.as_ref() }.gappiv,
        globals,
    );
}

#[allow(dead_code)]
pub(crate) fn incrihgaps(arg: &Arg, globals: &mut Globals) {
    let Arg::I(i) = arg else {
        unreachable!("invalid value given to incrgaps")
    };
    setgaps(
        unsafe { globals.selmon.as_ref() }.gappoh,
        unsafe { globals.selmon.as_ref() }.gappov,
        unsafe { globals.selmon.as_ref() }.gappih + *i,
        unsafe { globals.selmon.as_ref() }.gappiv,
        globals,
    );
}

#[allow(dead_code)]
pub(crate) fn incrivgaps(arg: &Arg, globals: &mut Globals) {
    let Arg::I(i) = arg else {
        unreachable!("invalid value given to incrgaps")
    };
    setgaps(
        unsafe { globals.selmon.as_ref() }.gappoh,
        unsafe { globals.selmon.as_ref() }.gappov,
        unsafe { globals.selmon.as_ref() }.gappih,
        unsafe { globals.selmon.as_ref() }.gappiv + *i,
        globals,
    );
}

pub(crate) fn getgaps(m: &Monitor, globals: &mut Globals) -> (i32, i32, i32, i32, u32) {
    let ie = globals.enable_gaps as i32;
    let mut oe = ie;
    let mut n = 0;
    let mut c = Client::nexttiled(m.clients);
    while let Some(c_inner) = c {
        c = Client::nexttiled(unsafe { c_inner.as_ref() }.next);
        n += 1;
    }
    if crate::load_resource!("SMART_GAPS", globals, Bool) && n == 1 {
        oe = 0;
    }
    (
        m.gappoh * oe,
        m.gappov * oe,
        m.gappih * ie,
        m.gappiv * ie,
        n,
    )
}

pub(crate) fn getfacts(m: &Monitor, msize: i32, ssize: i32) -> (f32, f32, i32, i32) {
    let mut mfacts = 0.0;
    let mut sfacts = 0.0;
    let mut mtotal = 0;
    let mut stotal = 0;

    let mut n = 0;
    let mut c = Client::nexttiled(m.clients);
    while let Some(c_inner) = c {
        if n < m.nmaster {
            mfacts += unsafe { c_inner.as_ref() }.cfact;
        } else {
            sfacts += unsafe { c_inner.as_ref() }.cfact;
        }
        c = Client::nexttiled(unsafe { c_inner.as_ref() }.next);
        n += 1;
    }
    n = 0;
    c = Client::nexttiled(m.clients);
    while let Some(c_inner) = c {
        if n < m.nmaster {
            mtotal += msize * ((unsafe { c_inner.as_ref() }.cfact / mfacts) as i32);
        } else {
            stotal += ssize * ((unsafe { c_inner.as_ref() }.cfact / sfacts) as i32);
        }
        c = Client::nexttiled(unsafe { c_inner.as_ref() }.next);
        n += 1
    }

    (
        mfacts,         // total factor of master area
        sfacts,         // total factor of stack area
        msize - mtotal, // the remainder (rest) of pixels after a cfacts master split
        ssize - stotal, // the remainder (rest) of pixels after a cfacts stack split
    )
}
