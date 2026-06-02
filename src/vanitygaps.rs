use crate::{Arg, Globals, Monitor};

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
