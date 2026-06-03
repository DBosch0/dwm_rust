use std::{ffi::CString, mem::MaybeUninit, ptr::NonNull};

use crate::event::ClickState;
use crate::resource::load_resource;
use crate::{
    BROKEN, Globals, Monitor, NET_ACTIVE_WINDOW, NET_WM_FULLSCREEN, NET_WM_NAME, NET_WM_STATE,
    NET_WM_STICKY, NET_WM_WINDOW_TYPE, NET_WM_WINDOW_TYPE_DIALOG, SCHEME_STATE_NORM,
    SCHEME_STATE_SEL, SPTAGMASK, TAGMASK, WM_PROTOCOLS, WM_STATE, WM_TAKE_FOCUS,
};

use crate::external_functions::{
    ANY_BUTTON, ANY_MODIFIER, Atom, BUTTON_PRESS_MASK, BUTTON_RELEASE_MASK, CURRENT_TIME,
    CW_BORDER_WIDTH, CW_HEIGHT, CW_WIDTH, CWX, CWY, GRAB_MODE_ASYNC, GRAB_MODE_SYNC, LOCK_MASK,
    NO_EVENT_MASK, NORMAL_STATE, PROP_MODE_REPLACE, REVERT_TO_POINTER_ROOT, STRUCTURE_NOTIFY_MASK,
    WITHDRAWN_STATE, Window, XA_ATOM, XA_WINDOW, XA_WM_NAME, XChangeProperty, XClassHint,
    XClientMessageEvent, XClientMessageEventData, XConfigureEvent, XConfigureWindow,
    XDeleteProperty, XEvent, XFree, XGetClassHint, XGetWMHints, XGetWMNormalHints, XGetWMProtocols,
    XGetWindowProperty, XGrabButton, XGrabServer, XMapWindow, XMoveResizeWindow, XMoveWindow,
    XRaiseWindow, XSelectInput, XSendEvent, XSetErrorHandler, XSetInputFocus, XSetWMHints,
    XSetWindowBorder, XSizeHints, XSync, XUngrabButton, XUngrabServer, XUnmapWindow, XWMHints,
    XWindowChanges,
};

pub(crate) struct Client {
    pub(crate) name: [i8; 256],
    pub(crate) mina: f32,
    pub(crate) maxa: f32,
    pub(crate) cfact: f32,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) w: i32,
    pub(crate) h: i32,
    pub(crate) oldx: i32,
    pub(crate) oldy: i32,
    pub(crate) oldw: i32,
    pub(crate) oldh: i32,
    pub(crate) basew: i32,
    pub(crate) baseh: i32,
    pub(crate) incw: i32,
    pub(crate) inch: i32,
    pub(crate) maxw: i32,
    pub(crate) maxh: i32,
    pub(crate) minw: i32,
    pub(crate) minh: i32,
    pub(crate) hintsvalid: bool,
    pub(crate) bw: i32,
    pub(crate) oldbw: i32,
    pub(crate) tags: u32,
    pub(crate) isfixed: bool,
    pub(crate) isfloating: bool,
    pub(crate) isurgent: bool,
    pub(crate) neverfocus: bool,
    pub(crate) oldstate: bool,
    pub(crate) isfullscreen: bool,
    pub(crate) isterminal: bool,
    pub(crate) noswallow: bool,
    pub(crate) issticky: bool,
    pub(crate) pid: libc::pid_t,
    pub(crate) next: Option<NonNull<Client>>,
    pub(crate) snext: Option<NonNull<Client>>,
    pub(crate) swallowing: Option<NonNull<Client>>,
    pub(crate) mon: NonNull<Monitor>,
    pub(crate) win: Window,
}

impl Client {
    pub(crate) fn width(&self) -> i32 {
        self.w + 2 * self.bw
    }

    pub(crate) fn height(&self) -> i32 {
        self.h + 2 * self.bw
    }

