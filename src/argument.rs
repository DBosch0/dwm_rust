use std::ffi::{CString, c_void};

use crate::{
    CURSOR_STATE_MOVE, CURSOR_STATE_RESIZE, Globals, Layout, MOUSE_MASK, PREV_SEL, TAGMASK,
    WM_DELETE,
    client::Client,
    die,
    event::ClickState,
    external_functions::{
        BUTTON_RELEASE, CONFIGURE_REQUEST, CURRENT_TIME, ENTER_WINDOW_MASK, EXPOSE, EXPOSURE_MASK,
        GRAB_MODE_ASYNC, KeySym, MAP_REQUEST, MOTION_NOTIFY, SUBSTRUCTURE_REDIRECT_MASK, Time,
        XCheckMaskEvent, XEvent, XGrabPointer, XGrabServer, XKillClient, XMaskEvent,
        XMoveResizeWindow, XSetCloseDownMode, XSetErrorHandler, XSync, XUngrabPointer,
        XUngrabServer, XWarpPointer, connection_number,
    },
    load_resource,
    monitor::Monitor,
    resource::ResourceVal,
    util::{shift, sptag},
};

pub(crate) enum Arg {
    I(i32),
    Ui(u32),
    F(f32),
    Command(&'static [&'static str]),
    Layout(Option<&'static Layout>),
}

pub(crate) struct Button {
    pub(crate) click: ClickState,
    pub(crate) mask: u32,
    pub(crate) button: u32,
    pub(crate) func: Option<ArgumentFunction>,
    pub(crate) arg: Arg,
}

pub(crate) struct Key {
    pub(crate) r#mod: u32,
    pub(crate) keysym: KeySym,
    pub(crate) func: Option<ArgumentFunction>,
    pub(crate) arg: Arg,
}

pub(crate) type ArgumentFunction = fn(&Arg, &mut Globals);

impl Arg {
    pub(crate) fn view(&self, globals: &mut Globals) {
        let Arg::Ui(ui) = self else { unreachable!() };
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
        Client::focus(None, globals);
        Monitor::arrange(Some(globals.selmon), globals);
    }

    pub(crate) fn toggleview(&self, globals: &mut Globals) {
        let Arg::Ui(ui) = self else { unreachable!() };

        let newtagset = unsafe { globals.selmon.as_ref() }.tagset
            [unsafe { globals.selmon.as_ref() }.seltags as usize]
            ^ (*ui & TAGMASK);

        if newtagset != 0 {
            unsafe { globals.selmon.as_mut() }.tagset
                [unsafe { globals.selmon.as_ref() }.seltags as usize] = newtagset;
            Client::focus(None, globals);
            Monitor::arrange(Some(globals.selmon), globals);
        }
    }

    pub(crate) fn tag(&self, globals: &mut Globals) {
        let Arg::Ui(ui) = self else { unreachable!() };
        if let Some(mut sel) = unsafe { globals.selmon.as_ref() }.sel
            && *ui & TAGMASK != 0
        {
            unsafe { sel.as_mut() }.tags = *ui & TAGMASK;
            Client::focus(None, globals);
            Monitor::arrange(Some(globals.selmon), globals);
        }
    }

    pub(crate) fn togglesticky(&self, globals: &mut Globals) {
        if let Some(mut sel) = unsafe { globals.selmon.as_ref() }.sel {
            let sel = unsafe { sel.as_mut() };
            sel.setsticky(!sel.issticky, globals);
            Monitor::arrange(Some(globals.selmon), globals);
        }
    }

    pub(crate) fn toggletag(&self, globals: &mut Globals) {
        let Arg::Ui(ui) = self else { unreachable!() };
        if let Some(mut sel) = unsafe { globals.selmon.as_ref() }.sel {
            let newtags = unsafe { sel.as_ref() }.tags ^ (*ui & TAGMASK);
            if newtags != 0 {
                unsafe { sel.as_mut() }.tags = newtags;
                Client::focus(None, globals);
                Monitor::arrange(Some(globals.selmon), globals);
            }
        }
    }

