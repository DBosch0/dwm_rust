use crate::{Arg, Client, Globals, Monitor, resize};
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
    crate::arrange(Some(globals.selmon), globals);
}

#[allow(dead_code)]
pub(crate) fn togglegaps(_arg: &Arg, globals: &mut Globals) {
    globals.enable_gaps = !globals.enable_gaps;
    crate::arrange(None, globals);
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
    let mut c = crate::Client::nexttiled(m.clients);
    while let Some(c_inner) = c {
        c = crate::Client::nexttiled(unsafe { c_inner.as_ref() }.next);
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
    let mut c = crate::Client::nexttiled(m.clients);
    while let Some(c_inner) = c {
        if n < m.nmaster {
            mfacts += unsafe { c_inner.as_ref() }.cfact;
        } else {
            sfacts += unsafe { c_inner.as_ref() }.cfact;
        }
        c = crate::Client::nexttiled(unsafe { c_inner.as_ref() }.next);
        n += 1;
    }
    n = 0;
    c = crate::Client::nexttiled(m.clients);
    while let Some(c_inner) = c {
        if n < m.nmaster {
            mtotal += msize * ((unsafe { c_inner.as_ref() }.cfact / mfacts) as i32);
        } else {
            stotal += ssize * ((unsafe { c_inner.as_ref() }.cfact / sfacts) as i32);
        }
        c = crate::Client::nexttiled(unsafe { c_inner.as_ref() }.next);
        n += 1
    }

    (
        mfacts,         // total factor of master area
        sfacts,         // total factor of stack area
        msize - mtotal, // the remainder (rest) of pixels after a cfacts master split
        ssize - stotal, // the remainder (rest) of pixels after a cfacts stack split
    )
}

// LAYOUTS

pub(crate) fn bstack(m: &mut Monitor, globals: &mut Globals) {
    let (oh, ov, ih, iv, n) = getgaps(m, globals);
    if n == 0 {
        return;
    }

    let mut mx = m.wx + ov;
    let mut sx = mx;
    let my = m.wy + oh;
    let mut sy = my;
    let mut mh = m.wh - 2 * oh;
    let mut sh = mh;
    let mw = m.ww - 2 * ov - iv * ((n as i32).min(m.nmaster) - 1);
    let sw = m.ww - 2 * ov - iv * (n as i32 - m.nmaster - 1);

    if m.nmaster != 0 && n as i32 > m.nmaster {
        sh = ((mh - ih) as f32 * (1.0 - m.mfact)) as i32;
        mh = mh - ih - sh;
        sx = mx;
        sy = my + mh + ih;
    }

    let (mfacts, sfacts, mrest, srest) = getfacts(m, mw, sw);

    let mut i = 0;
    let mut c = crate::Client::nexttiled(m.clients);
    while let Some(mut c_inner) = c {
        if i < m.nmaster {
            crate::resize(
                unsafe { c_inner.as_mut() },
                mx,
                my,
                (mw as f32 * (unsafe { c_inner.as_ref() }.cfact / mfacts)) as i32
                    + (if i < mrest { 1 } else { 0 })
                    - (2 * unsafe { c_inner.as_ref() }.bw),
                mh - (2 * unsafe { c_inner.as_ref() }.bw),
                false,
                globals,
            );
            mx += unsafe { c_inner.as_ref() }.width() + iv;
        } else {
            crate::resize(
                unsafe { c_inner.as_mut() },
                sx,
                sy,
                (sw as f32 * (unsafe { c_inner.as_ref() }.cfact / sfacts)) as i32
                    + (if (i - m.nmaster) < srest { 1 } else { 0 })
                    - (2 * unsafe { c_inner.as_ref() }.bw),
                sh - (2 * unsafe { c_inner.as_ref() }.bw),
                false,
                globals,
            );
            sx += unsafe { c_inner.as_ref() }.width() + iv;
        }
        c = crate::Client::nexttiled(unsafe { c_inner.as_ref() }.next);
        i += 1;
    }
}

pub(crate) fn bstackhoriz(m: &mut Monitor, globals: &mut Globals) {
    let (oh, ov, ih, iv, n) = getgaps(m, globals);
    if n == 0 {
        return;
    }

    let mut mx = m.wx + ov;
    let sx = mx;
    let my = m.wy + oh;
    let mut sy = my;
    let mut mh = m.wh - 2 * oh;
    let mut sh = m.wh - 2 * oh - ih * (n as i32 - m.nmaster - 1);
    let mw = m.ww - 2 * ov - iv * ((n as i32).min(m.nmaster) - 1);
    let sw = m.ww - 2 * ov;

    if m.nmaster != 0 && n as i32 > m.nmaster {
        sh = ((mh - ih) as f32 * (1.0 - m.mfact)) as i32;
        mh = mh - ih - sh;
        sy = my + mh + ih;
        sh = m.wh - mh - 2 * oh - ih * (n as i32 - m.nmaster);
    }

    let (mfacts, sfacts, mrest, srest) = getfacts(m, mw, sh);

    let mut i = 0;
    let mut c = crate::Client::nexttiled(m.clients);
    while let Some(mut ci) = c {
        if i < m.nmaster {
            crate::resize(
                unsafe { ci.as_mut() },
                mx,
                my,
                (mw as f32 * (unsafe { ci.as_ref() }.cfact / mfacts)) as i32
                    + (if i < mrest { 1 } else { 0 })
                    - (2 * unsafe { ci.as_ref() }.bw),
                mh - (2 * unsafe { ci.as_ref() }.bw),
                false,
                globals,
            );
            mx += unsafe { ci.as_ref() }.width() + iv;
        } else {
            crate::resize(
                unsafe { ci.as_mut() },
                sx,
                sy,
                sw - (2 * unsafe { ci.as_ref() }.bw),
                (sh as f32 * (unsafe { ci.as_ref() }.cfact / sfacts)) as i32
                    + (if (i - m.nmaster) < srest { 1 } else { 0 })
                    - (2 * unsafe { ci.as_ref() }.bw),
                false,
                globals,
            );
            sy += unsafe { ci.as_ref() }.height() + ih;
        }
        c = crate::Client::nexttiled(unsafe { ci.as_ref() }.next);
        i += 1;
    }
}

pub(crate) fn centeredmaster(m: &mut Monitor, globals: &mut Globals) {
    let (oh, ov, ih, iv, n) = getgaps(m, globals);
    if n == 0 {
        return;
    }

    /* initialize areas */
    let mut mx = m.wx + ov;
    let mut my = m.wy + oh;
    let mh = m.wh
        - 2 * oh
        - ih * ((if m.nmaster == 0 {
            n as i32
        } else {
            (n as i32).min(m.nmaster)
        }) - 1);
    let mut mw = m.ww - 2 * ov;
    let lh = m.wh - 2 * oh - ih * (((n as i32 - m.nmaster) / 2) - 1);
    let rh = m.wh
        - 2 * oh
        - ih * (((n as i32 - m.nmaster) / 2)
            - (if (n as i32 - m.nmaster) % 2 == 0 {
                0
            } else {
                1
            }));
    let mut lw = 0;
    let mut rw = 0;
    let mut lx = 0;
    let mut ly = 0;
    let mut rx = 0;
    let mut ry = 0;
    if m.nmaster != 0 && n as i32 > m.nmaster {
        /* go mfact box in the center if more than nmaster clients */
        if n as i32 - m.nmaster > 1 {
            /* ||<-S->|<---M--->|<-S->|| */
            mw = ((m.ww - 2 * ov - 2 * iv) as f32 * m.mfact) as i32;
            lw = (m.ww - mw - 2 * ov - 2 * iv) / 2;
            rw = (m.ww - mw - 2 * ov - 2 * iv) - lw;
            mx += lw + iv;
        } else {
            /* ||<---M--->|<-S->|| */
            mw = ((mw - iv) as f32 * m.mfact) as i32;
            lw = 0;
            rw = m.ww - mw - iv - 2 * ov;
        }
        lx = m.wx + ov;
        ly = m.wy + oh;
        rx = mx + mw + iv;
        ry = m.wy + oh;
    }

    /* calculate facts */
    let mut mfacts = 0.0;
    let mut lfacts = 0.0;
    let mut rfacts = 0.0;
    let mut n = 0;
    let mut c = crate::Client::nexttiled(m.clients);
    while let Some(ci) = c {
        if m.nmaster == 0 || n < m.nmaster {
            mfacts += unsafe { ci.as_ref() }.cfact;
        } else if (n - m.nmaster) % 2 == 0 {
            lfacts += unsafe { ci.as_ref() }.cfact; // total factor of left hand stack area
        } else {
            rfacts += unsafe { ci.as_ref() }.cfact; // total factor of right hand stack area
        }
        c = crate::Client::nexttiled(unsafe { ci.as_ref() }.next);
    }

    n = 0;
    let mut mtotal = 0;
    let mut ltotal = 0;
    let mut rtotal = 0;
    c = crate::Client::nexttiled(m.clients);
    while let Some(ci) = c {
        if m.nmaster == 0 || n < m.nmaster {
            mtotal += (mh as f32 * (unsafe { ci.as_ref() }.cfact / mfacts)) as i32;
        } else if (n - m.nmaster) % 2 == 0 {
            ltotal += (lh as f32 * (unsafe { ci.as_ref() }.cfact / lfacts)) as i32;
        } else {
            rtotal += (rh as f32 * (unsafe { ci.as_ref() }.cfact / rfacts)) as i32;
        }
        c = crate::Client::nexttiled(unsafe { ci.as_ref() }.next);
        n += 1
    }
    let mrest = mh - mtotal;
    let lrest = lh - ltotal;
    let rrest = rh - rtotal;

    let mut i = 0;
    let mut c = crate::Client::nexttiled(m.clients);
    while let Some(mut ci) = c {
        if m.nmaster == 0 || i < m.nmaster {
            /* nmaster clients are stacked vertically, in the center of the screen */
            crate::resize(
                unsafe { ci.as_mut() },
                mx,
                my,
                mw - (2 * unsafe { ci.as_ref() }.bw),
                (mh as f32 * (unsafe { ci.as_ref() }.cfact / mfacts)) as i32
                    + (if i < mrest { 1 } else { 0 })
                    - (2 * unsafe { ci.as_ref() }.bw),
                false,
                globals,
            );
            my += unsafe { ci.as_ref() }.height() + ih;
        } else {
            /* stack clients are stacked vertically */
            if (i - m.nmaster) % 2 == 0 {
                crate::resize(
                    unsafe { ci.as_mut() },
                    lx,
                    ly,
                    lw - (2 * unsafe { ci.as_ref() }.bw),
                    (lh as f32 * (unsafe { ci.as_ref() }.cfact / lfacts)) as i32
                        + (if (i - 2 * m.nmaster) < 2 * lrest {
                            1
                        } else {
                            0
                        })
                        - (2 * unsafe { ci.as_ref() }.bw),
                    false,
                    globals,
                );
                ly += unsafe { ci.as_ref() }.height() + ih;
            } else {
                crate::resize(
                    unsafe { ci.as_mut() },
                    rx,
                    ry,
                    rw - (2 * unsafe { ci.as_ref() }.bw),
                    (rh as f32 * (unsafe { ci.as_ref() }.cfact / rfacts)) as i32
                        + (if (i - 2 * m.nmaster) < 2 * rrest {
                            1
                        } else {
                            0
                        })
                        - (2 * unsafe { ci.as_ref() }.bw),
                    false,
                    globals,
                );
                ry += unsafe { ci.as_ref() }.height() + ih;
            }
        }
        c = crate::Client::nexttiled(unsafe { ci.as_ref() }.next);
        i += 1;
    }
}

pub(crate) fn centeredfloatingmaster(m: &mut Monitor, globals: &mut Globals) {
    let (oh, ov, _ih, iv, n) = getgaps(m, globals);
    if n == 0 {
        return;
    }

    let mut mx = m.wx + ov;
    let mut my = m.wy + oh;
    let mut mh = m.wh - 2 * oh;
    let mut sx = mx;
    let mut sy = my;
    let mut sh = mh;
    let mut mw = m.ww - 2 * ov - iv * (n as i32 - 1);
    let sw = m.ww - 2 * ov - iv * (n as i32 - m.nmaster - 1);

    let mut mivf = 1.0;

    if m.nmaster != 0 && n as i32 > m.nmaster {
        mivf = 0.8;
        /* go mfact box in the center if more than nmaster clients */
        if m.ww > m.wh {
            mw = (m.ww as f32 * m.mfact - iv as f32 * mivf * ((n as i32).min(m.nmaster) - 1) as f32)
                as i32;
            mh = (m.wh as f32 * 0.9) as i32;
        } else {
            mw = (m.ww as f32 * 0.9 - iv as f32 * mivf * ((n as i32).min(m.nmaster) - 1) as f32)
                as i32;
            mh = (m.wh as f32 * m.mfact) as i32;
        }
        mx = m.wx + (m.ww - mw) / 2;
        my = m.wy + (m.wh - mh - 2 * oh) / 2;

        sx = m.wx + ov;
        sy = m.wy + oh;
        sh = m.wh - 2 * oh;
    }

    let (mfacts, sfacts, mrest, srest) = getfacts(m, mw, sw);
    let mut i = 0;
    let mut c = crate::Client::nexttiled(m.clients);
    while let Some(mut ci) = c {
        let cr = unsafe { ci.as_mut() };
        if i < m.nmaster {
            /* nmaster clients are stacked horizontally, in the center of the screen */
            crate::resize(
                cr,
                mx,
                my,
                (mw as f32 * (cr.cfact / mfacts)) as i32 + (if i < mrest { 1 } else { 0 })
                    - (2 * cr.bw),
                mh - (2 * cr.bw),
                false,
                globals,
            );
            mx += cr.width() + (iv as f32 * mivf) as i32;
        } else {
            /* stack clients are stacked horizontally */
            crate::resize(
                cr,
                sx,
                sy,
                (sw as f32 * (cr.cfact / sfacts)) as i32
                    + (if (i - m.nmaster) < srest { 1 } else { 0 })
                    - (2 * cr.bw),
                sh - (2 * cr.bw),
                false,
                globals,
            );
            sx += cr.width() + iv;
        }
        c = cr.next;
        i += 1;
    }
}

pub(crate) fn deck(m: &mut Monitor, globals: &mut Globals) {
    let (oh, ov, ih, iv, n) = getgaps(m, globals);
    if n == 0 {
        return;
    }
    let mx = m.wx + ov;
    let mut my = m.wy + oh;
    let mh = m.wh - 2 * oh - ih * ((n as i32).min(m.nmaster) - 1);
    let mut mw = m.ww - 2 * ov;
    let mut sx = mx;
    let sy = mh;
    let mut sh = mh;
    let mut sw = mw;

    if m.nmaster == 0 && n as i32 > m.nmaster {
        sw = ((mw - iv) as f32 * (1.0 - m.mfact)) as i32;
        mw = mw - iv - sw;
        sx = mx + mw + iv;
        sh = m.wh - 2 * oh;
    }

    let (mfacts, _sfacts, mrest, _srest) = getfacts(m, mh, sh);

    if n as i32 - m.nmaster > 0 {
        /* override layout symbol */
        let cstr = CString::new(format!("D {}", n as i32 - m.nmaster)).expect("valid CStr");
        unsafe { libc::snprintf(m.ltsymbol.as_mut_ptr(), m.ltsymbol.len(), cstr.as_ptr()) };
    }

    let mut i = 0;
    let mut c = crate::Client::nexttiled(m.clients);
    while let Some(mut ci) = c {
        let cr = unsafe { ci.as_mut() };
        if i < m.nmaster {
            crate::resize(
                cr,
                mx,
                my,
                mw - (2 * cr.bw),
                (mh as f32 * (cr.cfact / mfacts)) as i32 + (if i < mrest { 1 } else { 0 })
                    - (2 * cr.bw),
                false,
                globals,
            );
            my += cr.height() + ih;
        } else {
            crate::resize(
                cr,
                sx,
                sy,
                sw - (2 * cr.bw),
                sh - (2 * cr.bw),
                false,
                globals,
            );
        }
        i += 1;
        c = crate::Client::nexttiled(cr.next)
    }
}

pub(crate) fn fibonacci(m: &mut Monitor, s: bool, globals: &mut Globals) {
    let (oh, ov, ih, iv, n) = getgaps(m, globals);
    if n == 0 {
        return;
    }
    let mut nx = m.wx + ov;
    let mut ny = m.wy + oh;
    let mut nw = m.ww - 2 * ov;
    let mut nh = m.wh - 2 * oh;

    let mut i = 0;
    let mut c = crate::Client::nexttiled(m.clients);
    let mut r = true;
    let mut hrest = 0;
    let mut wrest = 0;
    while let Some(mut ci) = c {
        let cr = unsafe { ci.as_mut() };
        if r {
            if (i % 2 != 0 && (nh - ih) / 2 <= (globals.bh + 2 * cr.bw))
                || (i % 2 == 0 && (nw - iv) / 2 <= (globals.bh + 2 * cr.bw))
            {
                r = false;
            }
            if r && i < n - 1 {
                if i % 2 != 0 {
                    let nv = (nh - ih) / 2;
                    hrest = nh - 2 * nv - ih;
                    nh = nv;
                } else {
                    let nv = (nw - iv) / 2;
                    wrest = nw - 2 * nv - iv;
                    nw = nv;
                }

                if (i % 4) == 2 && !s {
                    nx += nw + iv;
                } else if (i % 4) == 3 && !s {
                    ny += nh + ih;
                }
            }

            if (i % 4) == 0 {
                if s {
                    ny += nh + ih;
                    nh += hrest;
                } else {
                    nh -= hrest;
                    ny -= nh + ih;
                }
            } else if (i % 4) == 1 {
                nx += nw + iv;
                nw += wrest;
            } else if (i % 4) == 2 {
                ny += nh + ih;
                nh += hrest;
                if i < n - 1 {
                    nw += wrest;
                }
            } else if (i % 4) == 3 {
                if s {
                    nx += nw + iv;
                    nw -= wrest;
                } else {
                    nw -= wrest;
                    nx -= nw + iv;
                    nh += hrest;
                }
            }
            if i == 0 {
                if n != 1 {
                    nw = (m.ww - iv - 2 * ov)
                        - ((m.ww - iv - 2 * ov) as f32 * (1.0 - m.mfact)) as i32;
                    wrest = 0;
                }
                ny = m.wy + oh;
            } else if i == 1 {
                nw = m.ww - nw - iv - 2 * ov;
            }
            i += 1;
        }

        crate::resize(
            cr,
            nx,
            ny,
            nw - (2 * cr.bw),
            nh - (2 * cr.bw),
            false,
            globals,
        );
        c = crate::Client::nexttiled(cr.next);
    }
}

pub(crate) fn dwindle(m: &mut Monitor, globals: &mut Globals) {
    fibonacci(m, true, globals);
}

pub(crate) fn spiral(m: &mut Monitor, globals: &mut Globals) {
    fibonacci(m, false, globals);
}

pub(crate) fn gaplessgrid(m: &mut Monitor, globals: &mut Globals) {
    let (oh, ov, ih, iv, n) = getgaps(m, globals);
    if n == 0 {
        return;
    }

    /* grid dimensions */
    let mut cols = 0i32;
    while cols <= (n as i32) / 2 {
        if cols * cols >= n as i32 {
            break;
        }
        cols += 1
    }
    if n == 5 {
        /* set layout against the general calculation: not 1:2:2, but 2:3 */
        cols = 2;
    }
    let mut rows = (n as i32) / cols;
    let mut rn = 0; // reset column no, row no, client count
    let mut cn = rn;

    let mut ch = (m.wh - 2 * oh - ih * (rows - 1)) / rows;
    let cw = (m.ww - 2 * ov - iv * (cols - 1)) / cols;
    let mut rrest = (m.wh - 2 * oh - ih * (rows - 1)) - ch * rows;
    let crest = (m.ww - 2 * ov - iv * (cols - 1)) - cw * cols;
    let mut x = m.wx + ov;
    let y = m.wy + oh;

    let mut i = 0;
    let mut c = crate::Client::nexttiled(m.clients);
    while let Some(mut ci) = c {
        let cr = unsafe { ci.as_mut() };
        if i / rows + 1 > cols - (n as i32) % cols {
            rows = (n as i32) / cols + 1;
            ch = (m.wh - 2 * oh - ih * (rows - 1)) / rows;
            rrest = (m.wh - 2 * oh - ih * (rows - 1)) - ch * rows;
        }
        crate::resize(
            cr,
            x,
            y + rn * (ch + ih) + rn.min(rrest),
            cw + (if cn < crest { 1 } else { 0 }) - 2 * cr.bw,
            ch + (if rn < rrest { 1 } else { 0 }) - 2 * cr.bw,
            false,
            globals,
        );
        rn += 1;
        if rn >= rows {
            rn = 0;
            x += cw + ih + (if cn < crest { 1 } else { 0 });
            cn += 1;
        }
        i += 1;
        c = crate::Client::nexttiled(cr.next);
    }
}

pub(crate) fn grid(m: &mut Monitor, globals: &mut Globals) {
    let (oh, ov, ih, iv, n) = getgaps(m, globals);

    /* grid dimensions */
    let mut rows = 0;
    while rows <= (n as i32) / 2 {
        if rows * rows >= n as i32 {
            break;
        }
        rows += 1
    }

    let cols = if rows != 0 && (rows - 1) * rows >= n as i32 {
        rows - 1
    } else {
        rows
    };

    /* window geoms (cell height/width) */
    let ch = (m.wh - 2 * oh - ih * (rows - 1)) / (if rows != 0 { rows } else { 1 });
    let cw = (m.ww - 2 * ov - iv * (cols - 1)) / (if cols != 0 { cols } else { 1 });
    let chrest = (m.wh - 2 * oh - ih * (rows - 1)) - ch * rows;
    let cwrest = (m.ww - 2 * ov - iv * (cols - 1)) - cw * cols;

    let mut i = 0;
    let mut c = crate::Client::nexttiled(m.clients);
    while let Some(mut ci) = c {
        let c_ref = unsafe { ci.as_mut() };
        let cc = i / rows;
        let cr = i % rows;
        let cx = m.wx + ov + cc * (cw + iv) + cc.min(cwrest);
        let cy = m.wy + oh + cr * (ch + ih) + cr.min(chrest);
        crate::resize(
            c_ref,
            cx,
            cy,
            cw + (if cc < cwrest { 1 } else { 0 }) - 2 * c_ref.bw,
            ch + (if cr < chrest { 1 } else { 0 }) - 2 * c_ref.bw,
            false,
            globals,
        );

        c = crate::Client::nexttiled(c_ref.next);
        i += 1;
    }
}

pub(crate) fn horizgrid(m: &mut Monitor, globals: &mut Globals) {
    /* Count windows */
    let (oh, ov, ih, iv, n) = getgaps(m, globals);
    if n == 0 {
        return;
    }

    let (ntop, nbottom) = if n <= 2 {
        (n, 1)
    } else {
        let ntop = n / 2;
        (ntop, n - ntop)
    };
    let mut mx = m.wx + ov;
    let my = m.wy + oh;
    let mut mh = m.wh - 2 * oh;
    let mut mw = m.ww - 2 * ov;
    let mut sx = mx;
    let mut sy = my;
    let mut sh = mh;
    let mut sw = mw;

    if n > ntop {
        sh = (mh - ih) / 2;
        mh = mh - ih - sh;
        sy = my + mh + ih;
        mw = m.ww - 2 * ov - iv * (ntop - 1) as i32;
        sw = m.ww - 2 * ov - iv * (nbottom - 1) as i32;
    }

    /* calculate facts */
    let mut i = 0;
    let mut c = crate::Client::nexttiled(m.clients);
    let mut mfacts = 0.0;
    let mut sfacts = 0.0;
    while let Some(mut ci) = c {
        let cr = unsafe { ci.as_mut() };
        if i < ntop {
            mfacts += cr.cfact;
        } else {
            sfacts += cr.cfact;
        }
        i += 1;
        c = crate::Client::nexttiled(cr.next);
    }

    i = 0;
    c = crate::Client::nexttiled(m.clients);
    let mut mtotal = 0;
    let mut stotal = 0;
    while let Some(mut ci) = c {
        let cr = unsafe { ci.as_mut() };
        if i < ntop {
            mtotal += (mh as f32 * (cr.cfact / mfacts)) as i32;
        } else {
            stotal += (sw as f32 * (cr.cfact / sfacts)) as i32;
        }
        i += 1;
        c = crate::Client::nexttiled(cr.next);
    }
    let mrest = mh - mtotal;
    let srest = sw - stotal;
    i = 0;
    c = crate::Client::nexttiled(m.clients);
    while let Some(mut ci) = c {
        let cr = unsafe { ci.as_mut() };
        if i < ntop {
            crate::resize(
                cr,
                mx,
                my,
                (mw as f32 * (cr.cfact / mfacts)) as i32 + (if (i as i32) < mrest { 1 } else { 0 })
                    - (2 * cr.bw),
                mh - (2 * cr.bw),
                false,
                globals,
            );
            mx += cr.width() + iv;
        } else {
            crate::resize(
                cr,
                sx,
                sy,
                (sw as f32 * (cr.cfact / sfacts)) as i32
                    + (if ((i - ntop) as i32) < srest { 1 } else { 0 })
                    - (2 * cr.bw),
                sh - (2 * cr.bw),
                false,
                globals,
            );
            sx += cr.width() + iv;
        }
        i += 1;
        c = crate::Client::nexttiled(cr.next);
    }
}

pub(crate) fn nrowgrid(m: &mut Monitor, globals: &mut Globals) {
    let mut ri = 0;
    let mut ci = 0;
    let mut uw = 0;
    let mut rows = m.nmaster + 1;

    /* count clients */
    let (oh, ov, ih, iv, n) = getgaps(m, globals);

    /* nothing to do here */
    if n == 0 {
        return;
    }

    /* force 2 clients to always split vertically */
    if crate::config::FORCE_VSPLIT && n == 2 {
        rows = 1;
    }

    /* never allow empty rows */
    if (n as i32) < rows {
        rows = n as i32;
    }

    /* define first row */
    let mut cols = n as i32 / rows;
    let mut uc = cols;
    let mut cy = m.wy + oh;
    let ch = (m.wh - 2 * oh - ih * (rows - 1)) / rows;
    let mut uh = ch;

    let mut c = crate::Client::nexttiled(m.clients);
    while let Some(mut c_inner) = c {
        let c_ref = unsafe { c_inner.as_mut() };
        if ci == cols {
            uw = 0;
            ci = 0;
            ri += 1;

            /* next row */
            cols = (n as i32 - uc) / (rows - ri);
            uc += cols;
            cy = m.wy + oh + uh + ih;
            uh += ch + ih;
        }

        let cx = m.wx + ov + uw;
        let cw = (m.ww - 2 * ov - uw) / (cols - ci);
        uw += cw + iv;

        crate::resize(
            c_ref,
            cx,
            cy,
            cw - (2 * c_ref.bw),
            ch - (2 * c_ref.bw),
            false,
            globals,
        );
        c = crate::Client::nexttiled(c_ref.next);
        ci += 1;
    }
}

pub(crate) fn tile(m: &mut Monitor, globals: &mut Globals) {
    let (oh, ov, ih, iv, n) = getgaps(m, globals);
    if n == 0 {
        return;
    }
    let mx = m.wx + ov;
    let mut sx = mx;
    let mut my = m.wy + oh;
    let mut sy = my;
    let mh = m.wh - 2 * oh - ih * ((n as i32).min(m.nmaster) - 1);
    let sh = m.wh - 2 * oh - ih * (n as i32 - m.nmaster - 1);
    let mut mw = m.ww - 2 * ov;
    let mut sw = mw;

    if m.nmaster != 0 && n > m.nmaster as u32 {
        sw = ((mw - iv) as f32 * (1.0 - m.mfact)) as i32;
        mw = mw - iv - sw;
        sx = mx + mw + iv;
    }

    let (mfacts, sfacts, mrest, srest) = getfacts(m, mh, sh);

    let mut i = 0;
    let mut c = crate::Client::nexttiled(m.clients);
    while let Some(mut c_inner) = c {
        if i < m.nmaster {
            crate::resize(
                unsafe { c_inner.as_mut() },
                mx,
                my,
                mw - (2 * unsafe { c_inner.as_ref() }.bw),
                (mh as f32 * (unsafe { c_inner.as_ref() }.cfact / mfacts)) as i32
                    + if i < mrest { 1 } else { 0 }
                    - 2 * unsafe { c_inner.as_ref() }.bw,
                false,
                globals,
            );
            my += unsafe { c_inner.as_ref() }.height() + ih;
        } else {
            crate::resize(
                unsafe { c_inner.as_mut() },
                sx,
                sy,
                sw - (2 * unsafe { c_inner.as_ref() }.bw),
                (sh as f32 * (unsafe { c_inner.as_ref() }.cfact / sfacts)) as i32
                    + if (i - m.nmaster) < srest { 1 } else { 0 }
                    - 2 * unsafe { c_inner.as_ref() }.bw,
                false,
                globals,
            );
            sy += unsafe { c_inner.as_ref() }.height() + ih;
        }
        c = crate::Client::nexttiled(unsafe { c_inner.as_ref() }.next);
        i += 1;
    }
}

pub(crate) fn monocle(m: &mut Monitor, globals: &mut Globals) {
    let mut n = 0;
    let mut c = m.clients;
    while let Some(c_inner) = c {
        if unsafe { c_inner.as_ref() }.is_visible() {
            n += 1;
        }
        c = unsafe { c_inner.as_ref() }.next;
    }
    if n > 0 {
        unsafe {
            libc::snprintf(
                m.ltsymbol.as_mut_ptr(),
                m.ltsymbol.len(),
                c"[%d]".as_ptr(),
                n,
            )
        };
    }
    let mut c = Client::nexttiled(m.clients);
    while let Some(mut c_inner) = c {
        let (bw, next) = unsafe { (c_inner.as_ref().bw, c_inner.as_ref().next) };
        resize(
            unsafe { c_inner.as_mut() },
            m.wx,
            m.wy,
            m.ww - 2 * bw,
            m.wh - 2 * bw,
            false,
            globals,
        );
        c = Client::nexttiled(next);
    }
}