    #[inline(always)]
    pub(crate) const fn is_visible(&self) -> bool {
        let m_ref = unsafe { self.mon.as_ref() };
        self.tags & m_ref.tagset[m_ref.seltags as usize] != 0 || self.issticky
    }

    pub(crate) fn nexttiled(mut c: Option<NonNull<Self>>) -> Option<NonNull<Self>> {
        while let Some(c_inner) = c
            && (unsafe { c_inner.as_ref() }.isfloating || !unsafe { c_inner.as_ref() }.is_visible())
        {
            c = unsafe { c_inner.as_ref() }.next;
        }
        c
    }

    pub(crate) fn termforwin(&self, globals: &Globals) -> Option<NonNull<Self>> {
        if self.pid == 0 || self.isterminal {
            return None;
        }

        let mut m = Some(globals.mons);
        while let Some(mi) = m {
            let mut c = unsafe { mi.as_ref().clients };
            while let Some(ci) = c {
                if unsafe { ci.as_ref().isterminal }
                    && unsafe { ci.as_ref().swallowing.is_none() }
                    && unsafe { ci.as_ref().pid } != 0
                    && crate::util::isdescprocess(unsafe { ci.as_ref().pid }, self.pid) != 0
                {
                    return c;
                }
                c = unsafe { ci.as_ref().next }
            }
            m = unsafe { mi.as_ref().next };
        }

        None
    }

    pub(crate) fn setsticky(&mut self, sticky: bool, globals: &mut Globals) {
        if sticky && !self.issticky {
            unsafe {
                XChangeProperty(
                    globals.dpy.as_ptr(),
                    self.win,
                    globals.netatom[NET_WM_STATE],
                    XA_ATOM,
                    32,
                    PROP_MODE_REPLACE,
                    &globals.netatom[NET_WM_STICKY] as *const u64 as *const u8,
                    1,
                );
            }
            self.issticky = true;
        } else if !sticky && self.issticky {
            unsafe {
                XChangeProperty(
                    globals.dpy.as_ptr(),
                    self.win,
                    globals.netatom[NET_WM_STATE],
                    XA_ATOM,
                    32,
                    PROP_MODE_REPLACE,
                    core::ptr::null(),
                    0,
                );
            }
            self.issticky = false;
            Monitor::arrange(Some(self.mon), globals);
        }
    }

    pub(crate) fn updatewindowtype(&mut self, globals: &mut Globals) {
        let state: Atom = self.getatomprop(globals.netatom[NET_WM_STATE], globals);
        let wtype: Atom = self.getatomprop(globals.netatom[NET_WM_WINDOW_TYPE], globals);

        if state == globals.netatom[NET_WM_FULLSCREEN] {
            self.setfullscreen(true, globals)
        }
        if state == globals.netatom[NET_WM_STICKY] {
            self.setsticky(true, globals);
        }
        if wtype == globals.netatom[NET_WM_WINDOW_TYPE_DIALOG] {
            self.isfloating = true;
        }
    }

    pub(crate) fn setfullscreen(&mut self, fullscreen: bool, globals: &mut Globals) {
        if fullscreen && !self.isfullscreen {
            unsafe {
                XChangeProperty(
                    globals.dpy.as_ptr(),
                    self.win,
                    globals.netatom[NET_WM_STATE],
                    XA_ATOM,
                    32,
                    PROP_MODE_REPLACE,
                    &globals.netatom[NET_WM_FULLSCREEN] as *const _ as *const u8,
                    1,
                )
            };
            self.isfullscreen = true;
            self.oldstate = self.isfloating;
            self.oldbw = self.bw;
            self.bw = 0;
            self.isfloating = true;
            let (mx, my, mw, mh) = unsafe {
                let m = self.mon.as_ref();
                (m.mx, m.my, m.mw, m.mh)
            };
            self.resizeclient(mx, my, mw, mh, globals);
            unsafe { XRaiseWindow(globals.dpy.as_ptr(), self.win) };
        } else if !fullscreen && self.isfullscreen {
            unsafe {
                XChangeProperty(
                    globals.dpy.as_ptr(),
                    self.win,
                    globals.netatom[NET_WM_STATE],
                    XA_ATOM,
                    32,
                    PROP_MODE_REPLACE,
                    core::ptr::null::<u8>(),
                    0,
                )
            };
            self.isfullscreen = false;
            self.isfloating = self.oldstate;
            self.bw = self.oldbw;
            self.x = self.oldx;
            self.y = self.oldy;
            self.w = self.oldw;
            self.h = self.oldh;
            // let (x, y, w, h, mon) = (self.x, self.y, self.w, self.h, self.mon);
            self.resizeclient(self.x, self.y, self.w, self.h, globals);
            Monitor::arrange(Some(self.mon), globals);
        }
    }

