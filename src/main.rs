use std::ffi::{CStr, CString, c_int};
use std::io::{Write, stderr};
use std::mem::MaybeUninit;
use std::os::raw::c_void;
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::argument::Arg;
use crate::client::Client;
use crate::drw::{Clr, Cur, Drw};
use crate::external_functions::*;
use crate::monitor::Monitor;
use crate::resource::{ResourceVal, Resources};

mod argument;
mod client;
mod config;
mod drw;
mod external_functions;
mod monitor;
mod resource;
mod util;

const VERSION: &str = "0.0.1";
const NUMTAGS: u32 = (config::TAGS.len() + config::SCRATCHPADS.len()) as u32;
const TAGMASK: u32 = (1 << NUMTAGS) - 1;
const BUTTON_MASK: i64 = BUTTON_PRESS_MASK | BUTTON_RELEASE_MASK;
const SPTAGMASK: u32 = ((1 << config::SCRATCHPADS.len()) as u32 - 1) << config::TAGS.len() as u32;
const MOUSE_MASK: i64 = BUTTON_MASK | POINTER_MOTION_MASK;
const PREV_SEL: i32 = 3000;
const BROKEN: &CStr = c"broken";

const CURSOR_STATE_NORMAL: usize = 0;
const CURSOR_STATE_RESIZE: usize = 1;
const CURSOR_STATE_MOVE: usize = 2;
const CURSOR_STATE_LAST: usize = 3;

const SCHEME_STATE_NORM: usize = 0;
const SCHEME_STATE_SEL: usize = 1;

const NET_SUPPORTED: usize = 0;
const NET_WM_NAME: usize = 1;
const NET_WM_STATE: usize = 2;
const NET_WM_CHECK: usize = 3;
const NET_WM_FULLSCREEN: usize = 4;
const NET_WM_STICKY: usize = 5;
const NET_ACTIVE_WINDOW: usize = 6;
const NET_WM_WINDOW_TYPE: usize = 7;
const NET_WM_WINDOW_TYPE_DIALOG: usize = 8;
const NET_CLIENT_LIST: usize = 9;
const NET_LAST: usize = 10;

const WM_PROTOCOLS: usize = 0;
const WM_DELETE: usize = 1;
const WM_STATE: usize = 2;
const WM_TAKE_FOCUS: usize = 3;
const WM_LAST: usize = 4;

type EventHandlerFunction = fn(&mut XEvent, &mut Globals);
type XErrorFunction = extern "C" fn(*mut Display, *mut XErrorEvent) -> c_int;

// dwm is single-threaded; Relaxed ordering is sufficient.
// Written once in checkotherwm before any X error can occur;
// read only in xerror thereafter.
static XERRORXLIB: AtomicUsize = AtomicUsize::new(0);

#[derive(PartialEq, Eq)]
enum ClickState {
    TagBar,
    LtSymbol,
    StatusText,
    WinTitle,
    ClientWin,
    RootWin,
}

struct Layout {
    symbol: &'static str,
    arrange: Option<monitor::layouts::LayoutFunction>,
}

struct ScratchPad {
    name: &'static str,
    cmd: &'static [&'static str],
}

struct Rule {
    class: &'static str,
    instance: &'static str,
    title: &'static str,
    tags: u32,
    isfloating: bool,
    isterminal: bool,
    noswallow: bool,
    monitor: i32,
}

#[derive(Debug)]
struct Globals {
    stext: [i8; 256],
    screen: i32,
    sw: i32, /* X display screen geometry width, height */
    sh: i32,
    bh: i32,    /* bar height */
    lrpad: i32, /* sum of left and right padding for text */
    numlockmask: u32,
    wmatom: [Atom; WM_LAST],
    netatom: [Atom; NET_LAST],
    running: bool,
    cursor: [Cur; CURSOR_STATE_LAST],
    scheme: Box<[Rc<[Clr]>]>,
    dpy: NonNull<Display>,
    drw: Box<Drw>,
    mons: NonNull<Monitor>,
    selmon: NonNull<Monitor>,
    root: Window,
    wmcheckwin: Window,
    last_motion_mon: Option<NonNull<Monitor>>,
    resources: Resources,
    xcon: NonNull<xcb_connection_t>,
    statusw: i32,
    statussig: i32,
    statuspid: libc::pid_t,
    enable_gaps: bool,
}

//HELPERS:

#[inline(always)]
fn text_w(x: *const i8, globals: &mut Globals) -> i32 {
    globals.drw.fontset_getwidth(x) as i32 + globals.lrpad
}

#[inline(always)]
const fn cleanmask(mask: u32, globals: &Globals) -> u32 {
    mask & !(globals.numlockmask | LOCK_MASK)
        & (SHIFT_MASK | CONTROL_MASK | MOD1_MASK | MOD2_MASK | MOD3_MASK | MOD4_MASK | MOD5_MASK)
}

#[inline(always)]
const fn sptag(i: u32) -> u32 {
    (1 << config::TAGS.len() as u32) << i
}

#[inline]
const fn shift(tag: u32, i: i32) -> u32 {
    if i > 0 {
        (tag << i as u32) | (tag >> (config::TAGS.len() as u32 - i as u32))
    } else {
        (tag >> (-i) as u32) | (tag << (config::TAGS.len() as u32 - (-i) as u32))
    }
}

const fn event_handler(event_type: i32) -> Option<EventHandlerFunction> {
    match event_type {
        KEY_PRESS => Some(keypress),
        BUTTON_PRESS => Some(buttonpress),
        MOTION_NOTIFY => Some(motionnotify),
        ENTER_NOTIFY => Some(enternotify),
        FOCUS_IN => Some(focusin),
        EXPOSE => Some(expose),
        DESTROY_NOTIFY => Some(destroynotify),
        UNMAP_NOTIFY => Some(unmapnotify),
        MAP_REQUEST => Some(maprequest),
        CONFIGURE_NOTIFY => Some(configurenotify),
        CONFIGURE_REQUEST => Some(configurerequest),
        PROPERTY_NOTIFY => Some(propertynotify),
        CLIENT_MESSAGE => Some(clientmessage),
        MAPPING_NOTIFY => Some(mappingnotify),
        _ => None,
    }
}

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

