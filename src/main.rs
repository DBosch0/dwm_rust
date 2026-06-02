use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::io::{Write, stderr};
use std::mem::MaybeUninit;
use std::os::raw::c_void;
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::drw::{Clr, Cur, Drw};
use crate::external_functions::*;

mod config;
mod drw;
mod external_functions;
mod util;
mod vanitygaps;

const VERSION: &str = "0.0.1";
const NUMTAGS: u32 = (config::TAGS.len() + config::SCRATCHPADS.len()) as u32;
const TAGMASK: u32 = (1 << NUMTAGS) - 1;
const BUTTON_MASK: i64 = BUTTON_PRESS_MASK | BUTTON_RELEASE_MASK;
const SPTAGMASK: u32 = ((1 << config::SCRATCHPADS.len()) as u32 - 1) << config::TAGS.len() as u32;
const MOUSE_MASK: i64 = BUTTON_MASK | POINTER_MOTION_MASK;
const PREV_SEL: i32 = 3000;

enum CursorState {
    Normal = 0,
    Resize,
    Move,
    Last,
}

enum SchemeState {
    Norm = 0,
    Sel,
}

enum NetAtom {
    Supported = 0,
    WMName,
    WMState,
    WMCheck,
    WMFullscreen,
    WMSticky,
    ActiveWindow,
    WMWindowType,
    WMWindowTypeDialog,
    ClientList,
    Last,
}

enum WMAtom {
    Protocols = 0,
    Delete,
    State,
    TakeFocus,
    Last,
}

#[derive(PartialEq, Eq)]
enum ClickState {
    TagBar,
    LtSymbol,
    StatusText,
    WinTitle,
    ClientWin,
    RootWin,
    // Last,
}

enum Arg {
    I(i32),
    Ui(u32),
    F(f32),
    Command(&'static [&'static str]),
    Layout(Option<&'static Layout>),
}

struct Button {
    click: ClickState,
    mask: u32,
    button: u32,
    func: Option<fn(arg: &Arg, globals: &mut Globals)>,
    arg: Arg,
}

struct Client {
    name: [i8; 256],
    mina: f32,
    maxa: f32,
    cfact: f32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    oldx: i32,
    oldy: i32,
    oldw: i32,
    oldh: i32,
    basew: i32,
    baseh: i32,
    incw: i32,
    inch: i32,
    maxw: i32,
    maxh: i32,
    minw: i32,
    minh: i32,
    hintsvalid: bool,
    bw: i32,
    oldbw: i32,
    tags: u32,
    isfixed: bool,
    isfloating: bool,
    isurgent: bool,
    neverfocus: bool,
    oldstate: bool,
    isfullscreen: bool,
    isterminal: bool,
    noswallow: bool,
    issticky: bool,
    pid: libc::pid_t,
    next: Option<NonNull<Client>>,
    snext: Option<NonNull<Client>>,
    swallowing: Option<NonNull<Client>>,
    mon: NonNull<Monitor>,
    win: Window,
}

impl Client {
    fn width(&self) -> i32 {
        self.w + 2 * self.bw
    }

    fn height(&self) -> i32 {
        self.h + 2 * self.bw
    }
}

struct Key {
    r#mod: u32,
    keysym: KeySym,
    func: Option<fn(&Arg, &mut Globals)>,
    arg: Arg,
}

struct Layout {
    symbol: &'static str,
    arrange: Option<fn(&mut Monitor, &mut Globals)>,
}

#[derive(Debug)]
enum ResourceVal {
    String(String),
    Integer(u32),
    Bool(bool),
    Float(f32),
}

#[derive(Debug)]
enum ResourceValConfig {
    String(&'static str),
    Integer(u32),
    Bool(bool),
    Float(f32),
}

impl ResourceValConfig {
    fn to_resource_val(&self) -> ResourceVal {
        match self {
            ResourceValConfig::String(s) => ResourceVal::String((*s).to_owned()),
            ResourceValConfig::Integer(i) => ResourceVal::Integer(*i),
            ResourceValConfig::Bool(b) => ResourceVal::Bool(*b),
            ResourceValConfig::Float(f) => ResourceVal::Float(*f),
        }
    }
}

#[derive(Debug)]
struct ResourceConfig {
    name: &'static str,
    x_resource_name: &'static str,
    default_value: ResourceValConfig,
}

type Resources = HashMap<&'static str, ResourceVal>;

struct ScratchPad {
    name: &'static str,
    cmd: &'static [&'static str],
}

