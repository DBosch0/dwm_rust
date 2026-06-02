use std::ffi::CString;

use crate::{
    ClickState, Globals, NET_ACTIVE_WINDOW, NET_WM_FULLSCREEN, NET_WM_NAME, NET_WM_STATE,
    NET_WM_STICKY, NET_WM_WINDOW_TYPE, SPTAGMASK, TAGMASK,
    argument::Arg,
    cleanmask,
    client::Client,
    external_functions::{
        BUTTON_PRESS, CLIENT_MESSAGE, CONFIGURE_NOTIFY, CONFIGURE_REQUEST, CURRENT_TIME,
        CW_BORDER_WIDTH, CW_HEIGHT, CW_WIDTH, CWX, CWY, DESTROY_NOTIFY, ENTER_NOTIFY, EXPOSE,
        FOCUS_IN, KEY_PRESS, KeyCode, MAP_REQUEST, MAPPING_NOTIFY, MOTION_NOTIFY, PROPERTY_NOTIFY,
        UNMAP_NOTIFY, WITHDRAWN_STATE, Window, XA_WM_HINTS, XA_WM_NAME, XA_WM_NORMAL_HINTS,
        XA_WM_TRANSIENT_FOR, XAllowEvents, XButtonPressedEvent, XClientMessageEvent,
        XConfigureEvent, XConfigureRequestEvent, XConfigureWindow, XCrossingEvent,
        XDestroyWindowEvent, XEvent, XExposeEvent, XFocusChangeEvent, XGetTransientForHint,
        XGetWindowAttributes, XKeyEvent, XKeycodeToKeysym, XMapRequestEvent, XMappingEvent,
        XMotionEvent, XMoveResizeWindow, XPropertyEvent, XRefreshKeyboardMapping, XSync,
        XUnmapEvent, XWindowAttributes, XWindowChanges,
    },
    monitor::Monitor,
    text_w, updatebars, updategeom,
};

pub(crate) type EventHandlerFunction = fn(&mut XEvent, &mut Globals);

pub(crate) const fn event_handler(event_type: i32) -> Option<EventHandlerFunction> {
    match event_type {
        KEY_PRESS => Some(XEvent::keypress),
        BUTTON_PRESS => Some(XEvent::buttonpress),
        MOTION_NOTIFY => Some(XEvent::motionnotify),
        ENTER_NOTIFY => Some(XEvent::enternotify),
        FOCUS_IN => Some(XEvent::focusin),
        EXPOSE => Some(XEvent::expose),
        DESTROY_NOTIFY => Some(XEvent::destroynotify),
        UNMAP_NOTIFY => Some(XEvent::unmapnotify),
        MAP_REQUEST => Some(XEvent::maprequest),
        CONFIGURE_NOTIFY => Some(XEvent::configurenotify),
        CONFIGURE_REQUEST => Some(XEvent::configurerequest),
        PROPERTY_NOTIFY => Some(XEvent::propertynotify),
        CLIENT_MESSAGE => Some(XEvent::clientmessage),
        MAPPING_NOTIFY => Some(XEvent::mappingnotify),
        _ => None,
    }
}