fn winpid(w: Window, globals: &Globals) -> libc::pid_t {
    let mut result: libc::pid_t = 0;
    const XCB_RES_CLIENT_ID_MASK_LOCAL_CLIENT_PID: u32 = 2;

    let mut spec = xcb_res_client_id_spec_t {
        client: w as u32,
        mask: XCB_RES_CLIENT_ID_MASK_LOCAL_CLIENT_PID,
    };

    let mut e: *mut xcb_generic_error_t = core::ptr::null_mut();
    let c: xcb_res_query_client_ids_cookie_t =
        unsafe { xcb_res_query_client_ids(globals.xcon.as_ptr(), 1, &spec) };
    let r: *mut xcb_res_query_client_ids_reply_t =
        unsafe { xcb_res_query_client_ids_reply(globals.xcon.as_ptr(), c, &mut e) };

    if r.is_null() {
        return 0 as libc::pid_t;
    }

    let mut i: xcb_res_client_id_value_iterator_t =
        unsafe { xcb_res_query_client_ids_ids_iterator(r) };
    while i.rem != 0 {
        spec = unsafe { &*i.data }.spec;
        if spec.mask & XCB_RES_CLIENT_ID_MASK_LOCAL_CLIENT_PID != 0 {
            let t = unsafe { xcb_res_client_id_value_value(i.data) };
            result = unsafe { *t } as i32;
            break;
        }
        unsafe { xcb_res_client_id_value_next(&mut i) }
    }

    unsafe { libc::free(r as *mut c_void) };

    if result == (-1) as libc::pid_t {
        result = 0;
    }
    result
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

fn isdescprocess(p: libc::pid_t, mut c: libc::pid_t) -> i32 {
    while p != c && c != 0 {
        c = getparentprocess(c);
    }
    c
}

fn buttonpress(ev: &mut XEvent, globals: &mut Globals) {
    const REPLAY_POINTER: i32 = 2;

    let mut click = ClickState::RootWin;
    let ev: &mut XButtonPressedEvent = unsafe { &mut ev.xbutton };
    let mut arg = Arg::Ui(0);

    /* focus monitor if necessary */
    let m = Monitor::wintomon(ev.window, globals);
    if m != globals.selmon {
        Client::unfocus(unsafe { globals.selmon.as_ref() }.sel, true, globals);
        globals.selmon = m;
        Client::focus(None, globals);
    }
    if ev.window == unsafe { globals.selmon.as_ref() }.barwin {
        let mut i = 0;
        let mut x = 0;
        let mut occ: u32 = 0;
        let mut c = unsafe { m.as_ref() }.clients;
        while let Some(c_inner) = c {
            occ |= if unsafe { c_inner.as_ref() }.tags == TAGMASK {
                0
            } else {
                unsafe { c_inner.as_ref() }.tags
            };
            c = unsafe { c_inner.as_ref() }.next;
        }

        loop {
            if occ & 1 << i != 0
                || unsafe { m.as_ref() }.tagset[unsafe { m.as_ref() }.seltags as usize] & 1 << i
                    != 0
            {
                let ctag = CString::new(config::TAGS[i]).expect("valid CStr");
                x += text_w(ctag.as_ptr(), globals);
                if ev.x < x {
                    break; // clicked on tag i
                }
            }
            i += 1;
            if i >= config::TAGS.len() {
                break; // clicked past all tags — i == TAGS.len()
            }
        }

        if i < config::TAGS.len() {
            click = ClickState::TagBar;
            arg = Arg::Ui(1 << i)
        } else if ev.x
            < x + text_w(
                unsafe { globals.selmon.as_ref() }.ltsymbol.as_ptr(),
                globals,
            )
        {
            click = ClickState::LtSymbol
        } else if ev.x > unsafe { globals.selmon.as_ref() }.ww - globals.statusw {
            x = unsafe { globals.selmon.as_ref() }.ww - globals.statusw;
            click = ClickState::StatusText;
            globals.statussig = 0;
            let mut text = globals.stext.as_mut_ptr();
            let mut s = globals.stext.as_mut_ptr();
            while unsafe { *s } != 0 && x <= ev.x {
                // for (text = s = stext; *s && x <= ev->x; s++) {
                if ((unsafe { *s }) as u8) < b' ' {
                    let ch = unsafe { *s };
                    unsafe { *s = b'\0' as i8 };
                    x += text_w(text, globals) - globals.lrpad;
                    unsafe { *s = ch };
                    text = unsafe { s.add(1) };
                    if x >= ev.x {
                        break;
                    }
                    /* End clickable section on a matching signal raw byte */
                    if globals.statussig == ch as i32 {
                        globals.statussig = 0;
                    } else {
                        globals.statussig = ch as i32;
                    }
                }
                s = unsafe { s.add(1) };
            }
        } else {
            click = ClickState::WinTitle
        }
    } else if let Some(c) = Client::wintoclient(ev.window, globals) {
        Client::focus(Some(c), globals);
        unsafe { globals.selmon.as_ref() }.restack(globals);

        unsafe { XAllowEvents(globals.dpy.as_ptr(), REPLAY_POINTER, CURRENT_TIME) };
        click = ClickState::ClientWin;
    }

    for i in 0..config::BUTTONS.len() {
        if click == config::BUTTONS[i].click
            && let Some(f) = config::BUTTONS[i].func
            && config::BUTTONS[i].button == ev.button
            && cleanmask(config::BUTTONS[i].mask, globals) == cleanmask(ev.state, globals)
        {
            f(
                if click == ClickState::TagBar
                    && let Arg::I(ai) = config::BUTTONS[i].arg
                    && ai == 0
                {
                    &arg
                } else {
                    &config::BUTTONS[i].arg
                },
                globals,
            )
        }
    }
}

fn clientmessage(ev: &mut XEvent, globals: &mut Globals) {
    let cme: &mut XClientMessageEvent = unsafe { &mut ev.xclient };
    let c = Client::wintoclient(cme.window, globals);
    let Some(mut c) = c else {
        return;
    };
    let cr = unsafe { c.as_mut() };
    if cme.message_type == globals.netatom[NET_WM_STATE] {
        if unsafe { cme.data.l }[1] == globals.netatom[NET_WM_FULLSCREEN] as i64
            || unsafe { cme.data.l }[2] == globals.netatom[NET_WM_FULLSCREEN] as i64
        {
            cr.setfullscreen(
                unsafe { cme.data.l }[0] == 1  /* _NET_WM_STATE_ADD    */
                || (unsafe { cme.data.l }[0] == 2 /* _NET_WM_STATE_TOGGLE */
                && !cr.isfullscreen ),
                globals,
            );
        }

        if unsafe { cme.data.l[1] } == globals.netatom[NET_WM_STICKY] as i64
            || unsafe { cme.data.l[2] } == globals.netatom[NET_WM_STICKY] as i64
        {
            cr.setsticky(
                unsafe { cme.data.l[0] } == 1 || (unsafe { cme.data.l[0] } == 2 && !cr.issticky),
                globals,
            )
        }
    } else if cme.message_type == globals.netatom[NET_ACTIVE_WINDOW]
        && (unsafe { globals.selmon.as_ref().sel }.is_none()
            || c != unsafe { globals.selmon.as_ref() }
                .sel
                .expect("early termination"))
        && !cr.isurgent
    {
        cr.seturgent(true, globals);
    }
}

fn configurerequest(ev: &mut XEvent, globals: &mut Globals) {
    let ev: &mut XConfigureRequestEvent = unsafe { &mut ev.xconfigurerequest };

    if let Some(mut c) = Client::wintoclient(ev.window, globals) {
        let c_ref = unsafe { c.as_mut() };
        let vm = ev.value_mask as u32;
        if vm & CW_BORDER_WIDTH != 0 {
            c_ref.bw = ev.border_width;
        } else if c_ref.isfloating
            || unsafe { globals.selmon.as_ref() }.lt
                [unsafe { globals.selmon.as_ref() }.sellt as usize]
                .arrange
                .is_none()
        {
            let m = unsafe { c_ref.mon.as_ref() };
            if vm & CWX != 0 {
                c_ref.oldx = c_ref.x;
                c_ref.x = m.mx + ev.x;
            }
            if vm & CWY != 0 {
                c_ref.oldy = c_ref.y;
                c_ref.y = m.my + ev.y;
            }
            if vm & CW_WIDTH != 0 {
                c_ref.oldw = c_ref.w;
                c_ref.w = ev.width;
            }
            if vm & CW_HEIGHT != 0 {
                c_ref.oldh = c_ref.h;
                c_ref.h = ev.height;
            }
            if (c_ref.tags & SPTAGMASK) != 0 && c_ref.isfloating {
                c_ref.x = m.wx + (m.ww / 2 - c_ref.width() / 2);
                c_ref.y = m.wy + (m.wh / 2 - c_ref.height() / 2);
            } else {
                if (c_ref.x + c_ref.w) > m.mx + m.mw && c_ref.isfloating {
                    c_ref.x = m.mx + (m.mw / 2 - c_ref.width() / 2); /* center in x direction */
                }
                if (c_ref.y + c_ref.h) > m.my + m.mh && c_ref.isfloating {
                    c_ref.y = m.my + (m.mh / 2 - c_ref.height() / 2); /* center in y direction */
                }
            }
            if (vm & (CWX | CWY)) != 0 && (vm & (CW_WIDTH | CW_HEIGHT)) == 0 {
                c_ref.configure(globals);
            }
            if unsafe { c.as_ref() }.is_visible() {
                unsafe {
                    XMoveResizeWindow(
                        globals.dpy.as_ptr(),
                        c_ref.win,
                        c_ref.x,
                        c_ref.y,
                        c_ref.w as u32,
                        c_ref.h as u32,
                    )
                };
            }
        } else {
            c_ref.configure(globals);
        }
    } else {
        let mut wc = XWindowChanges {
            x: ev.x,
            y: ev.y,
            width: ev.width,
            height: ev.height,
            border_width: ev.border_width,
            sibling: ev.above,
            stack_mode: ev.detail,
        };
        unsafe {
            XConfigureWindow(
                globals.dpy.as_ptr(),
                ev.window,
                ev.value_mask as u32,
                &mut wc,
            )
        };
    }
    unsafe { XSync(globals.dpy.as_ptr(), 0) };
}

fn configurenotify(ev: &mut XEvent, globals: &mut Globals) {
    let ev: &mut XConfigureEvent = unsafe { &mut ev.xconfigure };

    if ev.window == globals.root {
        let dirty = globals.sw != ev.width || globals.sh != ev.height;
        globals.sw = ev.width;
        globals.sh = ev.height;
        if updategeom(globals) || dirty {
            globals.drw.resize(globals.sw as u32, globals.bh as u32);
            updatebars(globals);
            let mut m = Some(globals.mons);
            while let Some(m_inner) = m {
                let m_inner = unsafe { m_inner.as_ref() };
                let mut c = m_inner.clients;
                while let Some(mut c_inner) = c {
                    let c_ref = unsafe { c_inner.as_mut() };
                    if c_ref.isfullscreen {
                        c_ref.resizeclient(m_inner.mx, m_inner.my, m_inner.mw, m_inner.mh, globals);
                    }
                    c = c_ref.next
                }
                unsafe {
                    XMoveResizeWindow(
                        globals.dpy.as_ptr(),
                        m_inner.barwin,
                        m_inner.mx,
                        m_inner.by,
                        m_inner.mw as u32,
                        globals.bh as u32,
                    )
                };
                m = m_inner.next;
            }
            Client::focus(None, globals);
            Monitor::arrange(None, globals);
        }
    }
}

fn destroynotify(ev: &mut XEvent, globals: &mut Globals) {
    let ev: &mut XDestroyWindowEvent = unsafe { &mut ev.xdestroywindow };
    if let Some(c) = Client::wintoclient(ev.window, globals) {
        Client::unmanage(c, true, globals);
    } else if let Some(c) = Client::swallowingclient(ev.window, globals) {
        Client::unmanage(
            unsafe {
                c.as_ref()
                    .swallowing
                    .expect("swallowingclient only returns c when c.swallowing.is_some()")
            },
            true,
            globals,
        );
    }
}

fn enternotify(ev: &mut XEvent, globals: &mut Globals) {
    const NOTIFY_NORMAL: i32 = 0;
    const NOTIFY_INTERIOR: i32 = 2;

    let ev: &mut XCrossingEvent = unsafe { &mut ev.xcrossing };

    if (ev.mode != NOTIFY_NORMAL || ev.detail == NOTIFY_INTERIOR) && ev.window != globals.root {
        return;
    }
    let c = Client::wintoclient(ev.window, globals);
    let m = if let Some(c) = c {
        unsafe { c.as_ref() }.mon
    } else {
        Monitor::wintomon(ev.window, globals)
    };
    if m != globals.selmon {
        Client::unfocus(unsafe { globals.selmon.as_ref() }.sel, true, globals);
        globals.selmon = m;
    } else if c.is_none() || c == unsafe { globals.selmon.as_ref() }.sel {
        return;
    }
    Client::focus(c, globals);
}

fn expose(ev: &mut XEvent, globals: &mut Globals) {
    let ev: &mut XExposeEvent = unsafe { &mut ev.xexpose };
    if ev.count == 0 {
        let m = Monitor::wintomon(ev.window, globals);
        unsafe { m.as_ref() }.drawbar(globals);
    }
}

fn focusin(ev: &mut XEvent, globals: &mut Globals) {
    let ev: &mut XFocusChangeEvent = unsafe { &mut ev.xfocus };
    if let Some(sel) = unsafe { globals.selmon.as_ref() }.sel
        && ev.window != unsafe { sel.as_ref() }.win
    {
        unsafe { sel.as_ref() }.setfocus(globals);
    }
}

fn keypress(ev: &mut XEvent, globals: &mut Globals) {
    let ev: &mut XKeyEvent = unsafe { &mut ev.xkey };
    let keysym = unsafe { XKeycodeToKeysym(globals.dpy.as_ptr(), ev.keycode as KeyCode, 0) };
    for key in config::KEYS.iter() {
        if keysym == key.keysym
            && cleanmask(key.r#mod, globals) == cleanmask(ev.state, globals)
            && let Some(f) = key.func
        {
            f(&key.arg, globals);
        }
    }
}

fn mappingnotify(ev: &mut XEvent, globals: &mut Globals) {
    const MAPPING_KEYBOARD: i32 = 1;

    let ev: &mut XMappingEvent = unsafe { &mut ev.xmapping };
    unsafe { XRefreshKeyboardMapping(ev) };
    if ev.request == MAPPING_KEYBOARD {
        grabkeys(globals);
    }
}

fn maprequest(ev: &mut XEvent, globals: &mut Globals) {
    let mut wa: XWindowAttributes = unsafe { core::mem::zeroed() };
    let ev: &mut XMapRequestEvent = unsafe { &mut ev.xmaprequest };

    if unsafe { XGetWindowAttributes(globals.dpy.as_ptr(), ev.window, &mut wa) } == 0
        || wa.override_redirect != 0
    {
        return;
    }
    if Client::wintoclient(ev.window, globals).is_none() {
        manage(ev.window, &wa, globals);
    }
}

fn motionnotify(ev: &mut XEvent, globals: &mut Globals) {
    let ev: &mut XMotionEvent = unsafe { &mut ev.xmotion };

    if ev.window != globals.root {
        return;
    }
    let m = Monitor::recttomon(ev.x_root, ev.y_root, 1, 1, globals);
    if let Some(last) = globals.last_motion_mon
        && last != m
    {
        Client::unfocus(unsafe { globals.selmon.as_ref() }.sel, true, globals);
        globals.selmon = m;
        Client::focus(None, globals);
    }
    globals.last_motion_mon = Some(m);
}

fn propertynotify(ev: &mut XEvent, globals: &mut Globals) {
    const PROPERTY_DELETE: i32 = 1;

    let ev: &mut XPropertyEvent = unsafe { &mut ev.xproperty };
    let mut trans: Window = 0;

    if ev.window == globals.root && ev.atom == XA_WM_NAME {
        updatestatus(globals);
    } else if ev.state == PROPERTY_DELETE {
    } else if let Some(mut c) = Client::wintoclient(ev.window, globals) {
        let cr = unsafe { c.as_mut() };
        match ev.atom {
            XA_WM_TRANSIENT_FOR
                if !cr.isfloating
                    && (unsafe {
                        XGetTransientForHint(globals.dpy.as_mut(), cr.win, &mut trans)
                    } != 0) =>
            {
                cr.isfloating = Client::wintoclient(trans, globals).is_some();
                if cr.isfloating {
                    Monitor::arrange(Some(cr.mon), globals);
                }
            }

            XA_WM_NORMAL_HINTS => {
                cr.hintsvalid = false;
            }
            XA_WM_HINTS => {
                cr.updatewmhints(globals);
                drawbars(globals);
            }
            _ => {}
        }
        if ev.atom == XA_WM_NAME || ev.atom == globals.netatom[NET_WM_NAME] {
            cr.updatetitle(globals);
            if let Some(sel) = unsafe { cr.mon.as_ref() }.sel
                && c == sel
            {
                unsafe { c.as_ref().mon.as_ref() }.drawbar(globals);
            }
        }
        if ev.atom == globals.netatom[NET_WM_WINDOW_TYPE] {
            unsafe { c.as_mut() }.updatewindowtype(globals);
        }
    }
}

fn unmapnotify(ev: &mut XEvent, globals: &mut Globals) {
    let ev: &mut XUnmapEvent = unsafe { &mut ev.xunmap };

    if let Some(c) = Client::wintoclient(ev.window, globals) {
        if ev.send_event != 0 {
            unsafe { c.as_ref() }.setclientstate(WITHDRAWN_STATE as i64, globals);
        } else {
            Client::unmanage(c, false, globals);
        }
    }
}

extern "C" fn xerrordummy(_dpy: *mut Display, _ee: *mut XErrorEvent) -> i32 {
    0
}

extern "C" fn xerrorstart(_dpy: *mut Display, _ee: *mut XErrorEvent) -> i32 {
    die!("dwm: another window manager is already running");

    #[allow(unreachable_code)]
    // might be necessary for ABI compatability, even though `die` calls the exit syscal.
    // not sure if we need to setup the return value on the stack anyway for ABI compatibility
    // even if it is never actually run.
    1
}

extern "C" fn xerror(dpy: *mut Display, ee: *mut XErrorEvent) -> i32 {
    let ee = unsafe { &mut *ee };

    if ee.error_code == BAD_WINDOW
        || (ee.request_code == X_SETINPUTFOCUS && ee.error_code == BAD_MATCH)
        || (ee.request_code == X_POLYTEXT8 && ee.error_code == BAD_DRAWABLE)
        || (ee.request_code == X_POLYFILLRECTANGLE && ee.error_code == BAD_DRAWABLE)
        || (ee.request_code == X_POLYSEGMENT && ee.error_code == BAD_DRAWABLE)
        || (ee.request_code == X_CONFIGUREWINDOW && ee.error_code == BAD_MATCH)
        || (ee.request_code == X_GRABBUTTON && ee.error_code == BAD_ACCESS)
        || (ee.request_code == X_GRABKEY && ee.error_code == BAD_ACCESS)
        || (ee.request_code == X_COPYAREA && ee.error_code == BAD_DRAWABLE)
    {
        return 0;
    }
    let _ = writeln!(
        stderr(),
        "dwm: fatal error: requested code={}, error code = {}",
        ee.request_code,
        ee.error_code
    );
    // SAFETY: XERRORXLIB is always set in checkotherwm before xerror can be called.
    let xlib: XErrorFunction = unsafe { core::mem::transmute(XERRORXLIB.load(Ordering::Relaxed)) };
    xlib(dpy, ee)
}

fn checkotherwm(dpy: NonNull<Display>) {
    XERRORXLIB.store(
        unsafe { XSetErrorHandler(xerrorstart) } as usize,
        Ordering::Relaxed,
    );
    /* this causes an error if some other window manager is running */
    unsafe {
        XSelectInput(
            dpy.as_ptr(),
            default_root_window(dpy.as_ptr()),
            SUBSTRUCTURE_REDIRECT_MASK,
        );
        XSync(dpy.as_ptr(), 0);
        XSetErrorHandler(xerror);
        XSync(dpy.as_ptr(), 0);
    }
}

fn getrootptr(x: &mut i32, y: &mut i32, globals: &mut Globals) -> bool {
    let mut di: i32 = 0;
    let mut dui: u32 = 0;
    let mut dummy: Window = 0;

    (unsafe {
        XQueryPointer(
            globals.dpy.as_ptr(),
            globals.root,
            &mut dummy,
            &mut dummy,
            x,
            y,
            &mut di,
            &mut di,
            &mut dui,
        )
    }) != 0
}

fn updategeom(globals: &mut Globals) -> bool {
    let mut dirty = false;

    #[cfg(feature = "xinerama")]
    {}

    // We are in initialization
    if !globals.running {
        globals.mons = Monitor::createmon(globals);
    }

    let mons_ref = unsafe { globals.mons.as_mut() };
    if mons_ref.mw != globals.sw || mons_ref.mh != globals.sh {
        dirty = true;
        mons_ref.ww = globals.sw;
        mons_ref.mw = mons_ref.ww;
        mons_ref.wh = globals.sh;
        mons_ref.mh = mons_ref.wh;
        mons_ref.updatebarpos(globals);
    }
    if dirty {
        globals.selmon = globals.mons;
        globals.selmon = Monitor::wintomon(globals.root, globals);
    }

    dirty
}

fn updatebars(globals: &Globals) {
    let mut wa: XSetWindowAttributes = unsafe { std::mem::zeroed() };
    wa.override_redirect = 1;
    wa.background_pixel = PARENT_RELATIVE;
    wa.event_mask = BUTTON_PRESS_MASK | EXPOSURE_MASK;

    let mut ch = XClassHint {
        res_name: c"dwm".as_ptr(),
        res_class: c"dwm".as_ptr(),
    };

    let mut m = Some(globals.mons);
    while let Some(mut m_inner) = m {
        let m_ref = unsafe { m_inner.as_mut() };
        if m_ref.barwin > 0 {
            m = m_ref.next;
            continue;
        }
        m_ref.barwin = unsafe {
            XCreateWindow(
                globals.dpy.as_ptr(),
                globals.root,
                m_ref.wx,
                m_ref.by,
                m_ref.ww as u32,
                globals.bh as u32,
                0,
                default_depth(globals.dpy.as_ptr(), globals.screen),
                COPY_FROM_PARENT as u32,
                default_visual(globals.dpy.as_ptr(), globals.screen),
                CW_OVERRIDE_REDIRECT | CW_BACK_PIXMAP | CW_EVENT_MASK,
                &mut wa,
            )
        };
        unsafe {
            XDefineCursor(
                globals.dpy.as_ptr(),
                m_ref.barwin,
                globals.cursor[CURSOR_STATE_NORMAL].cursor,
            )
        };
        unsafe { XMapRaised(globals.dpy.as_ptr(), m_ref.barwin) };
        unsafe { XSetClassHint(globals.dpy.as_ptr(), m_ref.barwin, &mut ch) };
        m = m_ref.next;
    }
}

fn gettextprop(w: Window, atom: Atom, text: *mut i8, size: u32, globals: &Globals) -> bool {
    let mut list: *mut *mut i8 = core::ptr::null_mut();
    let mut n = 0;
    let mut name: MaybeUninit<XTextProperty> = MaybeUninit::uninit();

    if text.is_null() || size == 0 {
        return false;
    }

    unsafe { *text = b'\0' as i8 };
    let get_text_property_result = unsafe {
        XGetTextProperty(
            globals.dpy.as_ptr(),
            w,
            &mut name as *mut _ as *mut XTextProperty,
            atom,
        )
    };
    let name = unsafe { name.assume_init() };

    if get_text_property_result == 0 || name.nitems == 0 {
        return false;
    }
    if name.encoding == XA_STRING {
        unsafe { libc::strncpy(text, name.value as *const i8, size as usize - 1) };
    } else if unsafe { XmbTextPropertyToTextList(globals.dpy.as_ptr(), &name, &mut list, &mut n) }
        >= SUCCESS as i32
        && n > 0
        && !list.is_null()
    {
        unsafe { libc::strncpy(text, *list, size as usize - 1) };
        unsafe { XFreeStringList(list) };
    }
    unsafe { *{ text.add((size - 1) as usize) } = b'\0' as i8 };
    unsafe { XFree(name.value.cast()) };

    true
}

fn drawbars(globals: &mut Globals) {
    let mut m = Some(globals.mons);
    while let Some(m_inner) = m {
        let mr = unsafe { m_inner.as_ref() };
        mr.drawbar(globals);
        m = mr.next;
    }
}

fn getstatusbarpid(globals: &mut Globals) -> libc::pid_t {
    let mut buf = [0i8; 32];
    let mut s = buf.as_mut_ptr();
    let mut fp: *mut libc::FILE;

    if globals.statuspid > 0 {
        let cstr =
            CString::new(format!("/proc/{}/cmdline", globals.statuspid)).expect("valid CString");
        unsafe { libc::snprintf(buf.as_mut_ptr(), buf.len(), cstr.as_ptr()) };
        fp = unsafe { libc::fopen(buf.as_mut_ptr(), c"r".as_ptr()) };
        if !fp.is_null() {
            unsafe { libc::fgets(buf.as_mut_ptr(), buf.len() as i32, fp) };
            let mut c = unsafe { libc::strchr(s, b'/' as i32) };
            while !c.is_null() {
                s = unsafe { c.add(1) };
                c = unsafe { libc::strchr(s, b'/' as i32) };
            }
            unsafe { libc::fclose(fp) };
            let status_bar = CString::new(config::STATUS_BAR).expect("valid CString");
            if unsafe { libc::strcmp(s, status_bar.as_ptr()) } == 0 {
                return globals.statuspid;
            }
        }
    }
    let pid_of_cstr =
        CString::new(format!("pidof -s {}", config::STATUS_BAR)).expect("valid CString");
    fp = unsafe { libc::popen(pid_of_cstr.as_ptr(), c"r".as_ptr()) };
    if fp.is_null() {
        return -1;
    }
    unsafe {
        libc::fgets(buf.as_mut_ptr(), buf.len() as i32, fp);
        libc::pclose(fp);
        libc::strtol(buf.as_ptr(), core::ptr::null_mut(), 10) as libc::pid_t
    }
}

fn updatestatus(globals: &mut Globals) {
    if !gettextprop(
        globals.root,
        XA_WM_NAME,
        globals.stext.as_mut_ptr(),
        globals.stext.len() as u32,
        globals,
    ) {
        unsafe {
            libc::strcpy(
                globals.stext.as_mut_ptr(),
                CString::new(format!("dwm-{}", VERSION))
                    .expect("valid C string")
                    .as_ptr(),
            )
        };
        globals.statusw = text_w(globals.stext.as_mut_ptr(), globals) - globals.lrpad + 2;
    } else {
        globals.statusw = 0;
        let mut text = globals.stext.as_mut_ptr();
        let mut s = globals.stext.as_mut_ptr();
        while unsafe { *s } != 0 {
            // for (text = s = stext; *s; s++) {
            if (unsafe { *s } as u8) < b' ' {
                let ch = unsafe { *s };
                unsafe { *s = '\0' as i8 };
                globals.statusw += text_w(text, globals) - globals.lrpad;
                unsafe { *s = ch };
                text = unsafe { s.add(1) };
            }
            s = unsafe { s.add(1) };
        }
        globals.statusw += text_w(text, globals) - globals.lrpad + 2;
    }

    unsafe { globals.selmon.as_ref() }.drawbar(globals);
}

fn updatenumlockmask(globals: &mut Globals) {
    const XK_NUM_LOCK: u64 = 0xff7f;

    globals.numlockmask = 0;
    let modmap = unsafe { XGetModifierMapping(globals.dpy.as_ptr()) };

    for i in 0..8 {
        for j in 0..unsafe { &*modmap }.max_keypermod {
            if unsafe {
                *{
                    { &*modmap }
                        .modifiermap
                        .add((i * { &*modmap }.max_keypermod + j) as usize)
                }
            } == unsafe { XKeysymToKeycode(globals.dpy.as_ptr(), XK_NUM_LOCK) }
            {
                globals.numlockmask = 1 << i;
            }
        }
    }
    unsafe { XFreeModifiermap(modmap) };
}

fn grabkeys(globals: &mut Globals) {
    updatenumlockmask(globals);
    {
        let modifiers = [
            0,
            LOCK_MASK,
            globals.numlockmask,
            globals.numlockmask | LOCK_MASK,
        ];

        let mut start = 0;
        let mut end = 0;
        let mut skip = 0;

        const ANY_KEY: i32 = 0;

        unsafe { XUngrabKey(globals.dpy.as_ptr(), ANY_KEY, ANY_MODIFIER, globals.root) };
        unsafe { XDisplayKeycodes(globals.dpy.as_ptr(), &mut start, &mut end) };
        let syms = unsafe {
            XGetKeyboardMapping(
                globals.dpy.as_ptr(),
                start as u8,
                end - start + 1,
                &mut skip,
            )
        };
        if syms.is_null() {
            return;
        }

        for k in start..=end {
            for i in 0..config::KEYS.len() {
                /* skip modifier codes, we do that ourselves */
                if config::KEYS[i].keysym
                    == unsafe { *syms.add((k - start) as usize * skip as usize) }
                {
                    for modi in modifiers {
                        unsafe {
                            XGrabKey(
                                globals.dpy.as_ptr(),
                                k,
                                config::KEYS[i].r#mod | modi,
                                globals.root,
                                1,
                                GRAB_MODE_ASYNC,
                                GRAB_MODE_ASYNC,
                            )
                        };
                    }
                }
            }
        }
        unsafe { XFree(syms.cast()) };
    }
}

fn getstate(w: Window, globals: &Globals) -> i64 {
    let mut format: i32 = 0;
    let mut result = -1i64;
    let mut p: *mut u8 = core::ptr::null_mut();
    let mut n = 0u64;
    let mut extra = 0u64;
    let mut real: Atom = 0;

    if unsafe {
        XGetWindowProperty(
            globals.dpy.as_ptr(),
            w,
            globals.wmatom[WM_STATE],
            0,
            2,
            0,
            globals.wmatom[WM_STATE],
            &mut real,
            &mut format,
            &mut n,
            &mut extra,
            &mut p,
        )
    } != SUCCESS as i32
    {
        return -1;
    }
    if n != 0 && format == 32 {
        result = unsafe { *p.cast::<i64>() };
    }
    unsafe { XFree(p.cast()) };

    result
}

fn manage(w: Window, wa: &XWindowAttributes, globals: &mut Globals) {
    let mut trans: Window = 0;
    let mut wc: XWindowChanges = unsafe { core::mem::zeroed() };
    let mut c = NonNull::new(Box::into_raw(Box::new(Client {
        name: [0; 256],
        mina: 0.0,
        maxa: 0.0,
        x: wa.x,
        y: wa.y,
        w: wa.width,
        h: wa.height,
        oldx: wa.x,
        oldy: wa.y,
        oldw: wa.width,
        oldh: wa.height,
        basew: 0,
        baseh: 0,
        incw: 0,
        inch: 0,
        maxw: 0,
        maxh: 0,
        minw: 0,
        minh: 0,
        hintsvalid: false,
        bw: 0,
        oldbw: wa.border_width,
        cfact: 1.0,
        tags: 0,
        isfixed: false,
        isfloating: false,
        isurgent: false,
        neverfocus: false,
        oldstate: false,
        isfullscreen: false,
        issticky: false,
        next: None,
        snext: None,
        mon: NonNull::dangling(),
        win: w,
        isterminal: false,
        noswallow: false,
        pid: winpid(w, globals),
        swallowing: None,
    })))
    .expect("valid box construction");

    let c_ref = unsafe { c.as_mut() };
    let mut term: Option<NonNull<Client>> = None;

    c_ref.updatetitle(globals);

    if unsafe { XGetTransientForHint(globals.dpy.as_ptr(), w, &mut trans) } != 0
        && let Some(t) = Client::wintoclient(trans, globals)
    {
        let t_ref = unsafe { t.as_ref() };
        c_ref.mon = t_ref.mon;
        c_ref.tags = t_ref.tags;
    } else {
        c_ref.mon = globals.selmon;
        c_ref.applyrules(globals);
        term = c_ref.termforwin(globals);
    }

    if c_ref.x + c_ref.width() > unsafe { c_ref.mon.as_ref().wx + c_ref.mon.as_ref().ww } {
        c_ref.x = unsafe { c_ref.mon.as_ref().wx + c_ref.mon.as_ref().ww } - c_ref.width();
    }
    if c_ref.y + c_ref.height() > unsafe { c_ref.mon.as_ref().wy + c_ref.mon.as_ref().wh } {
        c_ref.y = unsafe { c_ref.mon.as_ref().wy + c_ref.mon.as_ref().wh } - c_ref.height();
    }
    c_ref.x = c_ref.x.max(unsafe { c_ref.mon.as_ref() }.wx);
    c_ref.y = c_ref.y.max(unsafe { c_ref.mon.as_ref() }.wy);
    c_ref.bw = load_resource!("BORDER_PX", globals, Integer) as i32;

    wc.border_width = c_ref.bw;

    const CW_BORDER_WIDTH: u32 = 1 << 4;

    unsafe {
        XConfigureWindow(globals.dpy.as_ptr(), w, CW_BORDER_WIDTH, &mut wc);
        XSetWindowBorder(
            globals.dpy.as_ptr(),
            w,
            globals.scheme[SCHEME_STATE_NORM][drw::COL_BORDER].pixel,
        )
    };
    c_ref.configure(globals); /* propagates border_width, if size doesn't change */
    c_ref.updatewindowtype(globals);
    c_ref.updatesizehints(globals);
    c_ref.updatewmhints(globals);
    unsafe {
        XSelectInput(
            globals.dpy.as_ptr(),
            w,
            ENTER_WINDOW_MASK | FOCUS_CHANGE_MASK | PROPERTY_CHANGE_MASK | STRUCTURE_NOTIFY_MASK,
        )
    };
    c_ref.grabbuttons(false, globals);
    if !unsafe { c.as_ref() }.isfloating {
        unsafe { c.as_mut().oldstate = trans != 0 || c.as_ref().isfixed };
        unsafe { c.as_mut().isfloating = c.as_ref().oldstate };
    }
    if unsafe { c.as_ref() }.isfloating {
        unsafe { XRaiseWindow(globals.dpy.as_ptr(), c.as_ref().win) };
    }
    Client::attach(c);
    Client::attachstack(c);
    unsafe {
        XChangeProperty(
            globals.dpy.as_ptr(),
            globals.root,
            globals.netatom[NET_CLIENT_LIST],
            XA_WINDOW,
            32,
            PROP_MODE_APPEND,
            (&c.as_ref().win) as *const _ as *const u8,
            1,
        )
    };
    unsafe {
        XMoveResizeWindow(
            globals.dpy.as_ptr(),
            c.as_ref().win,
            c.as_ref().x + 2 * globals.sw,
            c.as_ref().y,
            c.as_ref().w as u32,
            c.as_ref().h as u32,
        ); /* some windows require this */
    }
    c_ref.setclientstate(NORMAL_STATE as i64, globals);
    if unsafe { c.as_ref() }.mon == globals.selmon {
        Client::unfocus(unsafe { globals.selmon.as_ref() }.sel, false, globals);
    }
    unsafe { c.as_mut().mon.as_mut() }.sel = Some(c);
    Monitor::arrange(Some(unsafe { c.as_ref() }.mon), globals);
    unsafe { XMapWindow(globals.dpy.as_ptr(), c.as_ref().win) };

    if let Some(term) = term {
        Client::swallow(term, c, globals);
    }

    Client::focus(None, globals);
}

fn updateclientlist(globals: &Globals) {
    unsafe {
        XDeleteProperty(
            globals.dpy.as_ptr(),
            globals.root,
            globals.netatom[NET_CLIENT_LIST],
        );
    }
    let mut m = Some(globals.mons);
    while let Some(m_inner) = m {
        let mut c = unsafe { m_inner.as_ref() }.clients;
        while let Some(c_inner) = c {
            unsafe {
                XChangeProperty(
                    globals.dpy.as_ptr(),
                    globals.root,
                    globals.netatom[NET_CLIENT_LIST],
                    XA_WINDOW,
                    32,
                    PROP_MODE_APPEND,
                    (&c_inner.as_ref().win) as *const _ as *const u8,
                    1,
                )
            };
            c = unsafe { c_inner.as_ref() }.next;
        }
        m = unsafe { m_inner.as_ref() }.next
    }
}

fn run(globals: &mut Globals) {
    let mut ev: XEvent = unsafe { core::mem::zeroed() };
    unsafe { XSync(globals.dpy.as_ptr(), 0) };
    while globals.running && unsafe { XNextEvent(globals.dpy.as_ptr(), &mut ev) } == 0 {
        if let Some(f) = event_handler(unsafe { ev.r#type }) {
            f(&mut ev, globals)
        }
    }
}

fn scan(globals: &mut Globals) {
    let mut num = 0u32;
    let mut d1: Window = 0;
    let mut d2: Window = 0;
    let mut wins: *mut Window = core::ptr::null_mut();
    let mut wa: XWindowAttributes = unsafe { std::mem::zeroed() };

    const IS_VIEWABLE: i32 = 2;
    const ICONIC_STATE: i64 = 3;

    if unsafe {
        XQueryTree(
            globals.dpy.as_ptr(),
            globals.root,
            &mut d1,
            &mut d2,
            &mut wins,
            &mut num,
        )
    } != 0
    {
        for i in 0..num as usize {
            if unsafe { XGetWindowAttributes(globals.dpy.as_ptr(), *wins.add(i), &mut wa) } == 0
                || wa.override_redirect != 0
                || unsafe { XGetTransientForHint(globals.dpy.as_ptr(), *wins.add(i), &mut d1) } != 0
            {
                continue;
            }
            if wa.map_state == IS_VIEWABLE
                || getstate(unsafe { *wins.add(i) }, globals) == ICONIC_STATE
            {
                manage(unsafe { *wins.add(i) }, &wa, globals);
            }
        }
        for i in 0..num as usize {
            /* now the transients */
            if unsafe { XGetWindowAttributes(globals.dpy.as_ptr(), *wins.add(i), &mut wa) } == 0 {
                continue;
            }
            if unsafe { XGetTransientForHint(globals.dpy.as_ptr(), *wins.add(i), &mut d1) } != 0
                && (wa.map_state == IS_VIEWABLE
                    || getstate(unsafe { *wins.add(i) }, globals) == ICONIC_STATE)
            {
                manage(unsafe { *wins.add(i) }, &wa, globals);
            }
        }
        if !wins.is_null() {
            unsafe { XFree(wins.cast()) };
        }
    }
}

fn setup(dpy: NonNull<Display>, resources: Resources, xcon: NonNull<xcb_connection_t>) -> Globals {
    /* do not transform children into zombies when they terminate */
    let mut sa: libc::sigaction = unsafe { std::mem::zeroed() };
    unsafe { libc::sigemptyset(&mut sa.sa_mask) };
    sa.sa_flags = libc::SA_NOCLDSTOP | libc::SA_NOCLDWAIT | libc::SA_RESTART;
    sa.sa_sigaction = libc::SIG_IGN;
    unsafe { libc::sigaction(libc::SIGCHLD, &sa, core::ptr::null_mut()) };

    /* clean up any zombies (inherited from .xinitrc etc) immediately */
    while unsafe { libc::waitpid(-1, core::ptr::null_mut(), libc::WNOHANG) } > 0 {}

    /* init screen */
    let screen = unsafe { default_screen(dpy.as_ptr()) };
    let sw = unsafe { default_width(dpy.as_ptr(), screen) };
    let sh = unsafe { default_height(dpy.as_ptr(), screen) };
    let root = unsafe { root_window(dpy.as_ptr(), screen) };

    let mut drw = Drw::new(dpy, screen, root, sw as u32, sh as u32);
    drw.fontset_create(config::FONTS);

    let Some(drw_fonts) = drw.fonts else {
        die!("no fonts could be loaded");
    };

    let lrpad = unsafe { drw_fonts.as_ref() }.h as i32;
    let bh = unsafe { drw_fonts.as_ref() }.h as i32 + 2;

    let mut globals = Globals {
        stext: [0; 256],
        screen,
        sw,
        sh,
        bh,
        lrpad,
        numlockmask: 0,
        wmatom: [0; WM_LAST],
        netatom: [0; NET_LAST],
        running: false,
        cursor: [Cur { cursor: 0 }; CURSOR_STATE_LAST],
        scheme: Vec::new().into_boxed_slice(),
        dpy,
        drw,
        mons: NonNull::dangling(),
        selmon: NonNull::dangling(),
        root,
        wmcheckwin: 0,
        last_motion_mon: None,
        resources,
        xcon,
        statusw: 0,
        statussig: 0,
        statuspid: -1 as libc::pid_t,
        enable_gaps: true,
    };

    updategeom(&mut globals);
    globals.running = true;

    let utf8string: Atom;

    unsafe {
        utf8string = XInternAtom(dpy.as_ptr(), c"UTF8_STRING".as_ptr(), 0);
        globals.wmatom[WM_PROTOCOLS] = XInternAtom(dpy.as_ptr(), c"WM_PROTOCOLS".as_ptr(), 0);
        globals.wmatom[WM_DELETE] = XInternAtom(dpy.as_ptr(), c"WM_DELETE_WINDOW".as_ptr(), 0);
        globals.wmatom[WM_STATE] = XInternAtom(dpy.as_ptr(), c"WM_STATE".as_ptr(), 0);
        globals.wmatom[WM_TAKE_FOCUS] = XInternAtom(dpy.as_ptr(), c"WM_TAKE_FOCUS".as_ptr(), 0);
        globals.netatom[NET_ACTIVE_WINDOW] =
            XInternAtom(dpy.as_ptr(), c"_NET_ACTIVE_WINDOW".as_ptr(), 0);
        globals.netatom[NET_SUPPORTED] = XInternAtom(dpy.as_ptr(), c"_NET_SUPPORTED".as_ptr(), 0);
        globals.netatom[NET_WM_NAME] = XInternAtom(dpy.as_ptr(), c"_NET_WM_NAME".as_ptr(), 0);
        globals.netatom[NET_WM_STATE] = XInternAtom(dpy.as_ptr(), c"_NET_WM_STATE".as_ptr(), 0);
        globals.netatom[NET_WM_CHECK] =
            XInternAtom(dpy.as_ptr(), c"_NET_SUPPORTING_WM_CHECK".as_ptr(), 0);
        globals.netatom[NET_WM_FULLSCREEN] =
            XInternAtom(dpy.as_ptr(), c"_NET_WM_STATE_FULLSCREEN".as_ptr(), 0);
        globals.netatom[NET_WM_STICKY] =
            XInternAtom(dpy.as_ptr(), c"_NET_WM_STATE_STICKY".as_ptr(), 0);
        globals.netatom[NET_WM_WINDOW_TYPE] =
            XInternAtom(dpy.as_ptr(), c"_NET_WM_WINDOW_TYPE".as_ptr(), 0);
        globals.netatom[NET_WM_WINDOW_TYPE_DIALOG] =
            XInternAtom(dpy.as_ptr(), c"_NET_WM_WINDOW_TYPE_DIALOG".as_ptr(), 0);
        globals.netatom[NET_CLIENT_LIST] =
            XInternAtom(dpy.as_ptr(), c"_NET_CLIENT_LIST".as_ptr(), 0);
    }

    globals.cursor[CURSOR_STATE_NORMAL] = globals.drw.cur_create(XC_LEFT_PTR);
    globals.cursor[CURSOR_STATE_RESIZE] = globals.drw.cur_create(XC_SIZING);
    globals.cursor[CURSOR_STATE_MOVE] = globals.drw.cur_create(XC_FLEUR);

    let mut scheme = Vec::new();
    for pallette in config::COLORS {
        let mut pallette_iter = pallette.iter().map(|name| {
            let ResourceVal::String(color) = globals
                .resources
                .get(name)
                .expect("Color is present in the resources map")
            else {
                die!("Color is not of type string in resoures map")
            };
            color.as_str()
        });
        let pallette: [&str; config::COLORS[0].len()] = std::array::from_fn(|_| {
            pallette_iter.next().expect(
                "we know by construction that there exists a constant number of values in the map",
            )
        });
        let scm = globals.drw.scm_create(&pallette);
        scheme.push(scm);
    }
    globals.scheme = scheme.into_boxed_slice();

    updatebars(&globals);
    updatestatus(&mut globals);

    let wmcheckwin = unsafe { XCreateSimpleWindow(dpy.as_ptr(), root, 0, 0, 1, 1, 0, 0, 0) };

    unsafe {
        XChangeProperty(
            dpy.as_ptr(),
            wmcheckwin,
            globals.netatom[NET_WM_CHECK],
            XA_WINDOW,
            32,
            PROP_MODE_REPLACE,
            (&wmcheckwin) as *const u64 as *const u8,
            1,
        )
    };
    unsafe {
        XChangeProperty(
            dpy.as_ptr(),
            wmcheckwin,
            globals.netatom[NET_WM_NAME],
            utf8string,
            8,
            PROP_MODE_REPLACE,
            c"dwm".as_ptr() as *const u8,
            3,
        )
    };
    unsafe {
        XChangeProperty(
            dpy.as_ptr(),
            root,
            globals.netatom[NET_WM_CHECK],
            XA_WINDOW,
            32,
            PROP_MODE_REPLACE,
            (&wmcheckwin) as *const u64 as *const u8,
            1,
        )
    };
    /* EWMH support per view */
    unsafe {
        XChangeProperty(
            dpy.as_ptr(),
            root,
            globals.netatom[NET_SUPPORTED],
            XA_ATOM,
            32,
            PROP_MODE_REPLACE,
            (&globals.netatom) as *const u64 as *const u8,
            NET_LAST as i32,
        )
    };
    unsafe { XDeleteProperty(dpy.as_ptr(), root, globals.netatom[NET_CLIENT_LIST]) };

    let mut wa: XSetWindowAttributes = unsafe { core::mem::zeroed() };

    wa.cursor = globals.cursor[CURSOR_STATE_NORMAL].cursor;
    wa.event_mask = SUBSTRUCTURE_REDIRECT_MASK
        | SUBSTRUCTURE_NOTIFY_MASK
        | BUTTON_PRESS_MASK
        | POINTER_MOTION_MASK
        | ENTER_WINDOW_MASK
        | LEAVE_WINDOW_MASK
        | STRUCTURE_NOTIFY_MASK
        | PROPERTY_CHANGE_MASK;
    unsafe {
        XChangeWindowAttributes(
            dpy.as_ptr(),
            globals.root,
            CW_EVENT_MASK | CW_CURSOR,
            &mut wa,
        )
    };
    unsafe { XSelectInput(dpy.as_ptr(), globals.root, wa.event_mask) };
    grabkeys(&mut globals);
    Client::focus(None, &mut globals);
    globals
}

// Cleanup deallocates everything:
//   - Clients:  each unmanage() calls Box::from_raw on the Client allocation.
//   - Monitors: each cleanupmon() calls Box::from_raw on the Monitor allocation.
//   - Cursors:  cur_free() calls XFreeCursor for each Cur.
//   - Schemes:  scm_free() calls XftColorFree for each Clr; the Rc<[Clr]> then drops.
//   - Fonts:    drop(drw) triggers Drw::drop → Box::from_raw(first Fnt) → Fnt::drop
//               which recursively drops every node via its `next` field, calling
//               XftFontClose + optional FcPatternDestroy on each.
//   - Drw:      Drw::drop frees the pixmap and GC.
//   - X state:  XDestroyWindow, XSync, XSetInputFocus, XDeleteProperty handled below.
fn cleanup(mut globals: Globals) -> *mut Display {
    let a = Arg::Ui(!0);
    const EMPTY_LAYOUT: Layout = Layout {
        symbol: "",
        arrange: None,
    };
    const ANY_KEY: i32 = 0;
    const ANY_MODIFIER: u32 = 1 << 15;
    const POINTER_ROOT: u64 = 1;

    a.view(&mut globals);
    (unsafe { globals.selmon.as_mut().lt })[unsafe { globals.selmon.as_ref().sellt } as usize] =
        &EMPTY_LAYOUT;

    //cleanup clients
    let mut m = Some(globals.mons);
    while let Some(m_inner) = m {
        while let Some(stack) = unsafe { m_inner.as_ref() }.stack {
            Client::unmanage(stack, false, &mut globals)
        }
        m = unsafe { m_inner.as_ref() }.next;
    }

    //cleanup monitors
    unsafe { XUngrabKey(globals.dpy.as_ptr(), ANY_KEY, ANY_MODIFIER, globals.root) };
    globals.selmon = NonNull::dangling(); // prevent use-after-free: monitors are freed next
    while !Monitor::cleanupmon(globals.mons, &mut globals) {}

    let Globals {
        cursor,
        scheme,
        dpy,
        mut drw,
        root,
        wmcheckwin,
        netatom,
        ..
    } = globals;

    // Cleanup cursors
    for cur in cursor {
        drw.cur_free(cur);
    }
    for scheme in scheme {
        drw.scm_free(scheme, true);
    }
    unsafe { XDestroyWindow(dpy.as_ptr(), wmcheckwin) };
    drop(drw);
    unsafe { XSync(dpy.as_ptr(), 0) };
    unsafe {
        XSetInputFocus(
            dpy.as_ptr(),
            POINTER_ROOT,
            REVERT_TO_POINTER_ROOT,
            CURRENT_TIME,
        )
    };
    unsafe { XDeleteProperty(dpy.as_ptr(), root, netatom[NET_ACTIVE_WINDOW]) };
    dpy.as_ptr()
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    if args.len() == 2 && args[1] == "-v" {
        die!("dwm-{}", VERSION);
    } else if args.len() != 1 {
        die!("usage: dwm [-v]");
    }

    if unsafe { libc::setlocale(libc::LC_CTYPE, c"".as_ptr()).is_null() }
        || unsafe { XSupportsLocale() } == 0
    {
        eprintln!("warning, no locale support");
    }

    let Some(dpy) = NonNull::new(unsafe { XOpenDisplay(core::ptr::null_mut()) }) else {
        die!("dwm: cannot open display");
    };

    let Some(xcon) = NonNull::new(unsafe { XGetXCBConnection(dpy.as_ptr()) }) else {
        die!("dwm: cannot get xcb connection\n");
    };

    checkotherwm(dpy);
    unsafe { XrmInitialize() };
    let resources = resource::load_xresources();
    let mut globals = setup(dpy, resources, xcon);
    scan(&mut globals);
    run(&mut globals);
    let dpy: *mut Display = cleanup(globals);
    unsafe { XCloseDisplay(dpy) };
}