struct Monitor {
    ltsymbol: [i8; 16],
    mfact: f32,
    nmaster: i32,
    num: i32,
    by: i32, //bar geometry
    mx: i32, //screen size
    my: i32,
    mw: i32,
    mh: i32,
    wx: i32, //window area
    wy: i32,
    ww: i32,
    wh: i32,
    gappih: i32, /* horizontal gap between windows */
    gappiv: i32, /* vertical gap between windows */
    gappoh: i32, /* horizontal outer gaps */
    gappov: i32, /* vertical outer gaps */
    seltags: u32,
    sellt: u32,
    tagset: [u32; 2],
    showbar: bool,
    topbar: bool,
    clients: Option<NonNull<Client>>,
    sel: Option<NonNull<Client>>,
    stack: Option<NonNull<Client>>,
    next: Option<NonNull<Monitor>>,
    barwin: Window,
    lt: [&'static Layout; 2],
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

//HELPERS:
macro_rules! intersect {
    ($t:ty, $x:ident, $y:ident, $w:ident, $h:ident, $m:ident) => {{
        <$t>::max(0, <$t>::min($x + $w, $m.wx + $m.ww) - <$t>::max($x, $m.wx))
            * <$t>::max(0, <$t>::min($y + $h, $m.wy + $m.wh) - <$t>::max($y, $m.wy))
    }};
}

#[inline(always)]
fn is_visible(c: NonNull<Client>) -> bool {
    let c_ref = unsafe { c.as_ref() };
    let m_ref = unsafe { c_ref.mon.as_ref() };
    c_ref.tags & m_ref.tagset[m_ref.seltags as usize] != 0 || c_ref.issticky
}

#[inline(always)]
fn text_w(x: *const i8, globals: &mut Globals) -> i32 {
    globals.drw.fontset_getwidth(x) as i32 + globals.lrpad
}

#[inline(always)]
fn cleanmask(mask: u32, globals: &Globals) -> u32 {
    mask & !(globals.numlockmask | LOCK_MASK)
        & (SHIFT_MASK | CONTROL_MASK | MOD1_MASK | MOD2_MASK | MOD3_MASK | MOD4_MASK | MOD5_MASK)
}

#[inline(always)]
const fn sptag(i: u32) -> u32 {
    (1 << config::TAGS.len() as u32) << i
}

#[macro_export]
macro_rules! load_resource {
    ($name:expr, $globals:expr, $variant:ident) => {{
        let crate::ResourceVal::$variant(value) = $globals
            .resources
            .get($name)
            .unwrap_or_else(|| panic!("{} is not in the resources map", $name))
        else {
            unreachable!("invalid type of variable {} in Resources map", $name);
        };
        *value
    }};
}

type HandlerFunction = fn(&mut XEvent, &mut Globals);

// dwm is single-threaded; Relaxed ordering is sufficient.
// Written once in checkotherwm before any X error can occur;
// read only in xerror thereafter.
// TODO: move into globals?
static XERRORXLIB: AtomicUsize = AtomicUsize::new(0);
const BROKEN: &CStr = c"broken";
// Indexed by X11 event type (0..LAST_EVENT). X11 event types start at 2;
// indices 0 and 1 are unused. This matches the C designated-initializer table:
//   static void (*handler[LASTEvent])(XEvent *) = { [ButtonPress]=buttonpress, ... }
// LAST_EVENT=36, so the array has 36 entries covering types 0-35.
const HANDLER: [Option<HandlerFunction>; LAST_EVENT as usize] = [
    None,                   // 0  — unused
    None,                   // 1  — unused
    Some(keypress),         // 2  KeyPress
    None,                   // 3  KeyRelease
    Some(buttonpress),      // 4  ButtonPress
    None,                   // 5  ButtonRelease
    Some(motionnotify),     // 6  MotionNotify
    Some(enternotify),      // 7  EnterNotify
    None,                   // 8  LeaveNotify
    Some(focusin),          // 9  FocusIn
    None,                   // 10 FocusOut
    None,                   // 11 KeymapNotify
    Some(expose),           // 12 Expose
    None,                   // 13 GraphicsExpose
    None,                   // 14 NoExpose
    None,                   // 15 VisibilityNotify
    None,                   // 16 CreateNotify
    Some(destroynotify),    // 17 DestroyNotify
    Some(unmapnotify),      // 18 UnmapNotify
    None,                   // 19 MapNotify
    Some(maprequest),       // 20 MapRequest
    None,                   // 21 ReparentNotify
    Some(configurenotify),  // 22 ConfigureNotify
    Some(configurerequest), // 23 ConfigureRequest
    None,                   // 24 GravityNotify
    None,                   // 25 ResizeRequest
    None,                   // 26 CirculateNotify
    None,                   // 27 CirculateRequest
    Some(propertynotify),   // 28 PropertyNotify
    None,                   // 29 SelectionClear
    None,                   // 30 SelectionRequest
    None,                   // 31 SelectionNotify
    None,                   // 32 ColormapNotify
    Some(clientmessage),    // 33 ClientMessage
    Some(mappingnotify),    // 34 MappingNotify
    None,                   // 35 GenericEvent
];

#[derive(Debug)]
struct Globals {
    stext: [i8; 256],
    screen: i32,
    sw: i32, /* X display screen geometry width, height */
    sh: i32,
    bh: i32,    /* bar height */
    lrpad: i32, /* sum of left and right padding for text */
    numlockmask: u32,
    wmatom: [Atom; WMAtom::Last as usize],
    netatom: [Atom; NetAtom::Last as usize],
    running: bool,
    cursor: [Cur; CursorState::Last as usize],
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
}

fn nexttiled(mut c: Option<NonNull<Client>>) -> Option<NonNull<Client>> {
    while let Some(c_inner) = c
        && (unsafe { c_inner.as_ref() }.isfloating || !is_visible(c_inner))
    {
        c = unsafe { c_inner.as_ref() }.next;
    }
    c
}

fn monocle(m: &mut Monitor, globals: &mut Globals) {
    let mut n = 0;
    let mut c = m.clients;
    while let Some(c_inner) = c {
        if is_visible(c_inner) {
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
    let mut c = nexttiled(m.clients);
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
        c = nexttiled(next);
    }
}

fn view(arg: &Arg, globals: &mut Globals) {
    let Arg::Ui(ui) = arg else { unreachable!() };
    let cur = unsafe { globals.selmon.as_ref() }.tagset
        [unsafe { globals.selmon.as_ref() }.seltags as usize];
    if *ui & TAGMASK == cur {
        return;
    }
    unsafe { globals.selmon.as_mut() }.seltags ^= 1; /* toggle sel tagset */
    if *ui & TAGMASK != 0 {
        (unsafe { globals.selmon.as_mut() }.tagset)
            [unsafe { globals.selmon.as_ref() }.seltags as usize] = *ui & TAGMASK;
    }
    focus(None, globals);
    arrange(Some(globals.selmon), globals);
}

fn toggleview(arg: &Arg, globals: &mut Globals) {
    let Arg::Ui(ui) = arg else { unreachable!() };

    let newtagset = unsafe { globals.selmon.as_ref() }.tagset
        [unsafe { globals.selmon.as_ref() }.seltags as usize]
        ^ (*ui & TAGMASK);

    if newtagset != 0 {
        unsafe { globals.selmon.as_mut() }.tagset
            [unsafe { globals.selmon.as_ref() }.seltags as usize] = newtagset;
        focus(None, globals);
        arrange(Some(globals.selmon), globals);
    }
}

fn tag(arg: &Arg, globals: &mut Globals) {
    let Arg::Ui(ui) = arg else { unreachable!() };
    if let Some(mut sel) = unsafe { globals.selmon.as_ref() }.sel
        && *ui & TAGMASK != 0
    {
        unsafe { sel.as_mut() }.tags = *ui & TAGMASK;
        focus(None, globals);
        arrange(Some(globals.selmon), globals);
    }
}

fn togglesticky(_arg: &Arg, globals: &mut Globals) {
    if let Some(mut sel) = unsafe { globals.selmon.as_ref() }.sel {
        setsticky(
            unsafe { sel.as_mut() },
            !unsafe { sel.as_ref() }.issticky,
            globals,
        );
        arrange(Some(globals.selmon), globals);
    }
}

fn toggletag(arg: &Arg, globals: &mut Globals) {
    let Arg::Ui(ui) = arg else { unreachable!() };
    if let Some(mut sel) = unsafe { globals.selmon.as_ref() }.sel {
        let newtags = unsafe { sel.as_ref() }.tags ^ (*ui & TAGMASK);
        if newtags != 0 {
            unsafe { sel.as_mut() }.tags = newtags;
            focus(None, globals);
            arrange(Some(globals.selmon), globals);
        }
    }
}

fn togglescratch(arg: &Arg, globals: &mut Globals) {
    let mut found = false;
    let Arg::Ui(ui) = arg else {
        unreachable!("invalid argument given to togglescratch function")
    };
    let scratchtag = sptag(*ui);
    let sparg = Arg::Command(config::SCRATCHPADS[*ui as usize].cmd);

    let mut c = unsafe { globals.selmon.as_ref().clients };
    while let Some(ci) = c {
        found = unsafe { ci.as_ref().tags } & scratchtag != 0;
        if found {
            break;
        }
        c = unsafe { ci.as_ref().next }
    }
    if found {
        let Some(c) = c else {
            unreachable!("we are the the found branch")
        };
        let tagset = unsafe { globals.selmon.as_ref().tagset }
            [unsafe { globals.selmon.as_ref().seltags } as usize];
        let newtagset = tagset ^ scratchtag;
        if newtagset != 0 {
            let seltags_idx = unsafe { globals.selmon.as_ref().seltags } as usize;
            unsafe { globals.selmon.as_mut().tagset[seltags_idx] = newtagset };
            focus(None, globals);
            arrange(Some(globals.selmon), globals);
        }
        if is_visible(c) {
            focus(Some(c), globals);
            restack(unsafe { globals.selmon.as_ref() }, globals);
        }
    } else {
        let seltags_idx = unsafe { globals.selmon.as_ref().seltags } as usize;
        unsafe { globals.selmon.as_mut().tagset[seltags_idx] |= scratchtag };
        spawn(&sparg, globals);
    }
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

fn termforwin(w: NonNull<Client>, globals: &Globals) -> Option<NonNull<Client>> {
    if unsafe { w.as_ref().pid } == 0 || unsafe { w.as_ref().isterminal } {
        return None;
    }

    let mut m = Some(globals.mons);
    while let Some(mi) = m {
        let mut c = unsafe { mi.as_ref().clients };
        while let Some(ci) = c {
            if unsafe { ci.as_ref().isterminal }
                && unsafe { ci.as_ref().swallowing.is_none() }
                && unsafe { ci.as_ref().pid } != 0
                && isdescprocess(unsafe { ci.as_ref().pid }, unsafe { w.as_ref().pid }) != 0
            {
                return c;
            }
            c = unsafe { ci.as_ref().next }
        }
        m = unsafe { mi.as_ref().next };
    }

    None
}

fn swallowingclient(w: Window, globals: &Globals) -> Option<NonNull<Client>> {
    let mut m = Some(globals.mons);
    while let Some(mi) = m {
        let mut c = unsafe { mi.as_ref().clients };
        while let Some(ci) = c {
            if let Some(swallowing) = unsafe { ci.as_ref().swallowing }
                && unsafe { swallowing.as_ref().win } == w
            {
                return c;
            }
            c = unsafe { ci.as_ref().next }
        }
        m = unsafe { mi.as_ref().next };
    }
    None
}

//NOTE: using libc and not `std::process` because setsid in `std::os::linux::process::CommandExt` is unstable.
//update when stable.
fn spawn(arg: &Arg, globals: &mut Globals) {
    let mut sa: libc::sigaction = unsafe { core::mem::zeroed() };
    let Arg::Command(arr) = arg else {
        unreachable!("invalid argument for spawn function")
    };

    let mon_num = unsafe { globals.selmon.as_ref() }.num;
    let cs_arr: Vec<CString> = arr
        .iter()
        .map(|&elem| {
            let s = if let Some(elem_stripped) = elem.strip_prefix("__") {
                if elem_stripped == "DMENU_MONITOR_PLACEHOLDER" {
                    format!("{}", mon_num)
                } else {
                    let Some(e) = globals.resources.get(elem_stripped) else {
                        die!("Tried to load placeholder not in the resources map");
                    };
                    let ResourceVal::String(s) = e else {
                        die!("Non String Resouce Values are not currently implemented for `spawn`")
                    };
                    s.clone()
                }
            } else {
                elem.to_string()
            };
            CString::new(s).expect("valid CStr")
        })
        .collect();
    let mut com: Vec<*const i8> = cs_arr.iter().map(|cs| cs.as_ptr()).collect();
    com.push(core::ptr::null()); // null terminator required by execvp

    if unsafe { libc::fork() } == 0 {
        // C dwm guards this with `if (dpy)`, but in Rust globals.dpy is NonNull<Display>,
        // guaranteed non-null by construction. If we reach spawn, setup() succeeded and dpy
        // is always valid, so the check is unnecessary.
        unsafe { libc::close(connection_number(globals.dpy.as_ptr())) };

        unsafe { libc::setsid() };
        unsafe { libc::sigemptyset(&mut sa.sa_mask) };
        sa.sa_flags = 0;
        sa.sa_sigaction = libc::SIG_DFL;
        unsafe { libc::sigaction(libc::SIGCHLD, &sa, core::ptr::null_mut()) };

        unsafe {
            libc::execvp(com[0], com.as_ptr());
        }
        die!("dwm: execvp failed");
    }
}

fn setsticky(c: &mut Client, sticky: bool, globals: &mut Globals) {
    if sticky && !c.issticky {
        unsafe {
            XChangeProperty(
                globals.dpy.as_ptr(),
                c.win,
                globals.netatom[NetAtom::WMState as usize],
                XA_ATOM,
                32,
                PROP_MODE_REPLACE,
                &globals.netatom[NetAtom::WMSticky as usize] as *const u64 as *const u8,
                1,
            );
        }
        c.issticky = true;
    } else if !sticky && c.issticky {
        unsafe {
            XChangeProperty(
                globals.dpy.as_ptr(),
                c.win,
                globals.netatom[NetAtom::WMState as usize],
                XA_ATOM,
                32,
                PROP_MODE_REPLACE,
                core::ptr::null(),
                0,
            );
        }
        c.issticky = false;
        arrange(Some(c.mon), globals);
    }
}

fn setlayout(arg: &Arg, globals: &mut Globals) {
    let Arg::Layout(layout) = *arg else {
        unreachable!("invalid argument for setlayout function")
    };
    let should_toggle = layout.is_none_or(|l| {
        !core::ptr::eq(
            l,
            unsafe { globals.selmon.as_ref() }.lt
                [unsafe { globals.selmon.as_ref() }.sellt as usize],
        )
    });
    if should_toggle {
        unsafe { globals.selmon.as_mut() }.sellt ^= 1;
    }

    if let Some(l) = layout {
        (unsafe { globals.selmon.as_mut() }.lt)
            [unsafe { globals.selmon.as_ref() }.sellt as usize] = l;
    }
    // symbol is &str (not null-terminated); build a CString first, matching arrangemon.
    let sellt = unsafe { globals.selmon.as_ref() }.sellt as usize;
    let symbol = CString::new(unsafe { globals.selmon.as_ref() }.lt[sellt].symbol)
        .expect("layout symbol is valid CString");
    unsafe {
        libc::strncpy(
            globals.selmon.as_mut().ltsymbol.as_mut_ptr(),
            symbol.as_ptr(),
            globals.selmon.as_ref().ltsymbol.len(),
        )
    };

    if unsafe { globals.selmon.as_ref() }.sel.is_some() {
        arrange(Some(globals.selmon), globals);
    } else {
        drawbar(unsafe { globals.selmon.as_ref() }, globals);
    }
}

fn quit(_arg: &Arg, globals: &mut Globals) {
    globals.running = false;
}

fn togglebar(_arg: &Arg, globals: &mut Globals) {
    unsafe { globals.selmon.as_mut() }.showbar = !unsafe { globals.selmon.as_ref() }.showbar;
    updatebarpos(unsafe { globals.selmon.as_mut() }, globals);
    unsafe {
        XMoveResizeWindow(
            globals.dpy.as_ptr(),
            globals.selmon.as_ref().barwin,
            globals.selmon.as_ref().wx,
            globals.selmon.as_ref().by,
            globals.selmon.as_ref().ww as u32,
            globals.bh as u32,
        )
    };
    arrange(Some(globals.selmon), globals);
}

fn togglefloating(_arg: &Arg, globals: &mut Globals) {
    // let selmon = unsafe { globals.selmon.as_mut() };
    let Some(mut sel_nn) = unsafe { globals.selmon.as_ref() }.sel else {
        return;
    };
    let sel = unsafe { sel_nn.as_mut() };
    if sel.isfullscreen {
        return;
    }
    sel.isfloating = !sel.isfloating || sel.isfixed;
    if sel.isfloating {
        let (x, y, w, h) = (sel.x, sel.y, sel.w, sel.h);
        resize(sel, x, y, w, h, false, globals);
    }
    arrange(Some(globals.selmon), globals);
}

fn togglefullscreen(_arg: &Arg, globals: &mut Globals) {
    if let Some(mut sel) = unsafe { globals.selmon.as_ref() }.sel {
        setfullscreen(
            unsafe { sel.as_mut() },
            !unsafe { sel.as_ref() }.isfullscreen,
            globals,
        );
    }
}

fn focusstack(arg: &Arg, globals: &mut Globals) {
    let mut i = stackpos(arg, globals);
    if i < 0 {
        return;
    }
    let mut p = None;
    let mut c = unsafe { globals.selmon.as_ref() }.clients;
    while let Some(c_inner) = c
        && (i > 0 || !is_visible(c_inner))
    {
        i -= if is_visible(c_inner) { 1 } else { 0 };
        p = c;
        c = unsafe { c_inner.as_ref() }.next;
    }
    focus(if c.is_some() { c } else { p }, globals);
    restack(unsafe { globals.selmon.as_ref() }, globals);
}

fn pushstack(arg: &Arg, globals: &mut Globals) {
    let mut i = stackpos(arg, globals);

    if i < 0 {
        return;
    } else if i == 0 {
        let Some(sel) = unsafe { globals.selmon.as_ref() }.sel else {
            unreachable!("should be unreachable state due to pushstack")
        };
        detach(sel);
        attach(sel);
    } else {
        let Some(mut sel) = unsafe { globals.selmon.as_ref() }.sel else {
            unreachable!("should be unreachable state due to pushstack")
        };
        let mut p = None;
        let mut c = unsafe { globals.selmon.as_ref() }.clients;
        while let Some(c_inner) = c {
            i -= if is_visible(c_inner) && c_inner != sel {
                1
            } else {
                0
            };
            if i == 0 {
                break;
            }
            p = c;
            c = unsafe { c_inner.as_ref() }.next;
        }
        let mut c = if let Some(c_inner) = c {
            c_inner
        } else {
            p.expect("should have value at this point if c is None")
        };
        detach(sel);
        unsafe { sel.as_mut() }.next = unsafe { c.as_ref() }.next;
        unsafe { c.as_mut() }.next = Some(sel);
    }
    arrange(Some(globals.selmon), globals);
}

fn stackpos(arg: &Arg, globals: &mut Globals) -> i32 {
    if unsafe { globals.selmon.as_ref() }.clients.is_none() {
        return -1;
    }
    let Arg::I(ai) = arg else {
        unreachable!("invalid argument to stackpos function")
    };
    if *ai == PREV_SEL {
        let mut l = unsafe { globals.selmon.as_ref() }.stack;
        while let Some(l_inner) = l
            && (!is_visible(l_inner) || l == unsafe { globals.selmon.as_ref() }.sel)
        {
            l = unsafe { l_inner.as_ref() }.snext
        }
        let Some(l) = l else { return -1 };
        let mut i = 0;
        let mut c = unsafe { globals.selmon.as_ref() }.clients;
        while let Some(c_inner) = c
            && c_inner != l
        {
            i += if is_visible(c_inner) { 1 } else { 0 };
            c = unsafe { c_inner.as_ref() }.next;
        }
        i
    } else if *ai > 1000 && *ai < 3000 {
        let Some(sel) = unsafe { globals.selmon.as_ref() }.sel else {
            return -1;
        };
        let mut i = 0;
        let mut c = unsafe { globals.selmon.as_ref() }.clients;
        while let Some(c_inner) = c
            && c_inner != sel
        {
            i += if is_visible(c_inner) { 1 } else { 0 };
            c = unsafe { c_inner.as_ref() }.next;
        }
        let mut n = i;
        while let Some(c_inner) = c {
            n += if is_visible(c_inner) { 1 } else { 0 };
            c = unsafe { c_inner.as_ref() }.next;
        }
        (i + (*ai - 2000)).rem_euclid(n)
    } else if *ai < 0 {
        let mut i = 0;
        let mut c = unsafe { globals.selmon.as_ref() }.clients;
        while let Some(c_inner) = c {
            i += if is_visible(c_inner) { 1 } else { 0 };
            c = unsafe { c_inner.as_ref() }.next;
        }
        (i + *ai).max(0)
    } else {
        *ai
    }
}

fn incnmaster(arg: &Arg, globals: &mut Globals) {
    let Arg::I(i) = arg else {
        unreachable!("invalid input to incnmaster")
    };
    unsafe { globals.selmon.as_mut() }.nmaster =
        (unsafe { globals.selmon.as_ref() }.nmaster + *i).max(0);
    arrange(Some(globals.selmon), globals);
}

#[allow(dead_code)]
fn setcfact(arg: &Arg, globals: &mut Globals) {
    let c = unsafe { globals.selmon.as_ref() }.sel;

    if c.is_none()
        || unsafe { globals.selmon.as_ref() }.lt[unsafe { globals.selmon.as_ref() }.sellt as usize]
            .arrange
            .is_none()
    {
        return;
    }
    let mut c = c.expect("checked to be Some");

    let Arg::F(fa) = arg else {
        unreachable!("invalid argument to setcfact function")
    };
    let mut f = *fa + unsafe { c.as_ref() }.cfact;
    if *fa == 0.0 {
        f = 1.0;
    } else if !(0.25..=4.0).contains(&f) {
        return;
    }
    unsafe { c.as_mut() }.cfact = f;
    arrange(Some(globals.selmon), globals);
}

fn setmfact(arg: &Arg, globals: &mut Globals) {
    if unsafe { globals.selmon.as_ref() }.lt[unsafe { globals.selmon.as_ref() }.sellt as usize]
        .arrange
        .is_none()
    {
        return;
    }
    let f = match arg {
        Arg::F(f) => {
            if *f < 1.0 {
                f + unsafe { globals.selmon.as_ref() }.mfact
            } else {
                f - 1.0
            }
        }
        _ => unreachable!("invalid argument for semfact function"),
    };
    if !(0.5..=0.95).contains(&f) {
        return;
    }
    unsafe { globals.selmon.as_mut() }.mfact = f;
    arrange(Some(globals.selmon), globals);
}

fn zoom(_arg: &Arg, globals: &mut Globals) {
    let mut c = unsafe { globals.selmon.as_ref() }.sel;

    if unsafe { globals.selmon.as_ref() }.lt[unsafe { globals.selmon.as_ref() }.sellt as usize]
        .arrange
        .is_none()
    {
        return;
    }
    let Some(mut c_inner) = c else {
        return;
    };
    if unsafe { c_inner.as_ref() }.isfloating {
        return;
    }
    if c == nexttiled(unsafe { globals.selmon.as_ref() }.clients) {
        c = nexttiled(unsafe { c_inner.as_ref() }.next);
        if c.is_none() {
            return;
        }
        c_inner = c.expect("checked non none");
    }
    pop(c_inner, globals)
}

fn xrdb(_arg: &Arg, globals: &mut Globals) {
    globals.resources = load_xresources();

    for (i, pallette) in config::COLORS.iter().enumerate() {
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
        let mut scm = globals.drw.scm_create(&pallette);
        std::mem::swap(&mut scm, &mut globals.scheme[i]);
        globals.drw.scm_free(scm, false);
    }

    focus(None, globals);
    arrange(None, globals);
}

fn resource_load(db: XrmDatabase, name: &str, value: &mut ResourceVal) {
    let mut fullname: [i8; 256] = [0; 256];
    let mut r#type: *mut i8 = core::ptr::null_mut();
    let mut ret: XrmValue = unsafe { core::mem::zeroed() };

    let format = CString::new(format!("{}.{}", "dwm", name)).expect("valid CString");
    unsafe { libc::snprintf(fullname.as_mut_ptr(), fullname.len(), format.as_ptr()) };
    fullname[fullname.len() - 1] = b'\0' as i8;

    unsafe { XrmGetResource(db, fullname.as_ptr(), c"*".as_ptr(), &mut r#type, &mut ret) };
    if !(ret.addr.is_null() || unsafe { libc::strncmp(c"String".as_ptr(), r#type, 64) } != 0) {
        match value {
            ResourceVal::String(s) => {
                *s = unsafe { CStr::from_ptr(ret.addr) }
                    .to_str()
                    .expect("valid &str")
                    .to_owned()
            }
            ResourceVal::Integer(u) => {
                *u = unsafe { libc::strtoul(ret.addr, core::ptr::null_mut(), 10) } as u32;
            }
            ResourceVal::Bool(b) => {
                *b = unsafe { libc::strtoul(ret.addr, core::ptr::null_mut(), 10) } != 0;
            }
            ResourceVal::Float(f) => {
                *f = unsafe { libc::strtof(ret.addr, core::ptr::null_mut()) };
            }
        }
    }
    // type points into XrmDatabase's internal memory — must not be freed.
}

fn load_xresources() -> Resources {
    let mut resources: Resources = HashMap::new();

    let display = unsafe { XOpenDisplay(core::ptr::null_mut()) };
    let resm = unsafe { XResourceManagerString(display) };

    let db = if !resm.is_null() {
        unsafe { XrmGetStringDatabase(resm) }
    } else {
        core::ptr::null_mut()
    };

    for ResourceConfig {
        name,
        x_resource_name,
        default_value,
    } in config::RESOURCE_MAPPING
    {
        let value = default_value.to_resource_val();
        let entry = resources.entry(name).or_insert(value);

        // see if we can load an updated value from the xresouces;
        if !db.is_null() {
            resource_load(db, x_resource_name, entry);
        }
    }

    unsafe { XCloseDisplay(display) };
    resources
}

fn killclient(_arg: &Arg, globals: &mut Globals) {
    const DESTROY_ALL: i32 = 0;
    let Some(sel) = unsafe { globals.selmon.as_ref() }.sel else {
        return;
    };
    if !sendevent(
        unsafe { sel.as_ref() },
        globals.wmatom[WMAtom::Delete as usize],
        globals,
    ) {
        unsafe {
            XGrabServer(globals.dpy.as_ptr());
            XSetErrorHandler(xerrordummy);
            XSetCloseDownMode(globals.dpy.as_ptr(), DESTROY_ALL);
            XKillClient(globals.dpy.as_ptr(), sel.as_ref().win);
            XSync(globals.dpy.as_ptr(), 0);
            XSetErrorHandler(xerror);
            XUngrabServer(globals.dpy.as_ptr());
        }
    }
}

fn focusmon(arg: &Arg, globals: &mut Globals) {
    if unsafe { globals.mons.as_ref() }.next.is_none() {
        return;
    }
    let Arg::I(i) = arg else {
        unreachable!("invalid argument to focus mon")
    };
    let m = dirtomon(*i, globals);
    if m == globals.selmon {
        return;
    }
    unfocus(unsafe { globals.selmon.as_ref() }.sel, false, globals);
    globals.selmon = m;
    focus(None, globals);
}

fn tagmon(arg: &Arg, globals: &mut Globals) {
    if unsafe { globals.selmon.as_ref() }.sel.is_none()
        || unsafe { globals.mons.as_ref() }.next.is_none()
    {
        return;
    }
    let Arg::I(i) = arg else {
        unreachable!("invalid argument to tagmon")
    };
    sendmon(
        unsafe { globals.selmon.as_ref() }
            .sel
            .expect("checked above to be not None"),
        dirtomon(*i, globals),
        globals,
    );
}

fn movemouse(_arg: &Arg, globals: &mut Globals) {
    const GRAB_SUCCESS: i32 = 0;

    let Some(mut c) = unsafe { globals.selmon.as_ref() }.sel else {
        return;
    };
    if unsafe { c.as_ref() }.isfullscreen {
        return;
    }
    let c_ref = unsafe { c.as_ref() };
    restack(unsafe { globals.selmon.as_ref() }, globals);
    let ocx = c_ref.x;
    let ocy = c_ref.y;
    if unsafe {
        XGrabPointer(
            globals.dpy.as_ptr(),
            globals.root,
            0,
            MOUSE_MASK as u32,
            GRAB_MODE_ASYNC,
            GRAB_MODE_ASYNC,
            0,
            globals.cursor[CursorState::Move as usize].cursor,
            CURRENT_TIME,
        )
    } != GRAB_SUCCESS
    {
        return;
    }
    let mut x = 0;
    let mut y = 0;
    let mut lasttime: Time = 0;
    if !getrootptr(&mut x, &mut y, globals) {
        return;
    }
    let mut ev: XEvent = unsafe { core::mem::zeroed() };

    loop {
        unsafe {
            XMaskEvent(
                globals.dpy.as_ptr(),
                MOUSE_MASK | EXPOSURE_MASK | SUBSTRUCTURE_REDIRECT_MASK,
                &mut ev,
            )
        };
        match unsafe { ev.r#type } {
            CONFIGURE_REQUEST | EXPOSE | MAP_REQUEST => {
                (HANDLER[unsafe { ev.r#type } as usize].expect("valid function"))(&mut ev, globals);
            }
            MOTION_NOTIFY => {
                if unsafe { ev.xmotion }.time - lasttime <= 1000 / config::REFRESH_RATE as u64 {
                    continue;
                }
                lasttime = unsafe { ev.xmotion.time };

                let mut nx = ocx + (unsafe { ev.xmotion.x } - x);
                let mut ny = ocy + (unsafe { ev.xmotion.y } - y);

                // let snap = load_resource_int("SNAP", globals);
                let snap = load_resource!("SNAP", globals, Integer);

                if (unsafe { globals.selmon.as_ref().wx } - nx).abs() < snap as i32 {
                    nx = unsafe { globals.selmon.as_ref().wx };
                } else if ((unsafe { globals.selmon.as_ref().wx }
                    + unsafe { globals.selmon.as_ref().ww })
                    - (nx + unsafe { c.as_ref().width() }))
                .abs()
                    < snap as i32
                {
                    nx = unsafe { globals.selmon.as_ref().wx }
                        + unsafe { globals.selmon.as_ref().ww }
                        - unsafe { c.as_ref().width() };
                }
                if (unsafe { globals.selmon.as_ref().wy } - ny).abs() < snap as i32 {
                    ny = unsafe { globals.selmon.as_ref().wy };
                } else if ((unsafe { globals.selmon.as_ref().wy }
                    + unsafe { globals.selmon.as_ref().wh })
                    - (ny + unsafe { c.as_ref().height() }))
                .abs()
                    < snap as i32
                {
                    ny = unsafe { globals.selmon.as_ref().wy }
                        + unsafe { globals.selmon.as_ref().wh }
                        - unsafe { c.as_ref().height() };
                }
                if !unsafe { c.as_ref().isfloating }
                    && unsafe { globals.selmon.as_ref().lt }
                        [unsafe { globals.selmon.as_ref().sellt } as usize]
                        .arrange
                        .is_some()
                    && ((nx - unsafe { c.as_ref().x }).abs() > snap as i32
                        || (ny - unsafe { c.as_ref().y }).abs() > snap as i32)
                {
                    togglefloating(&Arg::I(0), globals);
                }
                if unsafe { globals.selmon.as_ref().lt }
                    [unsafe { globals.selmon.as_ref().sellt } as usize]
                    .arrange
                    .is_none()
                    || unsafe { c.as_ref().isfloating }
                {
                    let (w, h) = unsafe { (c.as_ref().w, c.as_ref().h) };
                    resize(unsafe { c.as_mut() }, nx, ny, w, h, true, globals);
                }
            }
            _ => {}
        }
        if unsafe { ev.r#type } == BUTTON_RELEASE {
            break;
        }
    }
    unsafe { XUngrabPointer(globals.dpy.as_ptr(), CURRENT_TIME) };
    let m = recttomon(
        unsafe { c.as_ref() }.x,
        unsafe { c.as_ref() }.y,
        unsafe { c.as_ref() }.w,
        unsafe { c.as_ref() }.h,
        globals,
    );
    if m != globals.selmon {
        sendmon(c, m, globals);
        globals.selmon = m;
        focus(None, globals);
    }
}

fn resizemouse(_arg: &Arg, globals: &mut Globals) {
    const GRAB_SUCCESS: i32 = 0;

    let Some(mut c) = unsafe { globals.selmon.as_ref() }.sel else {
        return;
    };
    let cr = unsafe { c.as_ref() }; /* no support resizing fullscreen windows by mouse */
    if cr.isfullscreen {
        return;
    }
    restack(unsafe { globals.selmon.as_ref() }, globals);
    let ocx = cr.x;
    let ocy = cr.y;
    if unsafe {
        XGrabPointer(
            globals.dpy.as_ptr(),
            globals.root,
            0,
            MOUSE_MASK as u32,
            GRAB_MODE_ASYNC,
            GRAB_MODE_ASYNC,
            0,
            globals.cursor[CursorState::Resize as usize].cursor,
            CURRENT_TIME,
        )
    } != GRAB_SUCCESS
    {
        return;
    }
    unsafe {
        XWarpPointer(
            globals.dpy.as_ptr(),
            0,
            cr.win,
            0,
            0,
            0,
            0,
            cr.w + cr.bw - 1,
            cr.h + cr.bw - 1,
        )
    };

    let mut ev: XEvent = unsafe { core::mem::zeroed() };
    let mut lasttime = 0;
    loop {
        unsafe {
            XMaskEvent(
                globals.dpy.as_ptr(),
                MOUSE_MASK | EXPOSURE_MASK | SUBSTRUCTURE_REDIRECT_MASK,
                &mut ev,
            )
        };
        match unsafe { ev.r#type } {
            CONFIGURE_REQUEST | EXPOSE | MAP_REQUEST => {
                (HANDLER[unsafe { ev.r#type } as usize].expect("valid function"))(&mut ev, globals);
            }
            MOTION_NOTIFY => {
                if unsafe { ev.xmotion.time } - lasttime <= 1000 / config::REFRESH_RATE as u64 {
                    continue;
                }
                lasttime = unsafe { ev.xmotion.time };

                let nw = 1.max(unsafe { ev.xmotion.x } - ocx - 2 * cr.bw + 1);
                let nh = 1.max(unsafe { ev.xmotion.y } - ocy - 2 * cr.bw + 1);

                // let snap = load_resource_int("SNAP", globals);
                let snap = load_resource!("SNAP", globals, Integer);

                if unsafe { cr.mon.as_ref().wx } + nw >= unsafe { globals.selmon.as_ref().wx }
                    && unsafe { cr.mon.as_ref().wx } + nw
                        <= unsafe { globals.selmon.as_ref().wx }
                            + unsafe { globals.selmon.as_ref().ww }
                    && unsafe { cr.mon.as_ref().wy } + nh >= unsafe { globals.selmon.as_ref().wy }
                    && unsafe { cr.mon.as_ref().wy } + nh
                        <= unsafe { globals.selmon.as_ref().wy }
                            + unsafe { globals.selmon.as_ref().wh }
                    && !cr.isfloating
                    && unsafe { globals.selmon.as_ref().lt }
                        [unsafe { globals.selmon.as_ref().sellt } as usize]
                        .arrange
                        .is_some()
                    && ((nw - cr.w).abs() > snap as i32 || (nh - cr.h).abs() > snap as i32)
                {
                    togglefloating(&Arg::I(0), globals);
                }

                if unsafe { globals.selmon.as_ref().lt }
                    [unsafe { globals.selmon.as_ref().sellt } as usize]
                    .arrange
                    .is_none()
                    || cr.isfloating
                {
                    resize(unsafe { c.as_mut() }, cr.x, cr.y, nw, nh, true, globals);
                }
            }
            _ => {}
        }
        if unsafe { ev.r#type } == BUTTON_RELEASE {
            break;
        }
    }
    unsafe {
        XWarpPointer(
            globals.dpy.as_ptr(),
            0,
            cr.win,
            0,
            0,
            0,
            0,
            cr.w + cr.bw - 1,
            cr.h + cr.bw - 1,
        );
        XUngrabPointer(globals.dpy.as_ptr(), CURRENT_TIME);
    };
    while unsafe { XCheckMaskEvent(globals.dpy.as_ptr(), ENTER_WINDOW_MASK, &mut ev) } != 0 {}
    let m = recttomon(cr.x, cr.y, cr.w, cr.h, globals);
    if m != globals.selmon {
        sendmon(c, m, globals);
        globals.selmon = m;
        focus(None, globals);
    }
}

fn buttonpress(ev: &mut XEvent, globals: &mut Globals) {
    const REPLAY_POINTER: i32 = 2;

    let mut click = ClickState::RootWin;
    let ev: &mut XButtonPressedEvent = unsafe { &mut ev.xbutton };
    let mut arg = Arg::Ui(0);

    /* focus monitor if necessary */
    let m = wintomon(ev.window, globals);
    if m != globals.selmon {
        unfocus(unsafe { globals.selmon.as_ref() }.sel, true, globals);
        globals.selmon = m;
        focus(None, globals);
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
    } else if let Some(c) = wintoclient(ev.window, globals) {
        focus(Some(c), globals);
        restack(unsafe { globals.selmon.as_ref() }, globals);

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
    let c = wintoclient(cme.window, globals);
    let Some(mut c) = c else {
        return;
    };
    if cme.message_type == globals.netatom[NetAtom::WMState as usize] {
        if unsafe { cme.data.l }[1] == globals.netatom[NetAtom::WMFullscreen as usize] as i64
            || unsafe { cme.data.l }[2] == globals.netatom[NetAtom::WMFullscreen as usize] as i64
        {
            setfullscreen(
                unsafe { c.as_mut() },
                unsafe { cme.data.l }[0] == 1  /* _NET_WM_STATE_ADD    */
                || (unsafe { cme.data.l }[0] == 2 /* _NET_WM_STATE_TOGGLE */
                && !unsafe { c.as_ref()}.isfullscreen ),
                globals,
            );
        }

        if unsafe { cme.data.l[1] } == globals.netatom[NetAtom::WMSticky as usize] as i64
            || unsafe { cme.data.l[2] } == globals.netatom[NetAtom::WMSticky as usize] as i64
        {
            setsticky(
                unsafe { c.as_mut() },
                unsafe { cme.data.l[0] } == 1
                    || (unsafe { cme.data.l[0] } == 2 && !unsafe { c.as_ref() }.issticky),
                globals,
            )
        }
    } else if cme.message_type == globals.netatom[NetAtom::ActiveWindow as usize]
        && (unsafe { globals.selmon.as_ref().sel }.is_none()
            || c != unsafe { globals.selmon.as_ref() }
                .sel
                .expect("early termination"))
        && !unsafe { c.as_ref() }.isurgent
    {
        seturgent(unsafe { c.as_mut() }, true, globals);
    }
}

fn configurerequest(ev: &mut XEvent, globals: &mut Globals) {
    let ev: &mut XConfigureRequestEvent = unsafe { &mut ev.xconfigurerequest };

    if let Some(mut c) = wintoclient(ev.window, globals) {
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
                configure(c_ref, globals);
            }
            if is_visible(c) {
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
            configure(c_ref, globals);
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
                    if unsafe { c_inner.as_ref() }.isfullscreen {
                        resizeclient(
                            unsafe { c_inner.as_mut() },
                            m_inner.mx,
                            m_inner.my,
                            m_inner.mw,
                            m_inner.mh,
                            globals,
                        );
                    }
                    c = unsafe { c_inner.as_ref() }.next
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
            focus(None, globals);
            arrange(None, globals);
        }
    }
}

fn destroynotify(ev: &mut XEvent, globals: &mut Globals) {
    let ev: &mut XDestroyWindowEvent = unsafe { &mut ev.xdestroywindow };
    if let Some(c) = wintoclient(ev.window, globals) {
        unmanage(c, true, globals);
    } else if let Some(c) = swallowingclient(ev.window, globals) {
        unmanage(
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
    let c = wintoclient(ev.window, globals);
    let m = if let Some(c) = c {
        unsafe { c.as_ref() }.mon
    } else {
        wintomon(ev.window, globals)
    };
    if m != globals.selmon {
        unfocus(unsafe { globals.selmon.as_ref() }.sel, true, globals);
        globals.selmon = m;
    } else if c.is_none() || c == unsafe { globals.selmon.as_ref() }.sel {
        return;
    }
    focus(c, globals);
}

fn expose(ev: &mut XEvent, globals: &mut Globals) {
    let ev: &mut XExposeEvent = unsafe { &mut ev.xexpose };
    if ev.count == 0 {
        let m = wintomon(ev.window, globals);
        drawbar(unsafe { m.as_ref() }, globals);
    }
}

fn focusin(ev: &mut XEvent, globals: &mut Globals) {
    let ev: &mut XFocusChangeEvent = unsafe { &mut ev.xfocus };
    if let Some(sel) = unsafe { globals.selmon.as_ref() }.sel
        && ev.window != unsafe { sel.as_ref() }.win
    {
        setfocus(unsafe { sel.as_ref() }, globals);
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
    if wintoclient(ev.window, globals).is_none() {
        manage(ev.window, &wa, globals);
    }
}

fn motionnotify(ev: &mut XEvent, globals: &mut Globals) {
    let ev: &mut XMotionEvent = unsafe { &mut ev.xmotion };

    if ev.window != globals.root {
        return;
    }
    let m = recttomon(ev.x_root, ev.y_root, 1, 1, globals);
    if let Some(last) = globals.last_motion_mon
        && last != m
    {
        unfocus(unsafe { globals.selmon.as_ref() }.sel, true, globals);
        globals.selmon = m;
        focus(None, globals);
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
    } else if let Some(mut c) = wintoclient(ev.window, globals) {
        match ev.atom {
            XA_WM_TRANSIENT_FOR
                if !unsafe { c.as_ref() }.isfloating
                    && (unsafe {
                        XGetTransientForHint(globals.dpy.as_mut(), c.as_ref().win, &mut trans)
                    } != 0) =>
            {
                unsafe { c.as_mut() }.isfloating = wintoclient(trans, globals).is_some();
                if unsafe { c.as_ref() }.isfloating {
                    arrange(Some(unsafe { c.as_ref() }.mon), globals);
                }
            }

            XA_WM_NORMAL_HINTS => {
                unsafe { c.as_mut() }.hintsvalid = false;
            }
            XA_WM_HINTS => {
                updatewmhints(unsafe { c.as_mut() }, globals);
                drawbars(globals);
            }
            _ => {}
        }
        if ev.atom == XA_WM_NAME || ev.atom == globals.netatom[NetAtom::WMName as usize] {
            updatetitle(unsafe { c.as_mut() }, globals);
            if let Some(sel) = unsafe { c.as_ref().mon.as_ref() }.sel
                && c == sel
            {
                drawbar(unsafe { c.as_ref().mon.as_ref() }, globals);
            }
        }
        if ev.atom == globals.netatom[NetAtom::WMWindowType as usize] {
            updatewindowtype(c, globals);
        }
    }
}

fn unmapnotify(ev: &mut XEvent, globals: &mut Globals) {
    let ev: &mut XUnmapEvent = unsafe { &mut ev.xunmap };

    if let Some(c) = wintoclient(ev.window, globals) {
        if ev.send_event != 0 {
            setclientstate(unsafe { c.as_ref() }, WITHDRAWN_STATE as i64, globals);
        } else {
            unmanage(c, false, globals);
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
    return 1;
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
    let xlib: extern "C" fn(*mut Display, *mut XErrorEvent) -> i32 =
        unsafe { core::mem::transmute(XERRORXLIB.load(Ordering::Relaxed)) };
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

fn createmon(globals: &Globals) -> NonNull<Monitor> {
    let mut ltsym: [i8; 16] = [0; 16];
    for (i, b) in config::LAYOUTS[0]
        .symbol
        .as_bytes()
        .iter()
        .take(16)
        .enumerate()
    {
        ltsym[i] = *b as i8;
    }

    let m: Box<Monitor> = Box::new(Monitor {
        ltsymbol: ltsym,
        mfact: load_resource!("M_FACT", globals, Float),
        nmaster: load_resource!("N_MASTER", globals, Integer) as i32,
        num: 0,
        by: 0,
        mx: 0,
        my: 0,
        mw: 0,
        mh: 0,
        wx: 0,
        wy: 0,
        ww: 0,
        wh: 0,
        gappih: load_resource!("GAPP_IH", globals, Integer) as i32,
        gappiv: load_resource!("GAPP_IV", globals, Integer) as i32,
        gappoh: load_resource!("GAPP_OH", globals, Integer) as i32,
        gappov: load_resource!("GAPP_OV", globals, Integer) as i32,
        seltags: 0,
        sellt: 0,
        tagset: [1, 1],
        showbar: load_resource!("SHOW_BAR", globals, Bool),
        topbar: load_resource!("TOP_BAR", globals, Bool),
        clients: None,
        sel: None,
        stack: None,
        next: None,
        barwin: 0,
        lt: [
            &config::LAYOUTS[0],
            &config::LAYOUTS[1 % config::LAYOUTS.len()],
        ],
    });

    NonNull::new(Box::leak(m)).expect("valid NonNull as created by Box")
}

fn updatebarpos(m: &mut Monitor, globals: &Globals) {
    m.wy = m.my;
    m.wh = m.mh;
    if m.showbar {
        m.wh -= globals.bh;
        m.by = if m.topbar { m.wy } else { m.wy + m.wh };
        m.wy = if m.topbar { m.wy + globals.bh } else { m.wy };
    } else {
        m.by = -globals.bh;
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

fn updatewmhints(c: &mut Client, globals: &Globals) {
    const INPUT_HINT: i64 = 1 << 0;
    const X_URGENCY_HINT: i64 = 1 << 8;

    let wmh: *mut XWMHints = unsafe { XGetWMHints(globals.dpy.as_ptr(), c.win) };
    if !wmh.is_null() {
        let is_sel = unsafe { globals.selmon.as_ref() }
            .sel
            .is_some_and(|sel| core::ptr::eq(c as *const _, sel.as_ptr()));
        // .map_or(false, |sel| core::ptr::eq(c as *const _, sel.as_ptr()));
        if is_sel && unsafe { &*wmh }.flags & X_URGENCY_HINT != 0 {
            unsafe { &mut *wmh }.flags &= !X_URGENCY_HINT;
            unsafe { XSetWMHints(globals.dpy.as_ptr(), c.win, wmh) };
        } else {
            c.isurgent = unsafe { &*wmh }.flags & X_URGENCY_HINT != 0;
        }
        c.neverfocus = if unsafe { &*wmh }.flags & INPUT_HINT != 0 {
            unsafe { &*wmh }.input == 0
        } else {
            false
        };
        unsafe { XFree(wmh.cast()) };
    }
}

fn recttomon(x: i32, y: i32, w: i32, h: i32, globals: &Globals) -> NonNull<Monitor> {
    let mut m: Option<NonNull<Monitor>>;
    let mut r = globals.selmon;
    let mut area = 0;

    m = Some(globals.mons);
    while let Some(m_inner) = m {
        let m_inner_ref = unsafe { m_inner.as_ref() };
        let a = intersect!(i32, x, y, w, h, m_inner_ref);
        if a > area {
            area = a;
            r = m_inner;
        }
        m = m_inner_ref.next;
    }
    r
}

fn wintoclient(w: Window, globals: &Globals) -> Option<NonNull<Client>> {
    let mut m = Some(globals.mons);
    while let Some(m_inner) = m {
        let m_inner = unsafe { m_inner.as_ref() };
        let mut c = m_inner.clients;
        while let Some(c_inner) = c {
            let c_inner = unsafe { c_inner.as_ref() };
            if c_inner.win == w {
                return c;
            }
            c = c_inner.next
        }
        m = m_inner.next;
    }
    None
}

fn wintomon(w: Window, globals: &mut Globals) -> NonNull<Monitor> {
    let mut x = 0;
    let mut y = 0;

    if w == globals.root && getrootptr(&mut x, &mut y, globals) {
        return recttomon(x, y, 1, 1, globals);
    }
    let mut m = Some(globals.mons);
    while let Some(m_inner) = m {
        if w == unsafe { m_inner.as_ref() }.barwin {
            return m_inner;
        }
        m = unsafe { m_inner.as_ref() }.next;
    }

    let c = wintoclient(w, globals);
    if let Some(c) = c {
        return unsafe { c.as_ref() }.mon;
    }
    globals.selmon
}

fn updategeom(globals: &mut Globals) -> bool {
    let mut dirty = false;

    #[cfg(feature = "xinerama")]
    {}

    // We are in initialization
    if !globals.running {
        globals.mons = createmon(globals);
    }

    let mons_ref = unsafe { globals.mons.as_mut() };
    if mons_ref.mw != globals.sw || mons_ref.mh != globals.sh {
        dirty = true;
        mons_ref.ww = globals.sw;
        mons_ref.mw = mons_ref.ww;
        mons_ref.wh = globals.sh;
        mons_ref.mh = mons_ref.wh;
        updatebarpos(mons_ref, globals);
    }
    if dirty {
        globals.selmon = globals.mons;
        globals.selmon = wintomon(globals.root, globals);
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
                globals.cursor[CursorState::Normal as usize].cursor,
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

fn drawbar(m: &Monitor, globals: &mut Globals) {
    let mut tw = 0i32;
    let font_h = unsafe {
        globals
            .drw
            .fonts
            .expect("checked at init that at least 1 font exists")
            .as_ref()
            .h
    };
    let boxs = font_h / 9;
    let boxw = font_h / 6 + 2;
    let mut occ = 0u32;
    let mut urg = 0u32;

    if !m.showbar {
        return;
    }

    let is_selmon = core::ptr::eq(m, globals.selmon.as_ptr());
    if is_selmon {
        globals
            .drw
            .setscheme(Rc::clone(&globals.scheme[SchemeState::Norm as usize]));

        // tw = globals
        //     .drw
        //     .as_mut()
        //     .fontset_getwidth(&globals.stext as *const i8) as i32
        //     + 2; /* 2px right padding */
        // globals.drw.text(
        //     m.ww - tw,
        //     0,
        //     tw as u32,
        //     globals.bh as u32,
        //     0,
        //     &globals.stext as *const i8,
        //     false,
        // );
        let mut text = globals.stext.as_mut_ptr();
        let mut s = globals.stext.as_mut_ptr();
        let mut x = 0;
        while unsafe { *s } != 0 {
            // for (text = s = stext; *s; s++) {
            if (unsafe { *s } as u8) < b' ' {
                let ch = unsafe { *s };
                unsafe { *s = b'\0' as i8 };
                tw = text_w(text, globals) - globals.lrpad;
                globals.drw.text(
                    m.ww - globals.statusw + x,
                    0,
                    tw as u32,
                    globals.bh as u32,
                    0,
                    text,
                    false,
                );
                x += tw;
                unsafe { *s = ch };
                text = unsafe { s.add(1) };
            }
            s = unsafe { s.add(1) };
        }
        tw = text_w(text, globals) - globals.lrpad + 2;
        globals.drw.text(
            m.ww - globals.statusw + x,
            0,
            tw as u32,
            globals.bh as u32,
            0,
            text,
            false,
        );
        tw = globals.statusw;
    }
    let mut c = m.clients;
    while let Some(c_inner) = c {
        let c_ref = unsafe { c_inner.as_ref() };
        occ |= if c_ref.tags == TAGMASK { 0 } else { c_ref.tags };
        if c_ref.isurgent {
            urg |= c_ref.tags;
        }
        c = c_ref.next
    }
    let mut x = 0;
    for i in 0..config::TAGS.len() {
        // Do not draw vacant tags
        if !(occ & 1 << i != 0 || m.tagset[m.seltags as usize] & 1 << i != 0) {
            continue;
        }

        let tag = CString::new(config::TAGS[i]).expect("valid c string");
        let w = globals.drw.fontset_getwidth(tag.as_ptr()) + globals.lrpad as u32;
        globals.drw.setscheme(Rc::clone(
            &globals.scheme[if (m.tagset[m.seltags as usize] & 1 << i) != 0 {
                SchemeState::Sel as usize
            } else {
                SchemeState::Norm as usize
            }],
        ));
        globals.drw.text(
            x,
            0,
            w,
            globals.bh as u32,
            globals.lrpad as u32 / 2,
            tag.as_ptr(),
            urg & 1 << i != 0,
        );
        x += w as i32;
    }

    let w = globals.drw.fontset_getwidth(&m.ltsymbol as *const i8) + globals.lrpad as u32;
    globals
        .drw
        .setscheme(Rc::clone(&globals.scheme[SchemeState::Norm as usize]));
    let x = globals.drw.text(
        x,
        0,
        w,
        globals.bh as u32,
        globals.lrpad as u32 / 2,
        &m.ltsymbol as *const i8,
        false,
    );

    let w = m.ww - tw - x;
    if w > globals.bh {
        if let Some(m_sel) = m.sel {
            let m_sel_ref = unsafe { m_sel.as_ref() };
            globals.drw.setscheme(Rc::clone(
                &globals.scheme[if is_selmon {
                    SchemeState::Sel as usize
                } else {
                    SchemeState::Norm as usize
                }],
            ));
            globals.drw.text(
                x,
                0,
                w as u32,
                globals.bh as u32,
                globals.lrpad as u32 / 2,
                &m_sel_ref.name as *const i8,
                false,
            );
            if m_sel_ref.isfloating {
                globals.drw.rect(
                    x + boxs as i32,
                    boxs as i32,
                    boxw,
                    boxw,
                    m_sel_ref.isfixed,
                    false,
                );
            }
        } else {
            globals
                .drw
                .setscheme(Rc::clone(&globals.scheme[SchemeState::Norm as usize]));
            globals
                .drw
                .rect(x, 0, w as u32, globals.bh as u32, true, true);
        }
    }
    globals
        .drw
        .map(m.barwin, 0, 0, m.ww as u32, globals.bh as u32)
}

fn drawbars(globals: &mut Globals) {
    let mut m = Some(globals.mons);
    while let Some(m_inner) = m {
        drawbar(unsafe { m_inner.as_ref() }, globals);
        m = unsafe { m_inner.as_ref() }.next;
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

fn sigstatusbar(arg: &Arg, globals: &mut Globals) {
    let mut sv: libc::sigval = unsafe { core::mem::zeroed() };

    if globals.statussig == 0 {
        return;
    }
    let Arg::I(i) = arg else {
        unreachable!("invalid argument to sigstatusbar")
    };
    sv.sival_ptr = (*i) as *mut c_void;
    let statuspid = getstatusbarpid(globals);
    if statuspid <= 0 {
        return;
    }

    unsafe { libc::sigqueue(statuspid, libc::SIGRTMIN() + globals.statussig, sv) };
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

    drawbar(unsafe { globals.selmon.as_ref() }, globals);
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

fn grabbuttons(c: &Client, focused: bool, globals: &mut Globals) {
    updatenumlockmask(globals);
    {
        let modifiers = [
            0,
            LOCK_MASK,
            globals.numlockmask,
            globals.numlockmask | LOCK_MASK,
        ];

        unsafe { XUngrabButton(globals.dpy.as_ptr(), ANY_BUTTON, ANY_MODIFIER, c.win) };
        if !focused {
            unsafe {
                XGrabButton(
                    globals.dpy.as_ptr(),
                    ANY_BUTTON,
                    ANY_MODIFIER,
                    c.win,
                    0,
                    (BUTTON_PRESS_MASK | BUTTON_RELEASE_MASK) as u32,
                    GRAB_MODE_SYNC,
                    GRAB_MODE_SYNC,
                    0,
                    0,
                )
            };
        }
        for i in 0..config::BUTTONS.len() {
            if config::BUTTONS[i].click == ClickState::ClientWin {
                for modi in modifiers {
                    unsafe {
                        XGrabButton(
                            globals.dpy.as_ptr(),
                            config::BUTTONS[i].button,
                            config::BUTTONS[i].mask | modi,
                            c.win,
                            0,
                            (BUTTON_PRESS_MASK | BUTTON_RELEASE_MASK) as u32,
                            GRAB_MODE_ASYNC,
                            GRAB_MODE_SYNC,
                            0,
                            0,
                        )
                    };
                }
            }
        }
    }
}

fn seturgent(c: &mut Client, urg: bool, globals: &Globals) {
    c.isurgent = urg;
    let wmh = unsafe { XGetWMHints(globals.dpy.as_ptr(), c.win) };
    if wmh.is_null() {
        return;
    }

    const X_URGENCY_HINT: i64 = 1 << 8;

    let f = unsafe { &*wmh }.flags;
    unsafe { &mut *wmh }.flags = if urg {
        f | X_URGENCY_HINT
    } else {
        f & !X_URGENCY_HINT
    };
    unsafe { XSetWMHints(globals.dpy.as_ptr(), c.win, wmh) };
    unsafe { XFree(wmh.cast()) };
}

fn attach(mut c: NonNull<Client>) {
    unsafe { c.as_mut().next = c.as_ref().mon.as_ref().clients }
    unsafe { c.as_mut().mon.as_mut().clients = Some(c) };
}

fn attachstack(mut c: NonNull<Client>) {
    unsafe { c.as_mut().snext = c.as_ref().mon.as_ref().stack };
    unsafe { c.as_mut().mon.as_mut().stack = Some(c) };
}

fn swallow(mut p: NonNull<Client>, mut c: NonNull<Client>, globals: &mut Globals) {
    let c_ref = unsafe { c.as_mut() };
    let p_ref = unsafe { p.as_mut() };
    if c_ref.noswallow || c_ref.isterminal {
        return;
    }
    if c_ref.noswallow && !load_resource!("SWALLOW_FLOATING", globals, Bool) && c_ref.isfloating {
        return;
    }

    detach(c);
    detachstack(c);

    setclientstate(c_ref, WITHDRAWN_STATE as i64, globals);
    unsafe { XUnmapWindow(globals.dpy.as_ptr(), p_ref.win) };

    p_ref.swallowing = Some(c);
    c_ref.mon = p_ref.mon;

    std::mem::swap(&mut p_ref.win, &mut c_ref.win);
    updatetitle(p_ref, globals);
    unsafe {
        XMoveResizeWindow(
            globals.dpy.as_ptr(),
            p_ref.win,
            p_ref.x,
            p_ref.y,
            p_ref.w as u32,
            p_ref.h as u32,
        )
    };
    arrange(Some(p_ref.mon), globals);
    configure(p_ref, globals);
    updateclientlist(globals);
}

fn unswallow(mut c: NonNull<Client>, globals: &mut Globals) {
    let c_ref = unsafe { c.as_mut() };
    let Some(swallowed) = c_ref.swallowing.take() else {
        unreachable!("gave a client to unswallow that has not swallowed anything.")
    };
    c_ref.win = unsafe { swallowed.as_ref() }.win;
    //Free the swallowed object, having set c.swallowing to None by take above.
    let _ = unsafe { Box::from_raw(swallowed.as_ptr()) };

    /* unfullscreen the client */
    setfullscreen(c_ref, false, globals);
    updatetitle(c_ref, globals);
    arrange(Some(c_ref.mon), globals);
    unsafe {
        XMapWindow(globals.dpy.as_ptr(), c_ref.win);
        XMoveResizeWindow(
            globals.dpy.as_ptr(),
            c_ref.win,
            c_ref.x,
            c_ref.y,
            c_ref.w as u32,
            c_ref.h as u32,
        );
    }
    setclientstate(c_ref, NORMAL_STATE as i64, globals);
    focus(None, globals);
    arrange(Some(c_ref.mon), globals);
}

fn detach(mut c: NonNull<Client>) {
    let mut tc = &mut unsafe { c.as_mut().mon.as_mut() }.clients;
    while let Some(tc_inner) = tc.as_mut()
        && *tc_inner != c
    {
        tc = &mut unsafe { tc_inner.as_mut() }.next
    }
    *tc = unsafe { c.as_ref().next };
}

fn detachstack(mut c: NonNull<Client>) {
    let mut tc = &mut unsafe { c.as_mut().mon.as_mut() }.stack;
    while let Some(mut inner) = *tc
        && c != inner
    {
        tc = &mut unsafe { inner.as_mut() }.snext;
    }
    *tc = unsafe { c.as_ref() }.snext;

    if let Some(sel) = unsafe { c.as_ref().mon.as_ref() }.sel
        && c == sel
    {
        let mut t = unsafe { c.as_ref().mon.as_ref() }.stack;
        while let Some(t_inner) = t
            && !is_visible(t_inner)
        {
            t = unsafe { t_inner.as_ref() }.snext;
        }
        unsafe { c.as_mut().mon.as_mut().sel = t };
    }
}

fn sendevent(c: &Client, proto: Atom, globals: &Globals) -> bool {
    const CLIENT_MESSAGE: i32 = 33;

    let mut n: i32 = 0;
    let mut protocols: *mut Atom = core::ptr::null_mut();
    let mut exists = false;

    if unsafe { XGetWMProtocols(globals.dpy.as_ptr(), c.win, &mut protocols, &mut n) } != 0 {
        while !exists && n > 0 {
            n -= 1;
            exists = unsafe { *protocols.add(n as usize) } == proto
        }
        unsafe { XFree(protocols.cast()) };
    }
    if exists {
        let mut ev = XEvent {
            xclient: XClientMessageEvent {
                r#type: CLIENT_MESSAGE,
                serial: 0,
                send_event: 0,
                display: core::ptr::null_mut(),
                window: c.win,
                message_type: globals.wmatom[WMAtom::Protocols as usize],
                format: 32,
                data: XClientMessageEventData {
                    l: [proto as i64, CURRENT_TIME as i64, 0, 0, 0],
                },
            },
        };
        unsafe { XSendEvent(globals.dpy.as_ptr(), c.win, 0, NO_EVENT_MASK, &mut ev) };
    }

    exists
}

fn sendmon(mut c: NonNull<Client>, m: NonNull<Monitor>, globals: &mut Globals) {
    if unsafe { c.as_ref() }.mon == m {
        return;
    }
    unfocus(Some(c), true, globals);
    detach(c);
    detachstack(c);
    unsafe { c.as_mut().mon = m };
    unsafe { c.as_mut() }.tags =
        unsafe { m.as_ref() }.tagset[unsafe { m.as_ref() }.seltags as usize]; /* assign tags of target monitor */
    attach(c);
    attachstack(c);
    if unsafe { c.as_ref() }.isfullscreen {
        resizeclient(
            unsafe { c.as_mut() },
            unsafe { m.as_ref() }.mx,
            unsafe { m.as_ref() }.my,
            unsafe { m.as_ref() }.mw,
            unsafe { m.as_ref() }.mh,
            globals,
        );
    }
    focus(None, globals);
    arrange(None, globals);
}

// dirtomon always returns a valid monitor. Callers must guard with
// `if mons.next.is_none() { return; }` before calling to ensure ≥2 monitors exist.
// In all three branches we either wrap around to `mons` (non-null by invariant) or
// walk a linked list that is guaranteed to contain `selmon`.
fn dirtomon(dir: i32, globals: &Globals) -> NonNull<Monitor> {
    if dir > 0 {
        // Next monitor, wrapping around to the first if selmon is the last.
        unsafe { globals.selmon.as_ref() }
            .next
            .unwrap_or(globals.mons)
    } else if globals.selmon == globals.mons {
        // Walk to the last monitor in the list.
        let mut m = globals.mons;
        while let Some(next) = unsafe { m.as_ref() }.next {
            m = next;
        }
        m
    } else {
        // Walk to the predecessor of selmon.
        let mut m = globals.mons;
        while unsafe { m.as_ref() }.next != Some(globals.selmon) {
            m = unsafe { m.as_ref() }
                .next
                .expect("selmon is always reachable from mons");
        }
        m
    }
}

fn pop(c: NonNull<Client>, globals: &mut Globals) {
    detach(c);
    attach(c);
    focus(Some(c), globals);
    arrange(Some(unsafe { c.as_ref() }.mon), globals);
}

fn setfocus(c: &Client, globals: &Globals) {
    unsafe {
        if !c.neverfocus {
            XSetInputFocus(
                globals.dpy.as_ptr(),
                c.win,
                REVERT_TO_POINTER_ROOT,
                CURRENT_TIME,
            );
        }
        XChangeProperty(
            globals.dpy.as_ptr(),
            globals.root,
            globals.netatom[NetAtom::ActiveWindow as usize],
            XA_WINDOW,
            32,
            PROP_MODE_REPLACE,
            (&c.win) as *const _ as *const u8,
            1,
        );
    }
    sendevent(c, globals.wmatom[WMAtom::TakeFocus as usize], globals);
}

fn unfocus(c: Option<NonNull<Client>>, setfocus: bool, globals: &mut Globals) {
    let Some(c) = c else { return };
    grabbuttons(unsafe { c.as_ref() }, false, globals);
    unsafe {
        XSetWindowBorder(
            globals.dpy.as_ptr(),
            c.as_ref().win,
            globals.scheme[SchemeState::Norm as usize][drw::COL_BORDER].pixel,
        )
    };

    if setfocus {
        unsafe {
            XSetInputFocus(
                globals.dpy.as_ptr(),
                globals.root,
                REVERT_TO_POINTER_ROOT,
                CURRENT_TIME,
            )
        };
        unsafe {
            XDeleteProperty(
                globals.dpy.as_ptr(),
                globals.root,
                globals.netatom[NetAtom::ActiveWindow as usize],
            )
        };
    }
}

fn focus(mut c: Option<NonNull<Client>>, globals: &mut Globals) {
    if !c.is_some_and(is_visible) {
        c = unsafe { globals.selmon.as_ref() }.stack;
        while let Some(c_inner) = c
            && !is_visible(c_inner)
        {
            c = unsafe { c_inner.as_ref() }.snext;
        }
    }
    if let Some(sel) = unsafe { globals.selmon.as_ref() }.sel
        && let Some(c_inner) = c
        && sel != c_inner
    {
        unfocus(Some(sel), false, globals);
    }
    if let Some(mut c_inner) = c {
        let c_ref = unsafe { c_inner.as_ref() };
        if c_ref.mon != globals.selmon {
            globals.selmon = c_ref.mon;
        }
        if c_ref.isurgent {
            seturgent(unsafe { c_inner.as_mut() }, false, globals)
        }
        detachstack(c_inner);
        attachstack(c_inner);
        grabbuttons(c_ref, true, globals);
        unsafe {
            XSetWindowBorder(
                globals.dpy.as_ptr(),
                c_ref.win,
                globals.scheme[SchemeState::Sel as usize][drw::COL_BORDER].pixel,
            )
        };
        setfocus(c_ref, globals);
    } else {
        unsafe {
            XSetInputFocus(
                globals.dpy.as_ptr(),
                globals.root,
                REVERT_TO_POINTER_ROOT,
                CURRENT_TIME,
            )
        };
        unsafe {
            XDeleteProperty(
                globals.dpy.as_ptr(),
                globals.root,
                globals.netatom[NetAtom::ActiveWindow as usize],
            )
        };
    }
    unsafe { globals.selmon.as_mut().sel = c };
    drawbars(globals);
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
            globals.wmatom[WMAtom::State as usize],
            0,
            2,
            0,
            globals.wmatom[WMAtom::State as usize],
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

fn updatetitle(c: &mut Client, globals: &Globals) {
    if !gettextprop(
        c.win,
        globals.netatom[NetAtom::WMName as usize],
        c.name.as_mut_ptr(),
        c.name.len() as u32,
        globals,
    ) {
        gettextprop(
            c.win,
            XA_WM_NAME,
            c.name.as_mut_ptr(),
            c.name.len() as u32,
            globals,
        );
    }
    if c.name[0] == b'\0' as i8 {
        unsafe { libc::strcpy(c.name.as_mut_ptr(), BROKEN.as_ptr()) };
    }
}

fn applyrules(c: &mut Client, globals: &Globals) {
    let mut ch = XClassHint {
        res_name: core::ptr::null_mut(),
        res_class: core::ptr::null_mut(),
    };
    c.isfloating = false;
    c.tags = 0;
    unsafe { XGetClassHint(globals.dpy.as_ptr(), c.win, &mut ch) };
    let class = if !ch.res_class.is_null() {
        ch.res_class
    } else {
        BROKEN.as_ptr()
    };
    let instance = if !ch.res_name.is_null() {
        ch.res_name
    } else {
        BROKEN.as_ptr()
    };

    for r in config::RULES.iter() {
        let r_title = if !r.title.is_empty() {
            !unsafe {
                libc::strstr(
                    c.name.as_ptr(),
                    CString::new(r.title).expect("valid CString").as_ptr(),
                )
                .is_null()
            }
        } else {
            true
        };
        let r_class = if !r.class.is_empty() {
            !unsafe {
                libc::strstr(
                    class,
                    CString::new(r.class).expect("valid CString").as_ptr(),
                )
                .is_null()
            }
        } else {
            true
        };
        let r_instance = if !r.instance.is_empty() {
            !unsafe {
                libc::strstr(
                    instance,
                    CString::new(r.instance).expect("valid CString").as_ptr(),
                )
                .is_null()
            }
        } else {
            true
        };
        if r_title && r_class && r_instance {
            c.isterminal = r.isterminal;
            c.noswallow = r.noswallow;
            c.isfloating = r.isfloating;
            c.tags |= r.tags;

            if r.tags & SPTAGMASK != 0 && r.isfloating {
                c.x = unsafe { c.mon.as_ref().wx }
                    + (unsafe { c.mon.as_ref().ww } / 2 - c.width() / 2);
                c.y = unsafe { c.mon.as_ref().wy }
                    + (unsafe { c.mon.as_ref().wh } / 2 - c.height() / 2);
            }

            let mut m = Some(globals.mons);
            while let Some(m_inner) = m
                && unsafe { m_inner.as_ref().num } != r.monitor
            {
                m = unsafe { m_inner.as_ref() }.next;
            }
            if let Some(m) = m {
                c.mon = m;
            }
        }
    }

    if !ch.res_class.is_null() {
        unsafe { XFree(ch.res_class.cast_mut().cast()) };
    }
    if !ch.res_name.is_null() {
        unsafe { XFree(ch.res_name.cast_mut().cast()) };
    }

    c.tags = if c.tags & TAGMASK != 0 {
        c.tags & TAGMASK
    } else {
        (unsafe { c.mon.as_ref().tagset })[unsafe { c.mon.as_ref().seltags } as usize] & !SPTAGMASK
    };
}

fn updatesizehints(c: &mut Client, globals: &Globals) {
    const P_SIZE: i64 = 1 << 3;
    const P_MIN_SIZE: i64 = 1 << 4;
    const P_MAX_SIZE: i64 = 1 << 5;
    const P_RESIZE_INC: i64 = 1 << 6;
    const P_ASPECT: i64 = 1 << 7;
    const P_BASE_SIZE: i64 = 1 << 8;

    let mut size: MaybeUninit<XSizeHints> = MaybeUninit::uninit();
    let mut msize = 0i64;

    let hint_result = unsafe {
        XGetWMNormalHints(
            globals.dpy.as_ptr(),
            c.win,
            &mut size as *mut _ as *mut _,
            &mut msize,
        )
    } != 0;
    let mut size = unsafe { size.assume_init() };
    if !hint_result {
        size.flags = P_SIZE;
    }
    if size.flags & P_BASE_SIZE != 0 {
        c.basew = size.base_width;
        c.baseh = size.base_height;
    } else if size.flags & P_MIN_SIZE != 0 {
        c.basew = size.min_width;
        c.baseh = size.min_height;
    } else {
        c.basew = 0;
        c.baseh = 0;
    }
    if size.flags & P_RESIZE_INC != 0 {
        c.incw = size.width_inc;
        c.inch = size.height_inc;
    } else {
        c.incw = 0;
        c.inch = 0;
    }
    if size.flags & P_MAX_SIZE != 0 {
        c.maxw = size.max_width;
        c.maxh = size.max_height;
    } else {
        c.maxw = 0;
        c.maxh = 0;
    }
    if size.flags & P_MIN_SIZE != 0 {
        c.minw = size.min_width;
        c.minh = size.min_height;
    } else if size.flags & P_BASE_SIZE != 0 {
        c.minw = size.base_width;
        c.minh = size.base_height;
    } else {
        c.minw = 0;
        c.minh = 0;
    }
    if size.flags & P_ASPECT != 0 {
        c.mina = size.min_aspect.y as f32 / size.min_aspect.x as f32;
        c.maxa = size.max_aspect.x as f32 / size.max_aspect.y as f32;
    } else {
        c.maxa = 0.0;
        c.mina = 0.0;
    }
    c.isfixed = c.maxw != 0 && c.maxh != 0 && c.maxw == c.minw && c.maxh == c.minh;
    c.hintsvalid = true;
}

fn applysizehints(
    c: &mut Client,
    x: &mut i32,
    y: &mut i32,
    w: &mut i32,
    h: &mut i32,
    interact: bool,
    globals: &Globals,
) -> bool {
    // Read the monitor fields up front before any mutation of c.
    let (m_wx, m_wy, m_ww, m_wh, _m_sellt, m_lt_has_arrange) = unsafe {
        let m = c.mon.as_ref();
        (
            m.wx,
            m.wy,
            m.ww,
            m.wh,
            m.sellt as usize,
            m.lt[m.sellt as usize].arrange.is_none(),
        )
    };

    *w = 1.max(*w);
    *h = 1.max(*h);
    if interact {
        if *x > globals.sw {
            *x = globals.sw - c.width();
        }
        if *y > globals.sh {
            *y = globals.sh - c.height();
        }
        if *x + *w + 2 * c.bw < 0 {
            *x = 0;
        }
        if *y + *h + 2 * c.bw < 0 {
            *y = 0
        }
    } else {
        if *x >= m_wx + m_ww {
            *x = m_wx + m_ww - c.width();
        }
        if *y >= m_wy + m_wh {
            *y = m_wy + m_wh - c.height();
        }
        if *x + *w + 2 * c.bw <= m_wx {
            *x = m_wx;
        }
        if *y + *h + 2 * c.bw <= m_wy {
            *y = m_wy;
        }
    }
    if *h < globals.bh {
        *h = globals.bh;
    }
    if *w < globals.bh {
        *w = globals.bh;
    }
    // m_lt_has_arrange reflects m.lt[sellt].arrange.is_none() read before any mutation
    if load_resource!("RESIZE_HINTS", globals, Bool) || c.isfloating || m_lt_has_arrange {
        if !c.hintsvalid {
            updatesizehints(c, globals)
        }
        /* see last two sentences in ICCCM 4.1.2.3 */
        let baseismin = c.basew == c.minw && c.baseh == c.minh;
        if !baseismin {
            /* temporarily remove base dimensions */
            *w -= c.basew;
            *h -= c.baseh;
        }
        /* adjust for aspect limits */
        if c.mina > 0.0 && c.maxa > 0.0 {
            if c.maxa < *w as f32 / *h as f32 {
                *w = (*h as f32 * c.maxa + 0.5) as i32;
            } else if c.mina < *h as f32 / *w as f32 {
                *h = (*w as f32 * c.mina + 0.5) as i32;
            }
        }
        if baseismin {
            /* increment calculation requires this */
            *w -= c.basew;
            *h -= c.baseh;
        }
        /* adjust for increment value */
        if c.incw != 0 {
            *w -= *w % c.incw;
        }
        if c.inch != 0 {
            *h -= *h % c.inch;
        }
        /* restore base dimensions */
        *w = (*w + c.basew).max(c.minw);
        *h = (*h + c.baseh).max(c.minh);
        if c.maxw != 0 {
            *w = (*w).min(c.maxw);
        }
        if c.maxh != 0 {
            *h = (*h).min(c.maxh);
        }
    }

    *x != c.x || *y != c.y || *w != c.w || *h != c.h
}

fn configure(c: &Client, globals: &Globals) {
    const CONFIGURE_NOTIFY: i32 = 22;
    let mut ce = XConfigureEvent {
        r#type: CONFIGURE_NOTIFY,
        serial: 0,
        send_event: 0,
        display: globals.dpy.as_ptr(),
        event: c.win,
        window: c.win,
        x: c.x,
        y: c.y,
        width: c.w,
        height: c.h,
        border_width: c.bw,
        above: 0,
        override_redirect: 0,
    };
    unsafe {
        XSendEvent(
            globals.dpy.as_ptr(),
            c.win,
            0,
            STRUCTURE_NOTIFY_MASK,
            (&mut ce) as *mut _ as *mut XEvent,
        )
    };
}

fn getatomprop(c: &Client, prop: Atom, globals: &Globals) -> Atom {
    let mut format = 0i32;
    let mut nitems = 0u64;
    let mut dl = 0u64;
    let mut p: *mut u8 = core::ptr::null_mut();
    let mut da: Atom = 0;

    let mut atom = 0;

    if unsafe {
        XGetWindowProperty(
            globals.dpy.as_ptr(),
            c.win,
            prop,
            0,
            core::mem::size_of::<Atom>() as i64,
            0,
            XA_ATOM,
            &mut da,
            &mut format,
            &mut nitems,
            &mut dl,
            &mut p,
        )
    } == 0
        && !p.is_null()
    {
        if nitems > 0 && format == 32 {
            atom = unsafe { *p.cast::<u64>() }
        }
        unsafe { XFree(p.cast()) };
    }

    atom
}

fn resizeclient(c: &mut Client, x: i32, y: i32, w: i32, h: i32, globals: &Globals) {
    let mut wc = XWindowChanges {
        x,
        y,
        width: w,
        height: h,
        border_width: 0,
        sibling: 0,
        stack_mode: 0,
    };
    c.oldx = c.x;
    c.x = wc.x;
    c.oldy = c.y;
    c.y = wc.y;
    c.oldw = c.w;
    c.w = wc.width;
    c.oldh = c.h;
    c.h = wc.height;
    wc.border_width = c.bw;
    unsafe {
        XConfigureWindow(
            globals.dpy.as_ptr(),
            c.win,
            CWX | CWY | CW_WIDTH | CW_HEIGHT | CW_BORDER_WIDTH,
            &mut wc,
        )
    };
    configure(c, globals);
    unsafe { XSync(globals.dpy.as_ptr(), 0) };
}

fn resize(
    c: &mut Client,
    mut x: i32,
    mut y: i32,
    mut w: i32,
    mut h: i32,
    interact: bool,
    globals: &Globals,
) {
    if applysizehints(c, &mut x, &mut y, &mut w, &mut h, interact, globals) {
        resizeclient(c, x, y, w, h, globals);
    }
}

fn showhide(c: Option<NonNull<Client>>, globals: &Globals) {
    let Some(mut c) = c else { return };
    let c_ref = unsafe { c.as_mut() };
    let vis = is_visible(c);
    if vis {
        if (c_ref.tags & SPTAGMASK) != 0 && c_ref.isfloating {
            c_ref.x = unsafe { c_ref.mon.as_ref().wx }
                + (unsafe { c_ref.mon.as_ref().ww } / 2 - c_ref.width() / 2);
            c_ref.y = unsafe { c_ref.mon.as_ref().wy }
                + (unsafe { c_ref.mon.as_ref().wh } / 2 - c_ref.height() / 2);
        }
        /* show clients top down */
        unsafe { XMoveWindow(globals.dpy.as_ptr(), c_ref.win, c_ref.x, c_ref.y) };
        if (unsafe { c_ref.mon.as_ref() }.lt[unsafe { c_ref.mon.as_ref() }.sellt as usize]
            .arrange
            .is_none()
            || c_ref.isfloating)
            && !c_ref.isfullscreen
        {
            let (x, y, w, h) = (c_ref.x, c_ref.y, c_ref.w, c_ref.h);
            resize(unsafe { c.as_mut() }, x, y, w, h, false, globals);
        }
        showhide(c_ref.snext, globals);
    } else {
        showhide(c_ref.snext, globals);
        unsafe { XMoveWindow(globals.dpy.as_ptr(), c_ref.win, c_ref.width() * -2, c_ref.y) };
    }
}

fn restack(m: &Monitor, globals: &mut Globals) {
    const BELOW: i32 = 1;
    let mut wc: XWindowChanges = XWindowChanges {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
        border_width: 0,
        sibling: 0,
        stack_mode: 0,
    };

    let mut ev: XEvent = unsafe { core::mem::zeroed() };

    drawbar(m, globals);
    if m.sel.is_none() {
        return;
    }
    if let Some(sel) = m.sel
        && (unsafe { sel.as_ref() }.isfloating || m.lt[m.sellt as usize].arrange.is_none())
    {
        unsafe { XRaiseWindow(globals.dpy.as_ptr(), sel.as_ref().win) };
    }
    if m.lt[m.sellt as usize].arrange.is_some() {
        wc.stack_mode = BELOW;
        wc.sibling = m.barwin;
        let mut c = m.stack;
        while let Some(c_inner) = c {
            if !unsafe { c_inner.as_ref() }.isfloating && is_visible(c_inner) {
                unsafe {
                    XConfigureWindow(
                        globals.dpy.as_ptr(),
                        c_inner.as_ref().win,
                        CW_SIBLING | CW_STACK_MODE,
                        &mut wc,
                    )
                };
                wc.sibling = unsafe { c_inner.as_ref().win };
            }
            c = unsafe { c_inner.as_ref() }.snext;
        }
    }
    unsafe { XSync(globals.dpy.as_ptr(), 0) };
    while unsafe { XCheckMaskEvent(globals.dpy.as_ptr(), ENTER_WINDOW_MASK, &mut ev) } != 0 {}
}

fn arrange(mut m: Option<NonNull<Monitor>>, globals: &mut Globals) {
    if let Some(m) = m {
        showhide(unsafe { m.as_ref().stack }, globals);
    } else {
        m = Some(globals.mons);
        while let Some(m_inner) = m {
            showhide(unsafe { m_inner.as_ref() }.stack, globals);
            m = unsafe { m_inner.as_ref() }.next;
        }
    }

    if let Some(mut m) = m {
        arrangemon(unsafe { m.as_mut() }, globals);
        restack(unsafe { m.as_ref() }, globals);
    } else {
        m = Some(globals.mons);
        while let Some(mut m_inner) = m {
            arrangemon(unsafe { m_inner.as_mut() }, globals);
            m = unsafe { m_inner.as_ref() }.next;
        }
    }
}

fn arrangemon(m: &mut Monitor, globals: &mut Globals) {
    let symbol = CString::new(m.lt[m.sellt as usize].symbol).expect("valid CString");
    unsafe { libc::strncpy(m.ltsymbol.as_mut_ptr(), symbol.as_ptr(), m.ltsymbol.len()) };
    if let Some(f) = m.lt[m.sellt as usize].arrange {
        f(m, globals)
    }
}

fn setclientstate(c: &Client, state: i64, globals: &Globals) {
    let data = [state, 0];

    unsafe {
        XChangeProperty(
            globals.dpy.as_ptr(),
            c.win,
            globals.wmatom[WMAtom::State as usize],
            globals.wmatom[WMAtom::State as usize],
            32,
            PROP_MODE_REPLACE,
            (&data) as *const _ as *const u8,
            2,
        );
    }
}

fn setfullscreen(c: &mut Client, fullscreen: bool, globals: &mut Globals) {
    if fullscreen && !c.isfullscreen {
        unsafe {
            XChangeProperty(
                globals.dpy.as_ptr(),
                c.win,
                globals.netatom[NetAtom::WMState as usize],
                XA_ATOM,
                32,
                PROP_MODE_REPLACE,
                &globals.netatom[NetAtom::WMFullscreen as usize] as *const _ as *const u8,
                1,
            )
        };
        c.isfullscreen = true;
        c.oldstate = c.isfloating;
        c.oldbw = c.bw;
        c.bw = 0;
        c.isfloating = true;
        let (mx, my, mw, mh) = unsafe {
            let m = c.mon.as_ref();
            (m.mx, m.my, m.mw, m.mh)
        };
        resizeclient(c, mx, my, mw, mh, globals);
        unsafe { XRaiseWindow(globals.dpy.as_ptr(), c.win) };
    } else if !fullscreen && c.isfullscreen {
        unsafe {
            XChangeProperty(
                globals.dpy.as_ptr(),
                c.win,
                globals.netatom[NetAtom::WMState as usize],
                XA_ATOM,
                32,
                PROP_MODE_REPLACE,
                core::ptr::null::<u8>(),
                0,
            )
        };
        c.isfullscreen = false;
        c.isfloating = c.oldstate;
        c.bw = c.oldbw;
        c.x = c.oldx;
        c.y = c.oldy;
        c.w = c.oldw;
        c.h = c.oldh;
        let (x, y, w, h, mon) = (c.x, c.y, c.w, c.h, c.mon);
        resizeclient(c, x, y, w, h, globals);
        arrange(Some(mon), globals);
    }
}

fn updatewindowtype(mut c: NonNull<Client>, globals: &mut Globals) {
    let state: Atom = getatomprop(
        unsafe { c.as_ref() },
        globals.netatom[NetAtom::WMState as usize],
        globals,
    );
    let wtype: Atom = getatomprop(
        unsafe { c.as_ref() },
        globals.netatom[NetAtom::WMWindowType as usize],
        globals,
    );

    if state == globals.netatom[NetAtom::WMFullscreen as usize] {
        setfullscreen(unsafe { c.as_mut() }, true, globals)
    }
    if state == globals.netatom[NetAtom::WMSticky as usize] {
        setsticky(unsafe { c.as_mut() }, true, globals);
    }
    if wtype == globals.netatom[NetAtom::WMWindowTypeDialog as usize] {
        unsafe { c.as_mut().isfloating = true };
    }
}

fn shift(tag: u32, i: i32) -> u32 {
    if i > 0 {
        (tag << i as u32) | (tag >> (config::TAGS.len() as u32 - i as u32))
    } else {
        (tag >> (-i) as u32) | (tag << (config::TAGS.len() as u32 - (-i) as u32))
    }
}

#[allow(dead_code)]
fn shifttag(arg: &Arg, globals: &mut Globals) {
    let mut shifted = unsafe { globals.selmon.as_ref() }.tagset
        [unsafe { globals.selmon.as_ref() }.seltags as usize];

    if unsafe { globals.selmon.as_ref() }.clients.is_none() {
        return;
    }
    let Arg::I(ai) = arg else {
        unreachable!("invalid argument type to shifttag function")
    };
    shifted = shift(shifted, *ai);
    tag(&Arg::Ui(shifted), globals);
}

fn shifttagclients(arg: &Arg, globals: &mut Globals) {
    let mut shifted = unsafe { globals.selmon.as_ref() }.tagset
        [unsafe { globals.selmon.as_ref() }.seltags as usize];
    let mut tagmask = 0u32;
    let mut c = unsafe { globals.selmon.as_ref() }.clients;
    while let Some(c_inner) = c {
        tagmask |= unsafe { c_inner.as_ref() }.tags;
        c = unsafe { c_inner.as_ref() }.next;
    }

    let Arg::I(ai) = arg else {
        unreachable!("invalid argument type to shifttagclients function")
    };

    loop {
        shifted = shift(shifted, *ai);
        if tagmask == 0 || shifted & tagmask != 0 {
            break;
        }
    }
    tag(&Arg::Ui(shifted), globals);
}

#[allow(dead_code)]
fn shiftview(arg: &Arg, globals: &mut Globals) {
    let mut shifted = unsafe { globals.selmon.as_ref() }.tagset
        [unsafe { globals.selmon.as_ref() }.seltags as usize];

    let Arg::I(ai) = arg else {
        unreachable!("invalid argument type to shiftview function")
    };
    shifted = shift(shifted, *ai);
    view(&Arg::Ui(shifted), globals);
}

fn shiftviewclients(arg: &Arg, globals: &mut Globals) {
    let mut shifted = unsafe { globals.selmon.as_ref() }.tagset
        [unsafe { globals.selmon.as_ref() }.seltags as usize];
    let mut tagmask = 0u32;
    let mut c = unsafe { globals.selmon.as_ref() }.clients;
    while let Some(c_inner) = c {
        tagmask |= unsafe { c_inner.as_ref() }.tags;
        c = unsafe { c_inner.as_ref() }.next;
    }

    let Arg::I(ai) = arg else {
        unreachable!("invalid argument type to shifttagview function")
    };

    loop {
        shifted = shift(shifted, *ai);
        if tagmask == 0 || shifted & tagmask != 0 {
            break;
        }
    }
    view(&Arg::Ui(shifted), globals);
}

#[allow(dead_code)]
fn shiftboth(arg: &Arg, globals: &mut Globals) {
    let mut shifted = unsafe { globals.selmon.as_ref() }.tagset
        [unsafe { globals.selmon.as_ref() }.seltags as usize];

    let Arg::I(ai) = arg else {
        unreachable!("invalid argument type to shiftboth function")
    };
    shifted = shift(shifted, *ai);
    tag(&Arg::Ui(shifted), globals);
    view(&Arg::Ui(shifted), globals);
}

fn swaptags(arg: &Arg, globals: &mut Globals) {
    let Arg::Ui(ui) = arg else {
        unreachable!("invalid argument type to swaptags function")
    };
    let newtag = *ui & TAGMASK;
    let curtag = unsafe { globals.selmon.as_ref() }.tagset
        [unsafe { globals.selmon.as_ref() }.seltags as usize];

    if newtag == curtag || curtag == 0 || (curtag & (curtag - 1)) != 0 {
        return;
    }

    let mut c = unsafe { globals.selmon.as_ref() }.clients;
    while let Some(mut c_inner) = c {
        if unsafe { c_inner.as_ref() }.tags & newtag != 0
            || unsafe { c_inner.as_ref() }.tags & curtag != 0
        {
            unsafe { c_inner.as_mut() }.tags ^= curtag ^ newtag;
        }
        if unsafe { c_inner.as_ref() }.tags == 0 {
            unsafe { c_inner.as_mut() }.tags = newtag;
        }

        c = unsafe { c_inner.as_ref() }.next;
    }

    //uncomment to 'view' the new swaped tag
    // unsafe { globals.selmon.as_mut() }.tagset
    //     [unsafe { globals.selmon.as_ref() }.seltags as usize] = newtag;

    focus(None, globals);
    arrange(Some(globals.selmon), globals);
}

#[allow(dead_code)]
fn shiftswaptags(arg: &Arg, globals: &mut Globals) {
    let mut shifted = unsafe { globals.selmon.as_ref() }.tagset
        [unsafe { globals.selmon.as_ref() }.seltags as usize];

    let Arg::I(ai) = arg else {
        unreachable!("invalid argument type to shiftswaptags function")
    };
    shifted = shift(shifted, *ai);
    swaptags(&Arg::Ui(shifted), globals);
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

    updatetitle(c_ref, globals);

    if unsafe { XGetTransientForHint(globals.dpy.as_ptr(), w, &mut trans) } != 0
        && let Some(t) = wintoclient(trans, globals)
    {
        let t_ref = unsafe { t.as_ref() };
        c_ref.mon = t_ref.mon;
        c_ref.tags = t_ref.tags;
    } else {
        c_ref.mon = globals.selmon;
        applyrules(c_ref, globals);
        term = termforwin(c, globals);
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
            globals.scheme[SchemeState::Norm as usize][drw::COL_BORDER].pixel,
        )
    };
    configure(c_ref, globals); /* propagates border_width, if size doesn't change */
    updatewindowtype(c, globals);
    updatesizehints(c_ref, globals);
    updatewmhints(c_ref, globals);
    unsafe {
        XSelectInput(
            globals.dpy.as_ptr(),
            w,
            ENTER_WINDOW_MASK | FOCUS_CHANGE_MASK | PROPERTY_CHANGE_MASK | STRUCTURE_NOTIFY_MASK,
        )
    };
    grabbuttons(c_ref, false, globals);
    if !unsafe { c.as_ref() }.isfloating {
        unsafe { c.as_mut().oldstate = trans != 0 || c.as_ref().isfixed };
        unsafe { c.as_mut().isfloating = c.as_ref().oldstate };
    }
    if unsafe { c.as_ref() }.isfloating {
        unsafe { XRaiseWindow(globals.dpy.as_ptr(), c.as_ref().win) };
    }
    attach(c);
    attachstack(c);
    unsafe {
        XChangeProperty(
            globals.dpy.as_ptr(),
            globals.root,
            globals.netatom[NetAtom::ClientList as usize],
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
    setclientstate(c_ref, NORMAL_STATE as i64, globals);
    if unsafe { c.as_ref() }.mon == globals.selmon {
        unfocus(unsafe { globals.selmon.as_ref() }.sel, false, globals);
    }
    unsafe { c.as_mut().mon.as_mut() }.sel = Some(c);
    arrange(Some(unsafe { c.as_ref() }.mon), globals);
    unsafe { XMapWindow(globals.dpy.as_ptr(), c.as_ref().win) };

    if let Some(term) = term {
        swallow(term, c, globals);
    }

    focus(None, globals);
}

fn updateclientlist(globals: &Globals) {
    unsafe {
        XDeleteProperty(
            globals.dpy.as_ptr(),
            globals.root,
            globals.netatom[NetAtom::ClientList as usize],
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
                    globals.netatom[NetAtom::ClientList as usize],
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

fn unmanage(c: NonNull<Client>, destroyed: bool, globals: &mut Globals) {
    let m = unsafe { c.as_ref() }.mon;
    let mut wc: XWindowChanges = unsafe { core::mem::zeroed() };

    if unsafe { c.as_ref().swallowing.is_some() } {
        unswallow(c, globals);
        return;
    }

    let s: Option<NonNull<Client>> = swallowingclient(unsafe { c.as_ref().win }, globals);
    if let Some(mut s) = s {
        let swallowing = unsafe {
            s.as_mut()
                .swallowing
                .take()
                .expect("swallowingclient only returns s when s.swallowing.is_some()")
        };
        let _ = unsafe { Box::from_raw(swallowing.as_ptr()) };
        arrange(Some(m), globals);
        focus(None, globals);
        return;
    }

    detach(c);
    detachstack(c);
    if !destroyed {
        wc.border_width = unsafe { c.as_ref() }.oldbw;
        unsafe {
            XGrabServer(globals.dpy.as_ptr());
            XSetErrorHandler(xerrordummy);
            XSelectInput(globals.dpy.as_ptr(), c.as_ref().win, NO_EVENT_MASK);

            XConfigureWindow(
                globals.dpy.as_ptr(),
                c.as_ref().win,
                CW_BORDER_WIDTH,
                &mut wc,
            ); /* restore border */

            XUngrabButton(
                globals.dpy.as_ptr(),
                ANY_BUTTON,
                ANY_MODIFIER,
                c.as_ref().win,
            );
        }
        setclientstate(unsafe { c.as_ref() }, WITHDRAWN_STATE as i64, globals);
        unsafe {
            XSync(globals.dpy.as_ptr(), 0);
            XSetErrorHandler(xerror);
            XUngrabServer(globals.dpy.as_ptr());
        }
    }

    unsafe {
        let _ = Box::from_raw(c.as_ptr());
    }
    //NOTE: swallowing patch has a check here if to only run this is s is none
    //but if s is some we will have returned already above. So not possible for
    //s to not be none in this case.
    focus(None, globals);
    updateclientlist(globals);
    arrange(Some(m), globals);
}

fn run(globals: &mut Globals) {
    let mut ev: XEvent = unsafe { core::mem::zeroed() };
    unsafe { XSync(globals.dpy.as_ptr(), 0) };
    while globals.running && unsafe { XNextEvent(globals.dpy.as_ptr(), &mut ev) } == 0 {
        if let Some(f) = HANDLER[unsafe { ev.r#type } as usize] {
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
        wmatom: [0; WMAtom::Last as usize],
        netatom: [0; NetAtom::Last as usize],
        running: false,
        cursor: [Cur { cursor: 0 }; CursorState::Last as usize],
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
    };

    updategeom(&mut globals);
    globals.running = true;

    let utf8string: Atom;

    unsafe {
        utf8string = XInternAtom(dpy.as_ptr(), c"UTF8_STRING".as_ptr(), 0);
        globals.wmatom[WMAtom::Protocols as usize] =
            XInternAtom(dpy.as_ptr(), c"WM_PROTOCOLS".as_ptr(), 0);
        globals.wmatom[WMAtom::Delete as usize] =
            XInternAtom(dpy.as_ptr(), c"WM_DELETE_WINDOW".as_ptr(), 0);
        globals.wmatom[WMAtom::State as usize] = XInternAtom(dpy.as_ptr(), c"WM_STATE".as_ptr(), 0);
        globals.wmatom[WMAtom::TakeFocus as usize] =
            XInternAtom(dpy.as_ptr(), c"WM_TAKE_FOCUS".as_ptr(), 0);
        globals.netatom[NetAtom::ActiveWindow as usize] =
            XInternAtom(dpy.as_ptr(), c"_NET_ACTIVE_WINDOW".as_ptr(), 0);
        globals.netatom[NetAtom::Supported as usize] =
            XInternAtom(dpy.as_ptr(), c"_NET_SUPPORTED".as_ptr(), 0);
        globals.netatom[NetAtom::WMName as usize] =
            XInternAtom(dpy.as_ptr(), c"_NET_WM_NAME".as_ptr(), 0);
        globals.netatom[NetAtom::WMState as usize] =
            XInternAtom(dpy.as_ptr(), c"_NET_WM_STATE".as_ptr(), 0);
        globals.netatom[NetAtom::WMCheck as usize] =
            XInternAtom(dpy.as_ptr(), c"_NET_SUPPORTING_WM_CHECK".as_ptr(), 0);
        globals.netatom[NetAtom::WMFullscreen as usize] =
            XInternAtom(dpy.as_ptr(), c"_NET_WM_STATE_FULLSCREEN".as_ptr(), 0);
        globals.netatom[NetAtom::WMSticky as usize] =
            XInternAtom(dpy.as_ptr(), c"_NET_WM_STATE_STICKY".as_ptr(), 0);
        globals.netatom[NetAtom::WMWindowType as usize] =
            XInternAtom(dpy.as_ptr(), c"_NET_WM_WINDOW_TYPE".as_ptr(), 0);
        globals.netatom[NetAtom::WMWindowTypeDialog as usize] =
            XInternAtom(dpy.as_ptr(), c"_NET_WM_WINDOW_TYPE_DIALOG".as_ptr(), 0);
        globals.netatom[NetAtom::ClientList as usize] =
            XInternAtom(dpy.as_ptr(), c"_NET_CLIENT_LIST".as_ptr(), 0);
    }

    globals.cursor[CursorState::Normal as usize] = globals.drw.cur_create(XC_LEFT_PTR);
    globals.cursor[CursorState::Resize as usize] = globals.drw.cur_create(XC_SIZING);
    globals.cursor[CursorState::Move as usize] = globals.drw.cur_create(XC_FLEUR);

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
            globals.netatom[NetAtom::WMCheck as usize],
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
            globals.netatom[NetAtom::WMName as usize],
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
            globals.netatom[NetAtom::WMCheck as usize],
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
            globals.netatom[NetAtom::Supported as usize],
            XA_ATOM,
            32,
            PROP_MODE_REPLACE,
            (&globals.netatom) as *const u64 as *const u8,
            NetAtom::Last as i32,
        )
    };
    unsafe {
        XDeleteProperty(
            dpy.as_ptr(),
            root,
            globals.netatom[NetAtom::ClientList as usize],
        )
    };

    let mut wa: XSetWindowAttributes = unsafe { core::mem::zeroed() };

    wa.cursor = globals.cursor[CursorState::Normal as usize].cursor;
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
    focus(None, &mut globals);
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

    view(&a, &mut globals);
    (unsafe { globals.selmon.as_mut().lt })[unsafe { globals.selmon.as_ref().sellt } as usize] =
        &EMPTY_LAYOUT;

    //cleanup clients
    let mut m = Some(globals.mons);
    while let Some(m_inner) = m {
        while let Some(stack) = unsafe { m_inner.as_ref() }.stack {
            unmanage(stack, false, &mut globals)
        }
        m = unsafe { m_inner.as_ref() }.next;
    }

    //cleanup monitors
    unsafe { XUngrabKey(globals.dpy.as_ptr(), ANY_KEY, ANY_MODIFIER, globals.root) };
    globals.selmon = NonNull::dangling(); // prevent use-after-free: monitors are freed next
    while !cleanupmon(globals.mons, &mut globals) {}

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
    unsafe { XDeleteProperty(dpy.as_ptr(), root, netatom[NetAtom::ActiveWindow as usize]) };
    dpy.as_ptr()
}

fn cleanupmon(mon: NonNull<Monitor>, globals: &mut Globals) -> bool {
    let mut done = false;
    if mon == globals.mons {
        globals.mons = match unsafe { globals.mons.as_ref() }.next {
            Some(m) => m,
            None => {
                done = true;
                NonNull::dangling()
            }
        };
    } else {
        let mut m = Some(globals.mons);
        while let Some(m_inner) = m
            && let Some(next) = unsafe { m_inner.as_ref() }.next
            && next != mon
        {
            m = unsafe { m_inner.as_ref() }.next;
        }
        unsafe { m.expect("should be a valid reference").as_mut() }.next =
            unsafe { mon.as_ref() }.next;
    }
    unsafe { XUnmapWindow(globals.dpy.as_ptr(), mon.as_ref().barwin) };
    unsafe { XDestroyWindow(globals.dpy.as_ptr(), mon.as_ref().barwin) };
    unsafe {
        let _ = Box::from_raw(mon.as_ptr());
    }
    done
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
    let resources = load_xresources();
    let mut globals = setup(dpy, resources, xcon);
    scan(&mut globals);
    run(&mut globals);
    let dpy: *mut Display = cleanup(globals);
    unsafe { XCloseDisplay(dpy) };
}
