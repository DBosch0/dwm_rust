use std::ffi::{CStr, CString, c_int};
use std::io::{Write, stderr};
use std::mem::MaybeUninit;
use std::os::raw::c_void;
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub(crate) use crate::argument::Arg;
pub(crate) use crate::client::Client;
use crate::drw::{Clr, Cur, Drw};
use crate::external_functions::*;
pub(crate) use crate::monitor::Monitor;
pub(crate) use crate::resource::Resources;
use crate::resource::{borrow_resource, load_resource};
pub(crate) use monitor::layouts::Layout;

mod argument;
mod client;
mod config;
mod drw;
mod event;
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

// dwm is single-threaded; Relaxed ordering is sufficient.
// Written once in checkotherwm before any X error can occur;
// read only in xerror thereafter.
static XERRORXLIB: AtomicUsize = AtomicUsize::new(0);
type XErrorFunction = extern "C" fn(*mut Display, *mut XErrorEvent) -> c_int;

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

impl Globals {
    #[inline(always)]
    fn text_w(&mut self, x: *const i8) -> i32 {
        self.drw.fontset_getwidth(x) as i32 + self.lrpad
    }

    #[inline(always)]
    const fn cleanmask(&self, mask: u32) -> u32 {
        mask & !(self.numlockmask | LOCK_MASK)
            & (SHIFT_MASK
                | CONTROL_MASK
                | MOD1_MASK
                | MOD2_MASK
                | MOD3_MASK
                | MOD4_MASK
                | MOD5_MASK)
    }

    #[allow(dead_code)]
    pub(crate) fn setgaps(&mut self, mut oh: i32, mut ov: i32, mut ih: i32, mut iv: i32) {
        oh = oh.max(0);
        ov = ov.max(0);
        ih = ih.max(0);
        iv = iv.max(0);

        unsafe { self.selmon.as_mut() }.gappoh = oh;
        unsafe { self.selmon.as_mut() }.gappov = ov;
        unsafe { self.selmon.as_mut() }.gappih = ih;
        unsafe { self.selmon.as_mut() }.gappiv = iv;
        Monitor::arrange(Some(self.selmon), self);
    }

    fn winpid(&self, w: Window) -> libc::pid_t {
        let mut result: libc::pid_t = 0;
        const XCB_RES_CLIENT_ID_MASK_LOCAL_CLIENT_PID: u32 = 2;

        let mut spec = xcb_res_client_id_spec_t {
            client: w as u32,
            mask: XCB_RES_CLIENT_ID_MASK_LOCAL_CLIENT_PID,
        };

        let mut e: *mut xcb_generic_error_t = core::ptr::null_mut();
        let c: xcb_res_query_client_ids_cookie_t =
            unsafe { xcb_res_query_client_ids(self.xcon.as_ptr(), 1, &spec) };
        let r: *mut xcb_res_query_client_ids_reply_t =
            unsafe { xcb_res_query_client_ids_reply(self.xcon.as_ptr(), c, &mut e) };

        if r.is_null() {
            return 0 as libc::pid_t;
        }

        let mut i: xcb_res_client_id_value_iterator_t =
            unsafe { xcb_res_query_client_ids_ids_iterator(r) };
        while i.rem != 0 {
            spec = unsafe { &*i.data }.spec;
            if spec.mask & XCB_RES_CLIENT_ID_MASK_LOCAL_CLIENT_PID != 0 {
                let t = unsafe { xcb_res_client_id_value_value(i.data) };
                result = unsafe { *t } as libc::pid_t;
                break;
            }
            unsafe { xcb_res_client_id_value_next(&mut i) }
        }

        unsafe { libc::free(r as *mut c_void) };

        if result == (-1) as libc::pid_t {
            result = 0 as libc::pid_t;
        }
        result
    }