    pub(crate) fn resizeclient(&mut self, x: i32, y: i32, w: i32, h: i32, globals: &Globals) {
        let mut wc = XWindowChanges {
            x,
            y,
            width: w,
            height: h,
            border_width: 0,
            sibling: 0,
            stack_mode: 0,
        };
        self.oldx = self.x;
        self.x = wc.x;
        self.oldy = self.y;
        self.y = wc.y;
        self.oldw = self.w;
        self.w = wc.width;
        self.oldh = self.h;
        self.h = wc.height;
        wc.border_width = self.bw;
        unsafe {
            XConfigureWindow(
                globals.dpy.as_ptr(),
                self.win,
                CWX | CWY | CW_WIDTH | CW_HEIGHT | CW_BORDER_WIDTH,
                &mut wc,
            )
        };
        self.configure(globals);
        unsafe { XSync(globals.dpy.as_ptr(), 0) };
    }

    pub(crate) fn configure(&self, globals: &Globals) {
        const CONFIGURE_NOTIFY: i32 = 22;
        let mut ce = XConfigureEvent {
            r#type: CONFIGURE_NOTIFY,
            serial: 0,
            send_event: 0,
            display: globals.dpy.as_ptr(),
            event: self.win,
            window: self.win,
            x: self.x,
            y: self.y,
            width: self.w,
            height: self.h,
            border_width: self.bw,
            above: 0,
            override_redirect: 0,
        };
        unsafe {
            XSendEvent(
                globals.dpy.as_ptr(),
                self.win,
                0,
                STRUCTURE_NOTIFY_MASK,
                (&mut ce) as *mut _ as *mut XEvent,
            )
        };
    }