    pub(crate) fn togglescratch(&self, globals: &mut Globals) {
        let mut found = false;
        let Arg::Ui(ui) = self else {
            unreachable!("invalid argument given to togglescratch function")
        };
        let scratchtag = sptag(*ui);
        let sparg = Arg::Command(crate::config::SCRATCHPADS[*ui as usize].cmd);

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
                Client::focus(None, globals);
                Monitor::arrange(Some(globals.selmon), globals);
            }
            if unsafe { c.as_ref() }.is_visible() {
                Client::focus(Some(c), globals);
                unsafe { globals.selmon.as_ref() }.restack(globals);
            }
        } else {
            let seltags_idx = unsafe { globals.selmon.as_ref().seltags } as usize;
            unsafe { globals.selmon.as_mut().tagset[seltags_idx] |= scratchtag };
            sparg.spawn(globals);
        }
    }

    //NOTE: using libc and not `std::process` because setsid in `std::os::linux::process::CommandExt` is unstable.
    //update when stable.
    pub(crate) fn spawn(&self, globals: &mut Globals) {
        let mut sa: libc::sigaction = unsafe { core::mem::zeroed() };
        let Arg::Command(arr) = self else {
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

    pub(crate) fn setlayout(&self, globals: &mut Globals) {
        let Arg::Layout(layout) = self else {
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
            Monitor::arrange(Some(globals.selmon), globals);
        } else {
            unsafe { globals.selmon.as_ref() }.drawbar(globals);
        }
    }

    pub(crate) fn quit(&self, globals: &mut Globals) {
        globals.running = false;
    }

    pub(crate) fn togglebar(&self, globals: &mut Globals) {
        let m = unsafe { globals.selmon.as_mut() };
        m.showbar = !m.showbar;
        m.updatebarpos(globals);
        unsafe {
            XMoveResizeWindow(
                globals.dpy.as_ptr(),
                m.barwin,
                m.wx,
                m.by,
                m.ww as u32,
                globals.bh as u32,
            )
        };
        Monitor::arrange(Some(globals.selmon), globals);
    }

    pub(crate) fn togglefloating(&self, globals: &mut Globals) {
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
            sel.resize(sel.x, sel.y, sel.w, sel.h, false, globals);
        }
        Monitor::arrange(Some(globals.selmon), globals);
    }

    pub(crate) fn togglefullscreen(&self, globals: &mut Globals) {
        if let Some(mut sel) = unsafe { globals.selmon.as_ref() }.sel {
            let sel = unsafe { sel.as_mut() };
            sel.setfullscreen(!sel.isfullscreen, globals);
        }
    }

    pub(crate) fn focusstack(&self, globals: &mut Globals) {
        let mut i = self.stackpos(globals);
        if i < 0 {
            return;
        }
        let mut p = None;
        let mut c = unsafe { globals.selmon.as_ref() }.clients;
        while let Some(c_inner) = c
            && (i > 0 || !unsafe { c_inner.as_ref() }.is_visible())
        {
            i -= if unsafe { c_inner.as_ref() }.is_visible() {
                1
            } else {
                0
            };
            p = c;
            c = unsafe { c_inner.as_ref() }.next;
        }
        Client::focus(if c.is_some() { c } else { p }, globals);
        unsafe { globals.selmon.as_ref() }.restack(globals);
    }

    pub(crate) fn pushstack(&self, globals: &mut Globals) {
        let mut i = self.stackpos(globals);

        if i < 0 {
            return;
        } else if i == 0 {
            let Some(sel) = unsafe { globals.selmon.as_ref() }.sel else {
                unreachable!("should be unreachable state due to pushstack")
            };
            Client::detach(sel);
            Client::attach(sel);
        } else {
            let Some(mut sel) = unsafe { globals.selmon.as_ref() }.sel else {
                unreachable!("should be unreachable state due to pushstack")
            };
            let mut p = None;
            let mut c = unsafe { globals.selmon.as_ref() }.clients;
            while let Some(c_inner) = c {
                i -= if unsafe { c_inner.as_ref() }.is_visible() && c_inner != sel {
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
            Client::detach(sel);
            unsafe { sel.as_mut() }.next = unsafe { c.as_ref() }.next;
            unsafe { c.as_mut() }.next = Some(sel);
        }
        Monitor::arrange(Some(globals.selmon), globals);
    }

    pub(crate) fn stackpos(&self, globals: &mut Globals) -> i32 {
        if unsafe { globals.selmon.as_ref() }.clients.is_none() {
            return -1;
        }
        let Arg::I(ai) = self else {
            unreachable!("invalid argument to stackpos function")
        };
        if *ai == PREV_SEL {
            let mut l = unsafe { globals.selmon.as_ref() }.stack;
            while let Some(l_inner) = l
                && (!unsafe { l_inner.as_ref() }.is_visible()
                    || l == unsafe { globals.selmon.as_ref() }.sel)
            {
                l = unsafe { l_inner.as_ref() }.snext
            }
            let Some(l) = l else { return -1 };
            let mut i = 0;
            let mut c = unsafe { globals.selmon.as_ref() }.clients;
            while let Some(c_inner) = c
                && c_inner != l
            {
                i += if unsafe { c_inner.as_ref() }.is_visible() {
                    1
                } else {
                    0
                };
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
                i += if unsafe { c_inner.as_ref() }.is_visible() {
                    1
                } else {
                    0
                };
                c = unsafe { c_inner.as_ref() }.next;
            }
            let mut n = i;
            while let Some(c_inner) = c {
                n += if unsafe { c_inner.as_ref() }.is_visible() {
                    1
                } else {
                    0
                };
                c = unsafe { c_inner.as_ref() }.next;
            }
            (i + (*ai - 2000)).rem_euclid(n)
        } else if *ai < 0 {
            let mut i = 0;
            let mut c = unsafe { globals.selmon.as_ref() }.clients;
            while let Some(c_inner) = c {
                i += if unsafe { c_inner.as_ref() }.is_visible() {
                    1
                } else {
                    0
                };
                c = unsafe { c_inner.as_ref() }.next;
            }
            (i + *ai).max(0)
        } else {
            *ai
        }
    }

    pub(crate) fn incnmaster(&self, globals: &mut Globals) {
        let Arg::I(i) = self else {
            unreachable!("invalid input to incnmaster")
        };
        unsafe { globals.selmon.as_mut() }.nmaster =
            (unsafe { globals.selmon.as_ref() }.nmaster + *i).max(0);
        Monitor::arrange(Some(globals.selmon), globals);
    }

    #[allow(dead_code)]
    pub(crate) fn setcfact(&self, globals: &mut Globals) {
        let c = unsafe { globals.selmon.as_ref() }.sel;

        if c.is_none()
            || unsafe { globals.selmon.as_ref() }.lt
                [unsafe { globals.selmon.as_ref() }.sellt as usize]
                .arrange
                .is_none()
        {
            return;
        }
        let mut c = c.expect("checked to be Some");

        let Arg::F(fa) = self else {
            unreachable!("invalid argument to setcfact function")
        };
        let mut f = *fa + unsafe { c.as_ref() }.cfact;
        if *fa == 0.0 {
            f = 1.0;
        } else if !(0.25..=4.0).contains(&f) {
            return;
        }
        unsafe { c.as_mut() }.cfact = f;
        Monitor::arrange(Some(globals.selmon), globals);
    }

    pub(crate) fn setmfact(&self, globals: &mut Globals) {
        if unsafe { globals.selmon.as_ref() }.lt[unsafe { globals.selmon.as_ref() }.sellt as usize]
            .arrange
            .is_none()
        {
            return;
        }
        let f = match self {
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
        Monitor::arrange(Some(globals.selmon), globals);
    }

    pub(crate) fn zoom(&self, globals: &mut Globals) {
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
        if c == Client::nexttiled(unsafe { globals.selmon.as_ref() }.clients) {
            c = Client::nexttiled(unsafe { c_inner.as_ref() }.next);
            if c.is_none() {
                return;
            }
            c_inner = c.expect("checked non none");
        }
        Client::pop(c_inner, globals)
    }

    pub(crate) fn xrdb(&self, globals: &mut Globals) {
        globals.resources = crate::resource::load_xresources();

        for (i, pallette) in crate::config::COLORS.iter().enumerate() {
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
            let pallette: [&str; crate::config::COLORS[0].len()] = std::array::from_fn(|_| {
                pallette_iter.next().expect(
                "we know by construction that there exists a constant number of values in the map",
            )
            });
            let mut scm = globals.drw.scm_create(&pallette);
            std::mem::swap(&mut scm, &mut globals.scheme[i]);
            globals.drw.scm_free(scm, false);
        }

        Client::focus(None, globals);
        Monitor::arrange(None, globals);
    }

    pub(crate) fn killclient(&self, globals: &mut Globals) {
        const DESTROY_ALL: i32 = 0;
        let Some(sel) = unsafe { globals.selmon.as_ref() }.sel else {
            return;
        };
        if !unsafe { sel.as_ref() }.sendevent(globals.wmatom[WM_DELETE], globals) {
            unsafe {
                XGrabServer(globals.dpy.as_ptr());
                XSetErrorHandler(crate::xerrordummy);
                XSetCloseDownMode(globals.dpy.as_ptr(), DESTROY_ALL);
                XKillClient(globals.dpy.as_ptr(), sel.as_ref().win);
                XSync(globals.dpy.as_ptr(), 0);
                XSetErrorHandler(crate::xerror);
                XUngrabServer(globals.dpy.as_ptr());
            }
        }
    }

    pub(crate) fn focusmon(&self, globals: &mut Globals) {
        if unsafe { globals.mons.as_ref() }.next.is_none() {
            return;
        }
        let Arg::I(i) = self else {
            unreachable!("invalid argument to focus mon")
        };
        let m = Monitor::dirtomon(*i, globals);
        if m == globals.selmon {
            return;
        }
        Client::unfocus(unsafe { globals.selmon.as_ref() }.sel, false, globals);
        globals.selmon = m;
        Client::focus(None, globals);
    }

    pub(crate) fn tagmon(&self, globals: &mut Globals) {
        if unsafe { globals.selmon.as_ref() }.sel.is_none()
            || unsafe { globals.mons.as_ref() }.next.is_none()
        {
            return;
        }
        let Arg::I(i) = self else {
            unreachable!("invalid argument to tagmon")
        };
        Client::sendmon(
            unsafe { globals.selmon.as_ref() }
                .sel
                .expect("checked above to be not None"),
            Monitor::dirtomon(*i, globals),
            globals,
        );
    }

    pub(crate) fn movemouse(&self, globals: &mut Globals) {
        const GRAB_SUCCESS: i32 = 0;

        let Some(mut c) = unsafe { globals.selmon.as_ref() }.sel else {
            return;
        };
        if unsafe { c.as_ref() }.isfullscreen {
            return;
        }
        let c_ref = unsafe { c.as_ref() };
        unsafe { globals.selmon.as_ref() }.restack(globals);
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
                globals.cursor[CURSOR_STATE_MOVE].cursor,
                CURRENT_TIME,
            )
        } != GRAB_SUCCESS
        {
            return;
        }
        let mut x = 0;
        let mut y = 0;
        let mut lasttime: Time = 0;
        if !globals.getrootptr(&mut x, &mut y) {
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
                    crate::event::event_handler(unsafe { ev.r#type }).expect("valid function")(
                        &mut ev, globals,
                    )
                }
                MOTION_NOTIFY => {
                    if unsafe { ev.xmotion }.time - lasttime
                        <= 1000 / crate::config::REFRESH_RATE as u64
                    {
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
                        Arg::I(0).togglefloating(globals);
                    }
                    if unsafe { globals.selmon.as_ref().lt }
                        [unsafe { globals.selmon.as_ref().sellt } as usize]
                        .arrange
                        .is_none()
                        || unsafe { c.as_ref().isfloating }
                    {
                        let c = unsafe { c.as_mut() };
                        // let (w, h) = unsafe { (c.as_ref().w, c.as_ref().h) };
                        c.resize(nx, ny, c.w, c.h, true, globals);
                    }
                }
                _ => {}
            }
            if unsafe { ev.r#type } == BUTTON_RELEASE {
                break;
            }
        }
        unsafe { XUngrabPointer(globals.dpy.as_ptr(), CURRENT_TIME) };
        let m = Monitor::recttomon(
            unsafe { c.as_ref() }.x,
            unsafe { c.as_ref() }.y,
            unsafe { c.as_ref() }.w,
            unsafe { c.as_ref() }.h,
            globals,
        );
        if m != globals.selmon {
            Client::sendmon(c, m, globals);
            globals.selmon = m;
            Client::focus(None, globals);
        }
    }

    pub(crate) fn resizemouse(&self, globals: &mut Globals) {
        const GRAB_SUCCESS: i32 = 0;

        let Some(mut c) = unsafe { globals.selmon.as_ref() }.sel else {
            return;
        };
        let cr = unsafe { c.as_ref() }; /* no support resizing fullscreen windows by mouse */
        if cr.isfullscreen {
            return;
        }
        unsafe { globals.selmon.as_ref() }.restack(globals);
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
                globals.cursor[CURSOR_STATE_RESIZE].cursor,
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
                    crate::event::event_handler(unsafe { ev.r#type }).expect("valid function")(
                        &mut ev, globals,
                    )
                }
                MOTION_NOTIFY => {
                    if unsafe { ev.xmotion.time } - lasttime
                        <= 1000 / crate::config::REFRESH_RATE as u64
                    {
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
                        && unsafe { cr.mon.as_ref().wy } + nh
                            >= unsafe { globals.selmon.as_ref().wy }
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
                        Arg::I(0).togglefloating(globals);
                    }

                    if unsafe { globals.selmon.as_ref().lt }
                        [unsafe { globals.selmon.as_ref().sellt } as usize]
                        .arrange
                        .is_none()
                        || cr.isfloating
                    {
                        unsafe { c.as_mut() }.resize(cr.x, cr.y, nw, nh, true, globals);
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
        let m = Monitor::recttomon(cr.x, cr.y, cr.w, cr.h, globals);
        if m != globals.selmon {
            Client::sendmon(c, m, globals);
            globals.selmon = m;
            Client::focus(None, globals);
        }
    }

    #[allow(dead_code)]
    pub(crate) fn sigstatusbar(&self, globals: &mut Globals) {
        let mut sv: libc::sigval = unsafe { core::mem::zeroed() };

        if globals.statussig == 0 {
            return;
        }
        let Arg::I(i) = self else {
            unreachable!("invalid argument to sigstatusbar")
        };
        sv.sival_ptr = (*i) as *mut c_void;
        let statuspid = globals.getstatusbarpid();
        if statuspid <= 0 {
            return;
        }

        unsafe { libc::sigqueue(statuspid, libc::SIGRTMIN() + globals.statussig, sv) };
    }

    #[allow(dead_code)]
    pub(crate) fn shifttag(&self, globals: &mut Globals) {
        let mut shifted = unsafe { globals.selmon.as_ref() }.tagset
            [unsafe { globals.selmon.as_ref() }.seltags as usize];

        if unsafe { globals.selmon.as_ref() }.clients.is_none() {
            return;
        }
        let Arg::I(ai) = self else {
            unreachable!("invalid argument type to shifttag function")
        };
        shifted = shift(shifted, *ai);
        Arg::Ui(shifted).tag(globals);
    }

    pub(crate) fn shifttagclients(&self, globals: &mut Globals) {
        let mut shifted = unsafe { globals.selmon.as_ref() }.tagset
            [unsafe { globals.selmon.as_ref() }.seltags as usize];
        let mut tagmask = 0u32;
        let mut c = unsafe { globals.selmon.as_ref() }.clients;
        while let Some(c_inner) = c {
            tagmask |= unsafe { c_inner.as_ref() }.tags;
            c = unsafe { c_inner.as_ref() }.next;
        }

        let Arg::I(ai) = self else {
            unreachable!("invalid argument type to shifttagclients function")
        };

        loop {
            shifted = shift(shifted, *ai);
            if tagmask == 0 || shifted & tagmask != 0 {
                break;
            }
        }
        Arg::Ui(shifted).tag(globals);
    }

    #[allow(dead_code)]
    pub(crate) fn shiftview(&self, globals: &mut Globals) {
        let mut shifted = unsafe { globals.selmon.as_ref() }.tagset
            [unsafe { globals.selmon.as_ref() }.seltags as usize];

        let Arg::I(ai) = self else {
            unreachable!("invalid argument type to shiftview function")
        };
        shifted = shift(shifted, *ai);
        Arg::Ui(shifted).view(globals);
    }

    pub(crate) fn shiftviewclients(&self, globals: &mut Globals) {
        let mut shifted = unsafe { globals.selmon.as_ref() }.tagset
            [unsafe { globals.selmon.as_ref() }.seltags as usize];
        let mut tagmask = 0u32;
        let mut c = unsafe { globals.selmon.as_ref() }.clients;
        while let Some(c_inner) = c {
            tagmask |= unsafe { c_inner.as_ref() }.tags;
            c = unsafe { c_inner.as_ref() }.next;
        }

        let Arg::I(ai) = self else {
            unreachable!("invalid argument type to shifttagview function")
        };

        loop {
            shifted = shift(shifted, *ai);
            if tagmask == 0 || shifted & tagmask != 0 {
                break;
            }
        }
        Arg::Ui(shifted).view(globals);
    }

    #[allow(dead_code)]
    pub(crate) fn shiftboth(&self, globals: &mut Globals) {
        let mut shifted = unsafe { globals.selmon.as_ref() }.tagset
            [unsafe { globals.selmon.as_ref() }.seltags as usize];

        let Arg::I(ai) = self else {
            unreachable!("invalid argument type to shiftboth function")
        };
        shifted = shift(shifted, *ai);
        Arg::Ui(shifted).tag(globals);
        Arg::Ui(shifted).view(globals);
    }

    pub(crate) fn swaptags(&self, globals: &mut Globals) {
        let Arg::Ui(ui) = self else {
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

        Client::focus(None, globals);
        Monitor::arrange(Some(globals.selmon), globals);
    }

    #[allow(dead_code)]
    pub(crate) fn shiftswaptags(&self, globals: &mut Globals) {
        let mut shifted = unsafe { globals.selmon.as_ref() }.tagset
            [unsafe { globals.selmon.as_ref() }.seltags as usize];

        let Arg::I(ai) = self else {
            unreachable!("invalid argument type to shiftswaptags function")
        };
        shifted = shift(shifted, *ai);
        Arg::Ui(shifted).swaptags(globals);
    }

    #[allow(dead_code)]
    pub(crate) fn togglegaps(_arg: &Arg, globals: &mut Globals) {
        globals.enable_gaps = !globals.enable_gaps;
        Monitor::arrange(None, globals);
    }

    #[allow(dead_code)]
    pub(crate) fn defaultgaps(&self, globals: &mut Globals) {
        globals.setgaps(
            crate::load_resource!("GAPP_OH", globals, Integer) as i32,
            crate::load_resource!("GAPP_OV", globals, Integer) as i32,
            crate::load_resource!("GAPP_IH", globals, Integer) as i32,
            crate::load_resource!("GAPP_IV", globals, Integer) as i32,
        );
    }

    #[allow(dead_code)]
    pub(crate) fn incrgaps(&self, globals: &mut Globals) {
        let Arg::I(i) = self else {
            unreachable!("invalid value given to incrgaps")
        };
        globals.setgaps(
            unsafe { globals.selmon.as_ref() }.gappoh + *i,
            unsafe { globals.selmon.as_ref() }.gappov + *i,
            unsafe { globals.selmon.as_ref() }.gappih + *i,
            unsafe { globals.selmon.as_ref() }.gappiv + *i,
        );
    }

    #[allow(dead_code)]
    pub(crate) fn incrigaps(&self, globals: &mut Globals) {
        let Arg::I(i) = self else {
            unreachable!("invalid value given to incrgaps")
        };
        globals.setgaps(
            unsafe { globals.selmon.as_ref() }.gappoh,
            unsafe { globals.selmon.as_ref() }.gappov,
            unsafe { globals.selmon.as_ref() }.gappih + *i,
            unsafe { globals.selmon.as_ref() }.gappiv + *i,
        );
    }

    #[allow(dead_code)]
    pub(crate) fn incrogaps(&self, globals: &mut Globals) {
        let Arg::I(i) = self else {
            unreachable!("invalid value given to incrgaps")
        };
        globals.setgaps(
            unsafe { globals.selmon.as_ref() }.gappoh + *i,
            unsafe { globals.selmon.as_ref() }.gappov + *i,
            unsafe { globals.selmon.as_ref() }.gappih,
            unsafe { globals.selmon.as_ref() }.gappiv,
        );
    }

    #[allow(dead_code)]
    pub(crate) fn incrovgaps(&self, globals: &mut Globals) {
        let Arg::I(i) = self else {
            unreachable!("invalid value given to incrgaps")
        };
        globals.setgaps(
            unsafe { globals.selmon.as_ref() }.gappoh,
            unsafe { globals.selmon.as_ref() }.gappov + *i,
            unsafe { globals.selmon.as_ref() }.gappih,
            unsafe { globals.selmon.as_ref() }.gappiv,
        );
    }

    #[allow(dead_code)]
    pub(crate) fn incrihgaps(&self, globals: &mut Globals) {
        let Arg::I(i) = self else {
            unreachable!("invalid value given to incrgaps")
        };
        globals.setgaps(
            unsafe { globals.selmon.as_ref() }.gappoh,
            unsafe { globals.selmon.as_ref() }.gappov,
            unsafe { globals.selmon.as_ref() }.gappih + *i,
            unsafe { globals.selmon.as_ref() }.gappiv,
        );
    }

    #[allow(dead_code)]
    pub(crate) fn incrivgaps(&self, globals: &mut Globals) {
        let Arg::I(i) = self else {
            unreachable!("invalid value given to incrgaps")
        };
        globals.setgaps(
            unsafe { globals.selmon.as_ref() }.gappoh,
            unsafe { globals.selmon.as_ref() }.gappov,
            unsafe { globals.selmon.as_ref() }.gappih,
            unsafe { globals.selmon.as_ref() }.gappiv + *i,
        );
    }
}