    fn getrootptr(&self, x: &mut i32, y: &mut i32) -> bool {
        let mut di: i32 = 0;
        let mut dui: u32 = 0;
        let mut dummy: Window = 0;

        (unsafe {
            XQueryPointer(
                self.dpy.as_ptr(),
                self.root,
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

    fn updategeom(&mut self) -> bool {
        let mut dirty = false;

        #[cfg(feature = "xinerama")]
        {
            todo!("feature: xinerama")
        }

        // We are in initialization
        if !self.running {
            self.mons = Monitor::createmon(self);
        }

        let m = unsafe { self.mons.as_mut() };
        if m.mw != self.sw || m.mh != self.sh {
            dirty = true;
            m.ww = self.sw;
            m.mw = m.ww;
            m.wh = self.sh;
            m.mh = m.wh;
            m.updatebarpos(self);
        }
        if dirty {
            self.selmon = self.mons;
            self.selmon = Monitor::wintomon(self.root, self);
        }

        dirty
    }

    fn updatebars(&self) {
        let mut wa: XSetWindowAttributes = unsafe { core::mem::zeroed() };
        wa.override_redirect = 1;
        wa.background_pixel = PARENT_RELATIVE;
        wa.event_mask = BUTTON_PRESS_MASK | EXPOSURE_MASK;

        let mut ch = XClassHint {
            res_name: c"dwm".as_ptr(),
            res_class: c"dwm".as_ptr(),
        };

        let mut m = Some(self.mons);
        while let Some(mut m_inner) = m {
            let m_ref = unsafe { m_inner.as_mut() };
            if m_ref.barwin > 0 {
                m = m_ref.next;
                continue;
            }
            m_ref.barwin = unsafe {
                XCreateWindow(
                    self.dpy.as_ptr(),
                    self.root,
                    m_ref.wx,
                    m_ref.by,
                    m_ref.ww as u32,
                    self.bh as u32,
                    0,
                    default_depth(self.dpy.as_ptr(), self.screen),
                    COPY_FROM_PARENT as u32,
                    default_visual(self.dpy.as_ptr(), self.screen),
                    CW_OVERRIDE_REDIRECT | CW_BACK_PIXMAP | CW_EVENT_MASK,
                    &mut wa,
                )
            };
            unsafe {
                XDefineCursor(
                    self.dpy.as_ptr(),
                    m_ref.barwin,
                    self.cursor[CURSOR_STATE_NORMAL].cursor,
                )
            };
            unsafe { XMapRaised(self.dpy.as_ptr(), m_ref.barwin) };
            unsafe { XSetClassHint(self.dpy.as_ptr(), m_ref.barwin, &mut ch) };
            m = m_ref.next;
        }
    }

    fn gettextprop(&self, w: Window, atom: Atom, text: *mut i8, size: u32) -> bool {
        let mut list: *mut *mut i8 = core::ptr::null_mut();
        let mut n = 0;
        let mut name: MaybeUninit<XTextProperty> = MaybeUninit::uninit();

        if text.is_null() || size == 0 {
            return false;
        }

        unsafe { *text = b'\0' as i8 };
        let get_text_property_result = unsafe {
            XGetTextProperty(
                self.dpy.as_ptr(),
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
        } else if unsafe { XmbTextPropertyToTextList(self.dpy.as_ptr(), &name, &mut list, &mut n) }
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

    fn drawbars(&mut self) {
        let mut m = Some(self.mons);
        while let Some(m_inner) = m {
            let mr = unsafe { m_inner.as_ref() };
            mr.drawbar(self);
            m = mr.next;
        }
    }

    fn getstatusbarpid(&mut self) -> libc::pid_t {
        let mut buf = [0i8; 32];
        let mut s = buf.as_mut_ptr();
        let mut fp: *mut libc::FILE;

        if self.statuspid > 0 {
            let cstr =
                CString::new(format!("/proc/{}/cmdline", self.statuspid)).expect("valid CString");
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
                    return self.statuspid;
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

    fn updatestatus(&mut self) {
        let stext_ptr = self.stext.as_mut_ptr();
        if !self.gettextprop(self.root, XA_WM_NAME, stext_ptr, self.stext.len() as u32) {
            unsafe {
                libc::strcpy(
                    self.stext.as_mut_ptr(),
                    CString::new(format!("dwm-{}", VERSION))
                        .expect("valid C string")
                        .as_ptr(),
                )
            };
            self.statusw = self.text_w(stext_ptr) - self.lrpad + 2;
        } else {
            self.statusw = 0;
            let mut text = self.stext.as_ptr();
            let mut s = self.stext.as_mut_ptr();
            while unsafe { *s } != 0 {
                if (unsafe { *s } as u8) < b' ' {
                    let ch = unsafe { *s };
                    unsafe { *s = '\0' as i8 };
                    self.statusw += self.text_w(text) - self.lrpad;
                    unsafe { *s = ch };
                    text = unsafe { s.add(1) };
                }
                s = unsafe { s.add(1) };
            }
            self.statusw += self.text_w(text) - self.lrpad + 2;
        }

        unsafe { self.selmon.as_ref() }.drawbar(self);
    }

    fn updatenumlockmask(&mut self) {
        const XK_NUM_LOCK: u64 = 0xff7f;

        self.numlockmask = 0;
        let modmap = unsafe { XGetModifierMapping(self.dpy.as_ptr()) };

        for i in 0..8 {
            for j in 0..unsafe { &*modmap }.max_keypermod {
                if unsafe {
                    *{ &*modmap }
                        .modifiermap
                        .add((i * { &*modmap }.max_keypermod + j) as usize)
                } == unsafe { XKeysymToKeycode(self.dpy.as_ptr(), XK_NUM_LOCK) }
                {
                    self.numlockmask = 1 << i;
                }
            }
        }
        unsafe { XFreeModifiermap(modmap) };
    }

    fn grabkeys(&mut self) {
        self.updatenumlockmask();
        {
            let modifiers = [0, LOCK_MASK, self.numlockmask, self.numlockmask | LOCK_MASK];

            let mut start = 0;
            let mut end = 0;
            let mut skip = 0;

            const ANY_KEY: i32 = 0;

            unsafe { XUngrabKey(self.dpy.as_ptr(), ANY_KEY, ANY_MODIFIER, self.root) };
            unsafe { XDisplayKeycodes(self.dpy.as_ptr(), &mut start, &mut end) };
            let syms = unsafe {
                XGetKeyboardMapping(self.dpy.as_ptr(), start as u8, end - start + 1, &mut skip)
            };
            if syms.is_null() {
                return;
            }

            for k in start..=end {
                for key in config::KEYS {
                    /* skip modifier codes, we do that ourselves */
                    if key.keysym == unsafe { *syms.add((k - start) as usize * skip as usize) } {
                        for modi in modifiers {
                            unsafe {
                                XGrabKey(
                                    self.dpy.as_ptr(),
                                    k,
                                    key.r#mod | modi,
                                    self.root,
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

    fn getstate(&self, w: Window) -> i64 {
        let mut format: i32 = 0;
        let mut result = -1i64;
        let mut p: *mut u8 = core::ptr::null_mut();
        let mut n = 0u64;
        let mut extra = 0u64;
        let mut real: Atom = 0;

        if unsafe {
            XGetWindowProperty(
                self.dpy.as_ptr(),
                w,
                self.wmatom[WM_STATE],
                0,
                2,
                0,
                self.wmatom[WM_STATE],
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

    fn manage(&mut self, w: Window, wa: &XWindowAttributes) {
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
            pid: self.winpid(w),
            swallowing: None,
        })))
        .expect("valid box construction");

        let c_ref = unsafe { c.as_mut() };
        let mut term: Option<NonNull<Client>> = None;

        c_ref.updatetitle(self);

        if unsafe { XGetTransientForHint(self.dpy.as_ptr(), w, &mut trans) } != 0
            && let Some(t) = Client::wintoclient(trans, self)
        {
            let t_ref = unsafe { t.as_ref() };
            c_ref.mon = t_ref.mon;
            c_ref.tags = t_ref.tags;
        } else {
            c_ref.mon = self.selmon;
            c_ref.applyrules(self);
            term = c_ref.termforwin(self);
        }

        let c_ref_mon = unsafe { c_ref.mon.as_ref() };
        if c_ref.x + c_ref.width() > c_ref_mon.wx + c_ref_mon.ww {
            c_ref.x = c_ref_mon.wx + c_ref_mon.ww - c_ref.width();
        }
        if c_ref.y + c_ref.height() > c_ref_mon.wy + c_ref_mon.wh {
            c_ref.y = c_ref_mon.wy + c_ref_mon.wh - c_ref.height();
        }
        c_ref.x = c_ref.x.max(c_ref_mon.wx);
        c_ref.y = c_ref.y.max(c_ref_mon.wy);
        c_ref.bw = load_resource!("BORDER_PX", self, Integer) as i32;

        wc.border_width = c_ref.bw;

        const CW_BORDER_WIDTH: u32 = 1 << 4;

        unsafe {
            XConfigureWindow(self.dpy.as_ptr(), w, CW_BORDER_WIDTH, &mut wc);
            XSetWindowBorder(
                self.dpy.as_ptr(),
                w,
                self.scheme[SCHEME_STATE_NORM][drw::COL_BORDER].pixel,
            )
        };
        c_ref.configure(self); /* propagates border_width, if size doesn't change */
        c_ref.updatewindowtype(self);
        c_ref.updatesizehints(self);
        c_ref.updatewmhints(self);
        unsafe {
            XSelectInput(
                self.dpy.as_ptr(),
                w,
                ENTER_WINDOW_MASK
                    | FOCUS_CHANGE_MASK
                    | PROPERTY_CHANGE_MASK
                    | STRUCTURE_NOTIFY_MASK,
            )
        };
        c_ref.grabbuttons(false, self);
        if !c_ref.isfloating {
            c_ref.oldstate = trans != 0 || c_ref.isfixed;
            c_ref.isfloating = c_ref.oldstate;
        }
        if c_ref.isfloating {
            unsafe { XRaiseWindow(self.dpy.as_ptr(), c_ref.win) };
        }
        Client::attach(c);
        Client::attachstack(c);
        unsafe {
            XChangeProperty(
                self.dpy.as_ptr(),
                self.root,
                self.netatom[NET_CLIENT_LIST],
                XA_WINDOW,
                32,
                PROP_MODE_APPEND,
                (&c_ref.win) as *const _ as *const u8,
                1,
            )
        };
        unsafe {
            XMoveResizeWindow(
                self.dpy.as_ptr(),
                c_ref.win,
                c_ref.x + 2 * self.sw,
                c_ref.y,
                c_ref.w as u32,
                c_ref.h as u32,
            ); /* some windows require this */
        }
        c_ref.setclientstate(NORMAL_STATE as i64, self);
        if c_ref.mon == self.selmon {
            Client::unfocus(unsafe { self.selmon.as_ref() }.sel, false, self);
        }
        unsafe { c_ref.mon.as_mut() }.sel = Some(c);
        Monitor::arrange(Some(c_ref.mon), self);
        unsafe { XMapWindow(self.dpy.as_ptr(), c_ref.win) };

        if let Some(term) = term {
            Client::swallow(term, c, self);
        }

        Client::focus(None, self);
    }

    fn updateclientlist(&self) {
        unsafe {
            XDeleteProperty(self.dpy.as_ptr(), self.root, self.netatom[NET_CLIENT_LIST]);
        }
        let mut m = Some(self.mons);
        while let Some(m_inner) = m {
            let mr = unsafe { m_inner.as_ref() };
            let mut c = unsafe { m_inner.as_ref() }.clients;
            while let Some(c_inner) = c {
                let cr = unsafe { c_inner.as_ref() };
                unsafe {
                    XChangeProperty(
                        self.dpy.as_ptr(),
                        self.root,
                        self.netatom[NET_CLIENT_LIST],
                        XA_WINDOW,
                        32,
                        PROP_MODE_APPEND,
                        (&cr.win) as *const _ as *const u8,
                        1,
                    )
                };
                c = cr.next;
            }
            m = mr.next
        }
    }

    fn run(&mut self) {
        let mut ev: XEvent = unsafe { core::mem::zeroed() };
        unsafe { XSync(self.dpy.as_ptr(), 0) };
        while self.running && unsafe { XNextEvent(self.dpy.as_ptr(), &mut ev) } == 0 {
            if let Some(event_handler_function) = event::event_handler(unsafe { ev.r#type }) {
                event_handler_function(&mut ev, self)
            }
        }
    }

    fn scan(&mut self) {
        let mut num = 0u32;
        let mut d1: Window = 0;
        let mut d2: Window = 0;
        let mut wins: *mut Window = core::ptr::null_mut();
        let mut wa: XWindowAttributes = unsafe { core::mem::zeroed() };

        const IS_VIEWABLE: i32 = 2;
        const ICONIC_STATE: i64 = 3;

        if unsafe {
            XQueryTree(
                self.dpy.as_ptr(),
                self.root,
                &mut d1,
                &mut d2,
                &mut wins,
                &mut num,
            )
        } != 0
        {
            for i in 0..num as usize {
                if unsafe { XGetWindowAttributes(self.dpy.as_ptr(), *wins.add(i), &mut wa) } == 0
                    || wa.override_redirect != 0
                    || unsafe { XGetTransientForHint(self.dpy.as_ptr(), *wins.add(i), &mut d1) }
                        != 0
                {
                    continue;
                }
                if wa.map_state == IS_VIEWABLE
                    || self.getstate(unsafe { *wins.add(i) }) == ICONIC_STATE
                {
                    self.manage(unsafe { *wins.add(i) }, &wa);
                }
            }
            for i in 0..num as usize {
                /* now the transients */
                if unsafe { XGetWindowAttributes(self.dpy.as_ptr(), *wins.add(i), &mut wa) } == 0 {
                    continue;
                }
                if unsafe { XGetTransientForHint(self.dpy.as_ptr(), *wins.add(i), &mut d1) } != 0
                    && (wa.map_state == IS_VIEWABLE
                        || self.getstate(unsafe { *wins.add(i) }) == ICONIC_STATE)
                {
                    self.manage(unsafe { *wins.add(i) }, &wa);
                }
            }
            if !wins.is_null() {
                unsafe { XFree(wins.cast()) };
            }
        }
    }

    fn setup(dpy: NonNull<Display>, resources: Resources, xcon: NonNull<xcb_connection_t>) -> Self {
        /* do not transform children into zombies when they terminate */
        let mut sa: libc::sigaction = unsafe { core::mem::zeroed() };
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

        // Sets values for mons and selmon
        globals.updategeom();
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
            globals.netatom[NET_SUPPORTED] =
                XInternAtom(dpy.as_ptr(), c"_NET_SUPPORTED".as_ptr(), 0);
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

        let mut scheme = Vec::with_capacity(config::COLORS.len());
        for pallette in config::COLORS {
            let mut pallette_iter = pallette
                .iter()
                .map(|name| borrow_resource!(name, globals, String).as_str());
            let pallette: [&str; config::COLORS[0].len()] = std::array::from_fn(|_| {
                pallette_iter.next().expect(
                "we know by construction that there exists a constant number of values in the map",
            )
            });
            let scm = globals.drw.scm_create(&pallette);
            scheme.push(scm);
        }
        globals.scheme = scheme.into_boxed_slice();

        globals.updatebars();
        globals.updatestatus();

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
            );
            XChangeProperty(
                dpy.as_ptr(),
                wmcheckwin,
                globals.netatom[NET_WM_NAME],
                utf8string,
                8,
                PROP_MODE_REPLACE,
                c"dwm".as_ptr() as *const u8,
                3,
            );
            XChangeProperty(
                dpy.as_ptr(),
                root,
                globals.netatom[NET_WM_CHECK],
                XA_WINDOW,
                32,
                PROP_MODE_REPLACE,
                (&wmcheckwin) as *const u64 as *const u8,
                1,
            );
            /* EWMH support per view */
            XChangeProperty(
                dpy.as_ptr(),
                root,
                globals.netatom[NET_SUPPORTED],
                XA_ATOM,
                32,
                PROP_MODE_REPLACE,
                (&globals.netatom) as *const u64 as *const u8,
                NET_LAST as i32,
            );
            XDeleteProperty(dpy.as_ptr(), root, globals.netatom[NET_CLIENT_LIST])
        };

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
        globals.grabkeys();
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
    fn cleanup(mut self) -> *mut Display {
        let a = Arg::Ui(!0);
        const EMPTY_LAYOUT: Layout = Layout {
            symbol: "",
            arrange: None,
        };
        const ANY_KEY: i32 = 0;
        const ANY_MODIFIER: u32 = 1 << 15;
        const POINTER_ROOT: u64 = 1;

        a.view(&mut self);
        let selmon = unsafe { self.selmon.as_mut() };
        selmon.lt[selmon.sellt as usize] = &EMPTY_LAYOUT;

        //cleanup clients
        let mut m = Some(self.mons);
        while let Some(m_inner) = m {
            let mr = unsafe { m_inner.as_ref() };
            while let Some(stack) = mr.stack {
                Client::unmanage(stack, false, &mut self)
            }
            m = mr.next;
        }

        //cleanup monitors
        unsafe { XUngrabKey(self.dpy.as_ptr(), ANY_KEY, ANY_MODIFIER, self.root) };
        self.selmon = NonNull::dangling(); // prevent use-after-free: monitors are freed next
        while !Monitor::cleanupmon(self.mons, &mut self) {}

        let Globals {
            cursor,
            scheme,
            dpy,
            mut drw,
            root,
            wmcheckwin,
            netatom,
            ..
        } = self;

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
    let mut globals = Globals::setup(dpy, resources, xcon);
    globals.scan();
    globals.run();
    let dpy: *mut Display = globals.cleanup();
    unsafe { XCloseDisplay(dpy) };
}