    pub(crate) fn applysizehints(
        &mut self,
        x: &mut i32,
        y: &mut i32,
        w: &mut i32,
        h: &mut i32,
        interact: bool,
        globals: &Globals,
    ) -> bool {
        // Read the monitor fields up front before any mutation of c.
        let (m_wx, m_wy, m_ww, m_wh, _m_sellt, m_lt_has_arrange) = unsafe {
            let m = self.mon.as_ref();
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
                *x = globals.sw - self.width();
            }
            if *y > globals.sh {
                *y = globals.sh - self.height();
            }
            if *x + *w + 2 * self.bw < 0 {
                *x = 0;
            }
            if *y + *h + 2 * self.bw < 0 {
                *y = 0
            }
        } else {
            if *x >= m_wx + m_ww {
                *x = m_wx + m_ww - self.width();
            }
            if *y >= m_wy + m_wh {
                *y = m_wy + m_wh - self.height();
            }
            if *x + *w + 2 * self.bw <= m_wx {
                *x = m_wx;
            }
            if *y + *h + 2 * self.bw <= m_wy {
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
        if load_resource!("RESIZE_HINTS", globals, Bool) || self.isfloating || m_lt_has_arrange {
            if !self.hintsvalid {
                self.updatesizehints(globals)
            }
            /* see last two sentences in ICCCM 4.1.2.3 */
            let baseismin = self.basew == self.minw && self.baseh == self.minh;
            if !baseismin {
                /* temporarily remove base dimensions */
                *w -= self.basew;
                *h -= self.baseh;
            }
            /* adjust for aspect limits */
            if self.mina > 0.0 && self.maxa > 0.0 {
                if self.maxa < *w as f32 / *h as f32 {
                    *w = (*h as f32 * self.maxa + 0.5) as i32;
                } else if self.mina < *h as f32 / *w as f32 {
                    *h = (*w as f32 * self.mina + 0.5) as i32;
                }
            }
            if baseismin {
                /* increment calculation requires this */
                *w -= self.basew;
                *h -= self.baseh;
            }
            /* adjust for increment value */
            if self.incw != 0 {
                *w -= *w % self.incw;
            }
            if self.inch != 0 {
                *h -= *h % self.inch;
            }
            /* restore base dimensions */
            *w = (*w + self.basew).max(self.minw);
            *h = (*h + self.baseh).max(self.minh);
            if self.maxw != 0 {
                *w = (*w).min(self.maxw);
            }
            if self.maxh != 0 {
                *h = (*h).min(self.maxh);
            }
        }

        *x != self.x || *y != self.y || *w != self.w || *h != self.h
    }

    pub(crate) fn updatesizehints(&mut self, globals: &Globals) {
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
                self.win,
                &mut size as *mut _ as *mut _,
                &mut msize,
            )
        } != 0;
        let mut size = unsafe { size.assume_init() };
        if !hint_result {
            size.flags = P_SIZE;
        }
        if size.flags & P_BASE_SIZE != 0 {
            self.basew = size.base_width;
            self.baseh = size.base_height;
        } else if size.flags & P_MIN_SIZE != 0 {
            self.basew = size.min_width;
            self.baseh = size.min_height;
        } else {
            self.basew = 0;
            self.baseh = 0;
        }
        if size.flags & P_RESIZE_INC != 0 {
            self.incw = size.width_inc;
            self.inch = size.height_inc;
        } else {
            self.incw = 0;
            self.inch = 0;
        }
        if size.flags & P_MAX_SIZE != 0 {
            self.maxw = size.max_width;
            self.maxh = size.max_height;
        } else {
            self.maxw = 0;
            self.maxh = 0;
        }
        if size.flags & P_MIN_SIZE != 0 {
            self.minw = size.min_width;
            self.minh = size.min_height;
        } else if size.flags & P_BASE_SIZE != 0 {
            self.minw = size.base_width;
            self.minh = size.base_height;
        } else {
            self.minw = 0;
            self.minh = 0;
        }
        if size.flags & P_ASPECT != 0 {
            self.mina = size.min_aspect.y as f32 / size.min_aspect.x as f32;
            self.maxa = size.max_aspect.x as f32 / size.max_aspect.y as f32;
        } else {
            self.maxa = 0.0;
            self.mina = 0.0;
        }
        self.isfixed =
            self.maxw != 0 && self.maxh != 0 && self.maxw == self.minw && self.maxh == self.minh;
        self.hintsvalid = true;
    }

    pub(crate) fn getatomprop(&self, prop: Atom, globals: &Globals) -> Atom {
        let mut format = 0i32;
        let mut nitems = 0u64;
        let mut dl = 0u64;
        let mut p: *mut u8 = core::ptr::null_mut();
        let mut da: Atom = 0;

        let mut atom = 0;

        if unsafe {
            XGetWindowProperty(
                globals.dpy.as_ptr(),
                self.win,
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

    pub(crate) fn resize(
        &mut self,
        mut x: i32,
        mut y: i32,
        mut w: i32,
        mut h: i32,
        interact: bool,
        globals: &Globals,
    ) {
        if self.applysizehints(&mut x, &mut y, &mut w, &mut h, interact, globals) {
            self.resizeclient(x, y, w, h, globals);
        }
    }

    pub(crate) fn updatewmhints(&mut self, globals: &Globals) {
        const INPUT_HINT: i64 = 1 << 0;
        const X_URGENCY_HINT: i64 = 1 << 8;

        let wmh: *mut XWMHints = unsafe { XGetWMHints(globals.dpy.as_ptr(), self.win) };
        if !wmh.is_null() {
            let is_sel = unsafe { globals.selmon.as_ref() }
                .sel
                .is_some_and(|sel| core::ptr::eq(self, unsafe { sel.as_ref() }));
            if is_sel && unsafe { &*wmh }.flags & X_URGENCY_HINT != 0 {
                unsafe { &mut *wmh }.flags &= !X_URGENCY_HINT;
                unsafe { XSetWMHints(globals.dpy.as_ptr(), self.win, wmh) };
            } else {
                self.isurgent = unsafe { &*wmh }.flags & X_URGENCY_HINT != 0;
            }
            self.neverfocus = if unsafe { &*wmh }.flags & INPUT_HINT != 0 {
                unsafe { &*wmh }.input == 0
            } else {
                false
            };
            unsafe { XFree(wmh.cast()) };
        }
    }

    pub(crate) fn grabbuttons(&self, focused: bool, globals: &mut Globals) {
        globals.updatenumlockmask();
        {
            let modifiers = [
                0,
                LOCK_MASK,
                globals.numlockmask,
                globals.numlockmask | LOCK_MASK,
            ];

            unsafe { XUngrabButton(globals.dpy.as_ptr(), ANY_BUTTON, ANY_MODIFIER, self.win) };
            if !focused {
                unsafe {
                    XGrabButton(
                        globals.dpy.as_ptr(),
                        ANY_BUTTON,
                        ANY_MODIFIER,
                        self.win,
                        0,
                        (BUTTON_PRESS_MASK | BUTTON_RELEASE_MASK) as u32,
                        GRAB_MODE_SYNC,
                        GRAB_MODE_SYNC,
                        0,
                        0,
                    )
                };
            }
            for i in 0..crate::config::BUTTONS.len() {
                if crate::config::BUTTONS[i].click == ClickState::ClientWin {
                    for modi in modifiers {
                        unsafe {
                            XGrabButton(
                                globals.dpy.as_ptr(),
                                crate::config::BUTTONS[i].button,
                                crate::config::BUTTONS[i].mask | modi,
                                self.win,
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

    pub(crate) fn updatetitle(&mut self, globals: &Globals) {
        if !globals.gettextprop(
            self.win,
            globals.netatom[NET_WM_NAME],
            self.name.as_mut_ptr(),
            self.name.len() as u32,
        ) {
            globals.gettextprop(
                self.win,
                XA_WM_NAME,
                self.name.as_mut_ptr(),
                self.name.len() as u32,
            );
        }
        if self.name[0] == b'\0' as i8 {
            unsafe { libc::strcpy(self.name.as_mut_ptr(), BROKEN.as_ptr()) };
        }
    }

    pub(crate) fn applyrules(&mut self, globals: &Globals) {
        let mut ch = XClassHint {
            res_name: core::ptr::null_mut(),
            res_class: core::ptr::null_mut(),
        };
        self.isfloating = false;
        self.tags = 0;
        unsafe { XGetClassHint(globals.dpy.as_ptr(), self.win, &mut ch) };
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

        for r in crate::config::RULES.iter() {
            let r_title = if !r.title.is_empty() {
                !unsafe {
                    libc::strstr(
                        self.name.as_ptr(),
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
                self.isterminal = r.isterminal;
                self.noswallow = r.noswallow;
                self.isfloating = r.isfloating;
                self.tags |= r.tags;

                if r.tags & SPTAGMASK != 0 && r.isfloating {
                    let mon = unsafe { self.mon.as_ref() };
                    self.x = mon.wx + (mon.ww / 2 - self.width() / 2);
                    self.y = mon.wy + (mon.wh / 2 - self.height() / 2);
                }

                let mut m = Some(globals.mons);
                while let Some(m_inner) = m
                    && unsafe { m_inner.as_ref().num } != r.monitor
                {
                    m = unsafe { m_inner.as_ref() }.next;
                }
                if let Some(m) = m {
                    self.mon = m;
                }
            }
        }

        if !ch.res_class.is_null() {
            unsafe { XFree(ch.res_class.cast_mut().cast()) };
        }
        if !ch.res_name.is_null() {
            unsafe { XFree(ch.res_name.cast_mut().cast()) };
        }

        self.tags = if self.tags & TAGMASK != 0 {
            self.tags & TAGMASK
        } else {
            let mon = unsafe { self.mon.as_ref() };
            mon.tagset[mon.seltags as usize] & !SPTAGMASK
        };
    }

    pub(crate) fn unmanage(c: NonNull<Self>, destroyed: bool, globals: &mut Globals) {
        let c_ref = unsafe { c.as_ref() };
        let m = c_ref.mon;
        let mut wc: XWindowChanges = unsafe { core::mem::zeroed() };

        if unsafe { c.as_ref().swallowing.is_some() } {
            Client::unswallow(c, globals);
            return;
        }

        let s: Option<NonNull<Client>> = Client::swallowingclient(c_ref.win, globals);
        if let Some(mut s) = s {
            let swallowing = unsafe {
                s.as_mut()
                    .swallowing
                    .take()
                    .expect("swallowingclient only returns s when s.swallowing.is_some()")
            };
            let _ = unsafe { Box::from_raw(swallowing.as_ptr()) };
            Monitor::arrange(Some(m), globals);
            Client::focus(None, globals);
            return;
        }

        Client::detach(c);
        Client::detachstack(c);
        if !destroyed {
            wc.border_width = c_ref.oldbw;
            unsafe {
                XGrabServer(globals.dpy.as_ptr());
                XSetErrorHandler(crate::xerrordummy);
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
            c_ref.setclientstate(WITHDRAWN_STATE as i64, globals);
            unsafe {
                XSync(globals.dpy.as_ptr(), 0);
                XSetErrorHandler(crate::xerror);
                XUngrabServer(globals.dpy.as_ptr());
            }
        }

        unsafe {
            let _ = Box::from_raw(c.as_ptr());
        }
        //NOTE: swallowing patch has a check here if to only run this is s is none
        //but if s is some we will have returned already above. So not possible for
        //s to not be none in this case.
        Client::focus(None, globals);
        globals.updateclientlist();
        Monitor::arrange(Some(m), globals);
    }

    pub(crate) fn setclientstate(&self, state: i64, globals: &Globals) {
        let data = [state, 0];

        unsafe {
            XChangeProperty(
                globals.dpy.as_ptr(),
                self.win,
                globals.wmatom[WM_STATE],
                globals.wmatom[WM_STATE],
                32,
                PROP_MODE_REPLACE,
                (&data) as *const _ as *const u8,
                2,
            );
        }
    }

    pub(crate) fn detach(mut c: NonNull<Self>) {
        let mut tc = &mut unsafe { c.as_mut().mon.as_mut() }.clients;
        while let Some(tc_inner) = tc.as_mut()
            && *tc_inner != c
        {
            tc = &mut unsafe { tc_inner.as_mut() }.next
        }
        *tc = unsafe { c.as_ref().next };
    }

    pub(crate) fn detachstack(mut c: NonNull<Self>) {
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
                && !unsafe { t_inner.as_ref() }.is_visible()
            {
                t = unsafe { t_inner.as_ref() }.snext;
            }
            unsafe { c.as_mut().mon.as_mut().sel = t };
        }
    }

    pub(crate) fn unfocus(c: Option<NonNull<Self>>, setfocus: bool, globals: &mut Globals) {
        let Some(c) = c else { return };
        unsafe { c.as_ref() }.grabbuttons(false, globals);
        unsafe {
            XSetWindowBorder(
                globals.dpy.as_ptr(),
                c.as_ref().win,
                globals.scheme[SCHEME_STATE_NORM][crate::drw::COL_BORDER].pixel,
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
                    globals.netatom[NET_ACTIVE_WINDOW],
                )
            };
        }
    }

    pub(crate) fn focus(mut c: Option<NonNull<Self>>, globals: &mut Globals) {
        if !c.is_some_and(|c| unsafe { c.as_ref() }.is_visible()) {
            c = unsafe { globals.selmon.as_ref() }.stack;
            while let Some(c_inner) = c
                && !unsafe { c_inner.as_ref() }.is_visible()
            {
                c = unsafe { c_inner.as_ref() }.snext;
            }
        }
        if let Some(sel) = unsafe { globals.selmon.as_ref() }.sel
            && let Some(c_inner) = c
            && sel != c_inner
        {
            Client::unfocus(Some(sel), false, globals);
        }
        if let Some(mut c_inner) = c {
            let c_ref = unsafe { c_inner.as_mut() };
            if c_ref.mon != globals.selmon {
                globals.selmon = c_ref.mon;
            }
            if c_ref.isurgent {
                c_ref.seturgent(false, globals)
            }
            Client::detachstack(c_inner);
            Client::attachstack(c_inner);
            c_ref.grabbuttons(true, globals);
            unsafe {
                XSetWindowBorder(
                    globals.dpy.as_ptr(),
                    c_ref.win,
                    globals.scheme[SCHEME_STATE_SEL][crate::drw::COL_BORDER].pixel,
                )
            };
            c_ref.setfocus(globals);
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
                    globals.netatom[NET_ACTIVE_WINDOW],
                )
            };
        }
        unsafe { globals.selmon.as_mut().sel = c };
        globals.drawbars();
    }

    pub(crate) fn seturgent(&mut self, urg: bool, globals: &Globals) {
        self.isurgent = urg;
        let wmh = unsafe { XGetWMHints(globals.dpy.as_ptr(), self.win) };
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
        unsafe { XSetWMHints(globals.dpy.as_ptr(), self.win, wmh) };
        unsafe { XFree(wmh.cast()) };
    }

    pub(crate) fn attach(mut c: NonNull<Self>) {
        unsafe { c.as_mut().next = c.as_ref().mon.as_ref().clients }
        unsafe { c.as_mut().mon.as_mut().clients = Some(c) };
    }

    pub(crate) fn attachstack(mut c: NonNull<Self>) {
        unsafe { c.as_mut().snext = c.as_ref().mon.as_ref().stack };
        unsafe { c.as_mut().mon.as_mut().stack = Some(c) };
    }

    pub(crate) fn swallow(mut p: NonNull<Self>, mut c: NonNull<Self>, globals: &mut Globals) {
        let c_ref = unsafe { c.as_mut() };
        let p_ref = unsafe { p.as_mut() };
        if c_ref.noswallow || c_ref.isterminal {
            return;
        }
        if c_ref.noswallow && !load_resource!("SWALLOW_FLOATING", globals, Bool) && c_ref.isfloating
        {
            return;
        }

        Client::detach(c);
        Client::detachstack(c);

        c_ref.setclientstate(WITHDRAWN_STATE as i64, globals);
        unsafe { XUnmapWindow(globals.dpy.as_ptr(), p_ref.win) };

        p_ref.swallowing = Some(c);
        c_ref.mon = p_ref.mon;

        std::mem::swap(&mut p_ref.win, &mut c_ref.win);
        p_ref.updatetitle(globals);
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
        Monitor::arrange(Some(p_ref.mon), globals);
        p_ref.configure(globals);
        globals.updateclientlist();
    }

    pub(crate) fn unswallow(mut c: NonNull<Self>, globals: &mut Globals) {
        let c_ref = unsafe { c.as_mut() };
        let Some(swallowed) = c_ref.swallowing.take() else {
            unreachable!("gave a client to unswallow that has not swallowed anything.")
        };
        c_ref.win = unsafe { swallowed.as_ref() }.win;
        //Free the swallowed object, having set c.swallowing to None by take above.
        let _ = unsafe { Box::from_raw(swallowed.as_ptr()) };

        /* unfullscreen the client */
        c_ref.setfullscreen(false, globals);
        c_ref.updatetitle(globals);
        Monitor::arrange(Some(c_ref.mon), globals);
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
        c_ref.setclientstate(NORMAL_STATE as i64, globals);
        Client::focus(None, globals);
        Monitor::arrange(Some(c_ref.mon), globals);
    }

    pub(crate) fn sendevent(&self, proto: Atom, globals: &Globals) -> bool {
        const CLIENT_MESSAGE: i32 = 33;

        let mut n: i32 = 0;
        let mut protocols: *mut Atom = core::ptr::null_mut();
        let mut exists = false;

        if unsafe { XGetWMProtocols(globals.dpy.as_ptr(), self.win, &mut protocols, &mut n) } != 0 {
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
                    window: self.win,
                    message_type: globals.wmatom[WM_PROTOCOLS],
                    format: 32,
                    data: XClientMessageEventData {
                        l: [proto as i64, CURRENT_TIME as i64, 0, 0, 0],
                    },
                },
            };
            unsafe { XSendEvent(globals.dpy.as_ptr(), self.win, 0, NO_EVENT_MASK, &mut ev) };
        }

        exists
    }

    pub(crate) fn sendmon(mut c: NonNull<Self>, m: NonNull<Monitor>, globals: &mut Globals) {
        if unsafe { c.as_ref() }.mon == m {
            return;
        }
        Client::unfocus(Some(c), true, globals);
        Client::detach(c);
        Client::detachstack(c);
        unsafe { c.as_mut().mon = m };
        unsafe { c.as_mut() }.tags =
            unsafe { m.as_ref() }.tagset[unsafe { m.as_ref() }.seltags as usize]; /* assign tags of target monitor */
        Client::attach(c);
        Client::attachstack(c);

        let c_ref = unsafe { c.as_mut() };
        if c_ref.isfullscreen {
            c_ref.resizeclient(
                unsafe { m.as_ref() }.mx,
                unsafe { m.as_ref() }.my,
                unsafe { m.as_ref() }.mw,
                unsafe { m.as_ref() }.mh,
                globals,
            );
        }
        Client::focus(None, globals);
        Monitor::arrange(None, globals);
    }

    pub(crate) fn pop(c: NonNull<Self>, globals: &mut Globals) {
        Client::detach(c);
        Client::attach(c);
        Client::focus(Some(c), globals);
        Monitor::arrange(Some(unsafe { c.as_ref() }.mon), globals);
    }

    pub(crate) fn setfocus(&self, globals: &Globals) {
        unsafe {
            if !self.neverfocus {
                XSetInputFocus(
                    globals.dpy.as_ptr(),
                    self.win,
                    REVERT_TO_POINTER_ROOT,
                    CURRENT_TIME,
                );
            }
            XChangeProperty(
                globals.dpy.as_ptr(),
                globals.root,
                globals.netatom[NET_ACTIVE_WINDOW],
                XA_WINDOW,
                32,
                PROP_MODE_REPLACE,
                (&self.win) as *const _ as *const u8,
                1,
            );
        }
        self.sendevent(globals.wmatom[WM_TAKE_FOCUS], globals);
    }

    pub(crate) fn showhide(c: Option<NonNull<Self>>, globals: &Globals) {
        let Some(mut c) = c else { return };
        let c_ref = unsafe { c.as_mut() };
        let vis = c_ref.is_visible();
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
                c_ref.resize(c_ref.x, c_ref.y, c_ref.w, c_ref.h, false, globals);
            }
            Client::showhide(c_ref.snext, globals);
        } else {
            Client::showhide(c_ref.snext, globals);
            unsafe { XMoveWindow(globals.dpy.as_ptr(), c_ref.win, c_ref.width() * -2, c_ref.y) };
        }
    }

    pub(crate) fn wintoclient(w: Window, globals: &Globals) -> Option<NonNull<Self>> {
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

    pub(crate) fn swallowingclient(w: Window, globals: &Globals) -> Option<NonNull<Self>> {
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
}