impl XEvent {
    pub(crate) fn buttonpress(&mut self, globals: &mut Globals) {
        const REPLAY_POINTER: i32 = 2;

        let mut click = ClickState::RootWin;
        let ev: &mut XButtonPressedEvent = unsafe { &mut self.xbutton };
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
                    let ctag = CString::new(crate::config::TAGS[i]).expect("valid CStr");
                    x += text_w(ctag.as_ptr(), globals);
                    if ev.x < x {
                        break; // clicked on tag i
                    }
                }
                i += 1;
                if i >= crate::config::TAGS.len() {
                    break; // clicked past all tags — i == TAGS.len()
                }
            }

            if i < crate::config::TAGS.len() {
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

        for i in 0..crate::config::BUTTONS.len() {
            if click == crate::config::BUTTONS[i].click
                && let Some(f) = crate::config::BUTTONS[i].func
                && crate::config::BUTTONS[i].button == ev.button
                && cleanmask(crate::config::BUTTONS[i].mask, globals)
                    == cleanmask(ev.state, globals)
            {
                f(
                    if click == ClickState::TagBar
                        && let Arg::I(ai) = crate::config::BUTTONS[i].arg
                        && ai == 0
                    {
                        &arg
                    } else {
                        &crate::config::BUTTONS[i].arg
                    },
                    globals,
                )
            }
        }
    }

    pub(crate) fn clientmessage(&mut self, globals: &mut Globals) {
        let cme: &mut XClientMessageEvent = unsafe { &mut self.xclient };
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
                    unsafe { cme.data.l[0] } == 1
                        || (unsafe { cme.data.l[0] } == 2 && !cr.issticky),
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

    pub(crate) fn configurerequest(&mut self, globals: &mut Globals) {
        let ev: &mut XConfigureRequestEvent = unsafe { &mut self.xconfigurerequest };

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

    pub(crate) fn configurenotify(&mut self, globals: &mut Globals) {
        let ev: &mut XConfigureEvent = unsafe { &mut self.xconfigure };

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
                            c_ref.resizeclient(
                                m_inner.mx, m_inner.my, m_inner.mw, m_inner.mh, globals,
                            );
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

    pub(crate) fn destroynotify(&mut self, globals: &mut Globals) {
        let ev: &mut XDestroyWindowEvent = unsafe { &mut self.xdestroywindow };
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

    pub(crate) fn enternotify(&mut self, globals: &mut Globals) {
        const NOTIFY_NORMAL: i32 = 0;
        const NOTIFY_INTERIOR: i32 = 2;

        let ev: &mut XCrossingEvent = unsafe { &mut self.xcrossing };

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

    pub(crate) fn expose(&mut self, globals: &mut Globals) {
        let ev: &mut XExposeEvent = unsafe { &mut self.xexpose };
        if ev.count == 0 {
            let m = Monitor::wintomon(ev.window, globals);
            unsafe { m.as_ref() }.drawbar(globals);
        }
    }

    pub(crate) fn focusin(&mut self, globals: &mut Globals) {
        let ev: &mut XFocusChangeEvent = unsafe { &mut self.xfocus };
        if let Some(sel) = unsafe { globals.selmon.as_ref() }.sel
            && ev.window != unsafe { sel.as_ref() }.win
        {
            unsafe { sel.as_ref() }.setfocus(globals);
        }
    }

    pub(crate) fn keypress(&mut self, globals: &mut Globals) {
        let ev: &mut XKeyEvent = unsafe { &mut self.xkey };
        let keysym = unsafe { XKeycodeToKeysym(globals.dpy.as_ptr(), ev.keycode as KeyCode, 0) };
        for key in crate::config::KEYS.iter() {
            if keysym == key.keysym
                && cleanmask(key.r#mod, globals) == cleanmask(ev.state, globals)
                && let Some(f) = key.func
            {
                f(&key.arg, globals);
            }
        }
    }

    pub(crate) fn mappingnotify(&mut self, globals: &mut Globals) {
        const MAPPING_KEYBOARD: i32 = 1;

        let ev: &mut XMappingEvent = unsafe { &mut self.xmapping };
        unsafe { XRefreshKeyboardMapping(ev) };
        if ev.request == MAPPING_KEYBOARD {
            crate::grabkeys(globals);
        }
    }

    pub(crate) fn maprequest(&mut self, globals: &mut Globals) {
        let mut wa: XWindowAttributes = unsafe { core::mem::zeroed() };
        let ev: &mut XMapRequestEvent = unsafe { &mut self.xmaprequest };

        if unsafe { XGetWindowAttributes(globals.dpy.as_ptr(), ev.window, &mut wa) } == 0
            || wa.override_redirect != 0
        {
            return;
        }
        if Client::wintoclient(ev.window, globals).is_none() {
            crate::manage(ev.window, &wa, globals);
        }
    }

    pub(crate) fn motionnotify(&mut self, globals: &mut Globals) {
        let ev: &mut XMotionEvent = unsafe { &mut self.xmotion };

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

    pub(crate) fn propertynotify(&mut self, globals: &mut Globals) {
        const PROPERTY_DELETE: i32 = 1;

        let ev: &mut XPropertyEvent = unsafe { &mut self.xproperty };
        let mut trans: Window = 0;

        if ev.window == globals.root && ev.atom == XA_WM_NAME {
            crate::updatestatus(globals);
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
                    crate::drawbars(globals);
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

    pub(crate) fn unmapnotify(&mut self, globals: &mut Globals) {
        let ev: &mut XUnmapEvent = unsafe { &mut self.xunmap };

        if let Some(c) = Client::wintoclient(ev.window, globals) {
            if ev.send_event != 0 {
                unsafe { c.as_ref() }.setclientstate(WITHDRAWN_STATE as i64, globals);
            } else {
                Client::unmanage(c, false, globals);
            }
        }
    }
}
