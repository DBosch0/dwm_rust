use std::{ffi::CString, ptr::NonNull, rc::Rc};

use crate::{
    Globals, SCHEME_STATE_NORM, SCHEME_STATE_SEL, TAGMASK,
    client::Client,
    external_functions::{
        CW_SIBLING, CW_STACK_MODE, ENTER_WINDOW_MASK, Window, XCheckMaskEvent, XConfigureWindow,
        XDestroyWindow, XEvent, XRaiseWindow, XSync, XUnmapWindow, XWindowChanges,
    },
    monitor::layouts::Layout,
    resource::load_resource,
};

pub(crate) struct Monitor {
    pub(crate) ltsymbol: [i8; 16],
    pub(crate) mfact: f32,
    pub(crate) nmaster: i32,
    pub(crate) num: i32,
    pub(crate) by: i32, //bar geometry
    pub(crate) mx: i32, //screen size
    pub(crate) my: i32,
    pub(crate) mw: i32,
    pub(crate) mh: i32,
    pub(crate) wx: i32, //window area
    pub(crate) wy: i32,
    pub(crate) ww: i32,
    pub(crate) wh: i32,
    pub(crate) gappih: i32, /* horizontal gap between windows */
    pub(crate) gappiv: i32, /* vertical gap between windows */
    pub(crate) gappoh: i32, /* horizontal outer gaps */
    pub(crate) gappov: i32, /* vertical outer gaps */
    pub(crate) seltags: u32,
    pub(crate) sellt: u32,
    pub(crate) tagset: [u32; 2],
    pub(crate) showbar: bool,
    pub(crate) topbar: bool,
    pub(crate) clients: Option<NonNull<Client>>,
    pub(crate) sel: Option<NonNull<Client>>,
    pub(crate) stack: Option<NonNull<Client>>,
    pub(crate) next: Option<NonNull<Monitor>>,
    pub(crate) barwin: Window,
    pub(crate) lt: [&'static Layout; 2],
}

impl Monitor {
    pub(crate) fn createmon(globals: &Globals) -> NonNull<Monitor> {
        let mut ltsym: [i8; 16] = [0; 16];
        for (i, b) in crate::config::LAYOUTS[0]
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
                &crate::config::LAYOUTS[0],
                &crate::config::LAYOUTS[1 % crate::config::LAYOUTS.len()],
            ],
        });

        NonNull::new(Box::leak(m)).expect("valid NonNull as created by Box")
    }

    pub(crate) fn updatebarpos(&mut self, globals: &Globals) {
        self.wy = self.my;
        self.wh = self.mh;
        if self.showbar {
            self.wh -= globals.bh;
            self.by = if self.topbar {
                self.wy
            } else {
                self.wy + self.wh
            };
            self.wy = if self.topbar {
                self.wy + globals.bh
            } else {
                self.wy
            };
        } else {
            self.by = -globals.bh;
        }
    }

    pub(crate) fn recttomon(x: i32, y: i32, w: i32, h: i32, globals: &Globals) -> NonNull<Self> {
        let mut m: Option<NonNull<Monitor>>;
        let mut r = globals.selmon;
        let mut area = 0;

        m = Some(globals.mons);
        while let Some(m_inner) = m {
            let m_inner_ref = unsafe { m_inner.as_ref() };
            let a = i32::max(
                0,
                i32::min(x + w, m_inner_ref.wx + m_inner_ref.ww) - i32::max(x, m_inner_ref.wx),
            ) * i32::max(
                0,
                i32::min(y + h, m_inner_ref.wy + m_inner_ref.wh) - i32::max(y, m_inner_ref.wy),
            );

            if a > area {
                area = a;
                r = m_inner;
            }
            m = m_inner_ref.next;
        }
        r
    }

    pub(crate) fn wintomon(w: Window, globals: &mut Globals) -> NonNull<Self> {
        let mut x = 0;
        let mut y = 0;

        if w == globals.root && globals.getrootptr(&mut x, &mut y) {
            return Monitor::recttomon(x, y, 1, 1, globals);
        }
        let mut m = Some(globals.mons);
        while let Some(m_inner) = m {
            if w == unsafe { m_inner.as_ref() }.barwin {
                return m_inner;
            }
            m = unsafe { m_inner.as_ref() }.next;
        }

        let c = Client::wintoclient(w, globals);
        if let Some(c) = c {
            return unsafe { c.as_ref() }.mon;
        }
        globals.selmon
    }

    pub(crate) fn drawbar(&self, globals: &mut Globals) {
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

        if !self.showbar {
            return;
        }

        let is_selmon = core::ptr::eq(self, unsafe { globals.selmon.as_ref() });
        if is_selmon {
            globals
                .drw
                .setscheme(Rc::clone(&globals.scheme[SCHEME_STATE_NORM]));

            let mut text = globals.stext.as_mut_ptr();
            let mut s = globals.stext.as_mut_ptr();
            let mut x = 0;
            while unsafe { *s } != 0 {
                // for (text = s = stext; *s; s++) {
                if (unsafe { *s } as u8) < b' ' {
                    let ch = unsafe { *s };
                    unsafe { *s = b'\0' as i8 };
                    tw = globals.text_w(text) - globals.lrpad;
                    globals.drw.text(
                        self.ww - globals.statusw + x,
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
            tw = globals.text_w(text) - globals.lrpad + 2;
            globals.drw.text(
                self.ww - globals.statusw + x,
                0,
                tw as u32,
                globals.bh as u32,
                0,
                text,
                false,
            );
            tw = globals.statusw;
        }
        let mut c = self.clients;
        while let Some(c_inner) = c {
            let c_ref = unsafe { c_inner.as_ref() };
            occ |= if c_ref.tags == TAGMASK { 0 } else { c_ref.tags };
            if c_ref.isurgent {
                urg |= c_ref.tags;
            }
            c = c_ref.next
        }
        let mut x = 0;
        for i in 0..crate::config::TAGS.len() {
            // Do not draw vacant tags
            if !(occ & 1 << i != 0 || self.tagset[self.seltags as usize] & 1 << i != 0) {
                continue;
            }

            let tag = CString::new(crate::config::TAGS[i]).expect("valid c string");
            let w = globals.drw.fontset_getwidth(tag.as_ptr()) + globals.lrpad as u32;
            globals.drw.setscheme(Rc::clone(
                &globals.scheme[if (self.tagset[self.seltags as usize] & 1 << i) != 0 {
                    SCHEME_STATE_SEL
                } else {
                    SCHEME_STATE_NORM
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

        let w = globals.drw.fontset_getwidth(&self.ltsymbol as *const i8) + globals.lrpad as u32;
        globals
            .drw
            .setscheme(Rc::clone(&globals.scheme[SCHEME_STATE_NORM]));
        let x = globals.drw.text(
            x,
            0,
            w,
            globals.bh as u32,
            globals.lrpad as u32 / 2,
            &self.ltsymbol as *const i8,
            false,
        );

        let w = self.ww - tw - x;
        if w > globals.bh {
            if let Some(m_sel) = self.sel {
                let m_sel_ref = unsafe { m_sel.as_ref() };
                globals.drw.setscheme(Rc::clone(
                    &globals.scheme[if is_selmon {
                        SCHEME_STATE_SEL
                    } else {
                        SCHEME_STATE_NORM
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
                    .setscheme(Rc::clone(&globals.scheme[SCHEME_STATE_NORM]));
                globals
                    .drw
                    .rect(x, 0, w as u32, globals.bh as u32, true, true);
            }
        }
        globals
            .drw
            .map(self.barwin, 0, 0, self.ww as u32, globals.bh as u32)
    }

    // dirtomon always returns a valid monitor. Callers must guard with
    // `if mons.next.is_none() { return; }` before calling to ensure ≥2 monitors exist.
    // In all three branches we either wrap around to `mons` (non-null by invariant) or
    // walk a linked list that is guaranteed to contain `selmon`.
    pub(crate) fn dirtomon(dir: i32, globals: &Globals) -> NonNull<Monitor> {
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

    pub(crate) fn restack(&self, globals: &mut Globals) {
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

        self.drawbar(globals);
        if self.sel.is_none() {
            return;
        }
        if let Some(sel) = self.sel
            && (unsafe { sel.as_ref() }.isfloating
                || self.lt[self.sellt as usize].arrange.is_none())
        {
            unsafe { XRaiseWindow(globals.dpy.as_ptr(), sel.as_ref().win) };
        }
        if self.lt[self.sellt as usize].arrange.is_some() {
            wc.stack_mode = BELOW;
            wc.sibling = self.barwin;
            let mut c = self.stack;
            while let Some(c_inner) = c {
                if !unsafe { c_inner.as_ref() }.isfloating
                    && unsafe { c_inner.as_ref() }.is_visible()
                {
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

    pub(crate) fn arrange(mut m: Option<NonNull<Self>>, globals: &mut Globals) {
        if let Some(m) = m {
            Client::showhide(unsafe { m.as_ref().stack }, globals);
        } else {
            m = Some(globals.mons);
            while let Some(m_inner) = m {
                Client::showhide(unsafe { m_inner.as_ref() }.stack, globals);
                m = unsafe { m_inner.as_ref() }.next;
            }
        }

        if let Some(mut m) = m {
            unsafe { m.as_mut() }.arrangemon(globals);
            unsafe { m.as_ref() }.restack(globals);
        } else {
            m = Some(globals.mons);
            while let Some(mut m_inner) = m {
                unsafe { m_inner.as_mut() }.arrangemon(globals);
                m = unsafe { m_inner.as_ref() }.next;
            }
        }
    }

    pub(crate) fn arrangemon(&mut self, globals: &mut Globals) {
        let symbol = CString::new(self.lt[self.sellt as usize].symbol).expect("valid CString");
        unsafe {
            libc::strncpy(
                self.ltsymbol.as_mut_ptr(),
                symbol.as_ptr(),
                self.ltsymbol.len(),
            )
        };
        if let Some(f) = self.lt[self.sellt as usize].arrange {
            f(self, globals)
        }
    }

    pub(crate) fn cleanupmon(mon: NonNull<Self>, globals: &mut Globals) -> bool {
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

    pub(crate) fn getgaps(&self, globals: &mut Globals) -> (i32, i32, i32, i32, u32) {
        let ie = globals.enable_gaps as i32;
        let mut oe = ie;
        let mut n = 0;
        let mut c = Client::nexttiled(self.clients);
        while let Some(c_inner) = c {
            c = Client::nexttiled(unsafe { c_inner.as_ref() }.next);
            n += 1;
        }
        if crate::load_resource!("SMART_GAPS", globals, Bool) && n == 1 {
            oe = 0;
        }
        (
            self.gappoh * oe,
            self.gappov * oe,
            self.gappih * ie,
            self.gappiv * ie,
            n,
        )
    }

    pub(crate) fn getfacts(&self, msize: i32, ssize: i32) -> (f32, f32, i32, i32) {
        let mut mfacts = 0.0;
        let mut sfacts = 0.0;
        let mut mtotal = 0;
        let mut stotal = 0;

        let mut n = 0;
        let mut c = Client::nexttiled(self.clients);
        while let Some(c_inner) = c {
            if n < self.nmaster {
                mfacts += unsafe { c_inner.as_ref() }.cfact;
            } else {
                sfacts += unsafe { c_inner.as_ref() }.cfact;
            }
            c = Client::nexttiled(unsafe { c_inner.as_ref() }.next);
            n += 1;
        }
        n = 0;
        c = Client::nexttiled(self.clients);
        while let Some(c_inner) = c {
            if n < self.nmaster {
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
}

pub(crate) mod layouts {
    use std::ffi::CString;

    use crate::{Globals, client::Client, monitor::Monitor};

    pub(crate) type LayoutFunction = fn(&mut super::Monitor, &mut super::Globals);

    pub(crate) struct Layout {
        pub(crate) symbol: &'static str,
        pub(crate) arrange: Option<LayoutFunction>,
    }

    pub(crate) fn bstack(m: &mut Monitor, globals: &mut Globals) {
        let (oh, ov, ih, iv, n) = m.getgaps(globals);
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

        let (mfacts, sfacts, mrest, srest) = m.getfacts(mw, sw);

        let mut i = 0;
        let mut c = Client::nexttiled(m.clients);
        while let Some(mut c_inner) = c {
            if i < m.nmaster {
                Client::resize(
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
                Client::resize(
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
            c = Client::nexttiled(unsafe { c_inner.as_ref() }.next);
            i += 1;
        }
    }

    pub(crate) fn bstackhoriz(m: &mut Monitor, globals: &mut Globals) {
        let (oh, ov, ih, iv, n) = m.getgaps(globals);
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

        let (mfacts, sfacts, mrest, srest) = m.getfacts(mw, sh);

        let mut i = 0;
        let mut c = Client::nexttiled(m.clients);
        while let Some(mut ci) = c {
            if i < m.nmaster {
                Client::resize(
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
                Client::resize(
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
            c = Client::nexttiled(unsafe { ci.as_ref() }.next);
            i += 1;
        }
    }

    pub(crate) fn centeredmaster(m: &mut Monitor, globals: &mut Globals) {
        let (oh, ov, ih, iv, n) = m.getgaps(globals);
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
        let mut c = Client::nexttiled(m.clients);
        while let Some(ci) = c {
            if m.nmaster == 0 || n < m.nmaster {
                mfacts += unsafe { ci.as_ref() }.cfact;
            } else if (n - m.nmaster) % 2 == 0 {
                lfacts += unsafe { ci.as_ref() }.cfact; // total factor of left hand stack area
            } else {
                rfacts += unsafe { ci.as_ref() }.cfact; // total factor of right hand stack area
            }
            c = Client::nexttiled(unsafe { ci.as_ref() }.next);
        }

        n = 0;
        let mut mtotal = 0;
        let mut ltotal = 0;
        let mut rtotal = 0;
        c = Client::nexttiled(m.clients);
        while let Some(ci) = c {
            if m.nmaster == 0 || n < m.nmaster {
                mtotal += (mh as f32 * (unsafe { ci.as_ref() }.cfact / mfacts)) as i32;
            } else if (n - m.nmaster) % 2 == 0 {
                ltotal += (lh as f32 * (unsafe { ci.as_ref() }.cfact / lfacts)) as i32;
            } else {
                rtotal += (rh as f32 * (unsafe { ci.as_ref() }.cfact / rfacts)) as i32;
            }
            c = Client::nexttiled(unsafe { ci.as_ref() }.next);
            n += 1
        }
        let mrest = mh - mtotal;
        let lrest = lh - ltotal;
        let rrest = rh - rtotal;

        let mut i = 0;
        let mut c = Client::nexttiled(m.clients);
        while let Some(mut ci) = c {
            if m.nmaster == 0 || i < m.nmaster {
                /* nmaster clients are stacked vertically, in the center of the screen */
                Client::resize(
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
                    Client::resize(
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
                    Client::resize(
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
            c = Client::nexttiled(unsafe { ci.as_ref() }.next);
            i += 1;
        }
    }

    pub(crate) fn centeredfloatingmaster(m: &mut Monitor, globals: &mut Globals) {
        let (oh, ov, _ih, iv, n) = m.getgaps(globals);
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
                mw = (m.ww as f32 * m.mfact
                    - iv as f32 * mivf * ((n as i32).min(m.nmaster) - 1) as f32)
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

        let (mfacts, sfacts, mrest, srest) = m.getfacts(mw, sw);
        let mut i = 0;
        let mut c = Client::nexttiled(m.clients);
        while let Some(mut ci) = c {
            let cr = unsafe { ci.as_mut() };
            if i < m.nmaster {
                /* nmaster clients are stacked horizontally, in the center of the screen */
                Client::resize(
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
                Client::resize(
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
        let (oh, ov, ih, iv, n) = m.getgaps(globals);
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

        let (mfacts, _sfacts, mrest, _srest) = m.getfacts(mh, sh);

        if n as i32 - m.nmaster > 0 {
            /* override layout symbol */
            let cstr = CString::new(format!("D {}", n as i32 - m.nmaster)).expect("valid CStr");
            unsafe { libc::snprintf(m.ltsymbol.as_mut_ptr(), m.ltsymbol.len(), cstr.as_ptr()) };
        }

        let mut i = 0;
        let mut c = Client::nexttiled(m.clients);
        while let Some(mut ci) = c {
            let cr = unsafe { ci.as_mut() };
            if i < m.nmaster {
                Client::resize(
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
                Client::resize(
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
            c = Client::nexttiled(cr.next)
        }
    }

    pub(crate) fn fibonacci(m: &mut Monitor, s: bool, globals: &mut Globals) {
        let (oh, ov, ih, iv, n) = m.getgaps(globals);
        if n == 0 {
            return;
        }
        let mut nx = m.wx + ov;
        let mut ny = m.wy + oh;
        let mut nw = m.ww - 2 * ov;
        let mut nh = m.wh - 2 * oh;

        let mut i = 0;
        let mut c = Client::nexttiled(m.clients);
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

            Client::resize(
                cr,
                nx,
                ny,
                nw - (2 * cr.bw),
                nh - (2 * cr.bw),
                false,
                globals,
            );
            c = Client::nexttiled(cr.next);
        }
    }

    pub(crate) fn dwindle(m: &mut Monitor, globals: &mut Globals) {
        fibonacci(m, true, globals);
    }

    pub(crate) fn spiral(m: &mut Monitor, globals: &mut Globals) {
        fibonacci(m, false, globals);
    }

    pub(crate) fn gaplessgrid(m: &mut Monitor, globals: &mut Globals) {
        let (oh, ov, ih, iv, n) = m.getgaps(globals);
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
        let mut c = Client::nexttiled(m.clients);
        while let Some(mut ci) = c {
            let cr = unsafe { ci.as_mut() };
            if i / rows + 1 > cols - (n as i32) % cols {
                rows = (n as i32) / cols + 1;
                ch = (m.wh - 2 * oh - ih * (rows - 1)) / rows;
                rrest = (m.wh - 2 * oh - ih * (rows - 1)) - ch * rows;
            }
            Client::resize(
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
            c = Client::nexttiled(cr.next);
        }
    }

    pub(crate) fn grid(m: &mut Monitor, globals: &mut Globals) {
        let (oh, ov, ih, iv, n) = m.getgaps(globals);

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
        let mut c = Client::nexttiled(m.clients);
        while let Some(mut ci) = c {
            let c_ref = unsafe { ci.as_mut() };
            let cc = i / rows;
            let cr = i % rows;
            let cx = m.wx + ov + cc * (cw + iv) + cc.min(cwrest);
            let cy = m.wy + oh + cr * (ch + ih) + cr.min(chrest);
            Client::resize(
                c_ref,
                cx,
                cy,
                cw + (if cc < cwrest { 1 } else { 0 }) - 2 * c_ref.bw,
                ch + (if cr < chrest { 1 } else { 0 }) - 2 * c_ref.bw,
                false,
                globals,
            );

            c = Client::nexttiled(c_ref.next);
            i += 1;
        }
    }

    pub(crate) fn horizgrid(m: &mut Monitor, globals: &mut Globals) {
        /* Count windows */
        let (oh, ov, ih, iv, n) = m.getgaps(globals);
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
        let mut c = Client::nexttiled(m.clients);
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
            c = Client::nexttiled(cr.next);
        }

        i = 0;
        c = Client::nexttiled(m.clients);
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
            c = Client::nexttiled(cr.next);
        }
        let mrest = mh - mtotal;
        let srest = sw - stotal;
        i = 0;
        c = Client::nexttiled(m.clients);
        while let Some(mut ci) = c {
            let cr = unsafe { ci.as_mut() };
            if i < ntop {
                Client::resize(
                    cr,
                    mx,
                    my,
                    (mw as f32 * (cr.cfact / mfacts)) as i32
                        + (if (i as i32) < mrest { 1 } else { 0 })
                        - (2 * cr.bw),
                    mh - (2 * cr.bw),
                    false,
                    globals,
                );
                mx += cr.width() + iv;
            } else {
                Client::resize(
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
            c = Client::nexttiled(cr.next);
        }
    }

    pub(crate) fn nrowgrid(m: &mut Monitor, globals: &mut Globals) {
        let mut ri = 0;
        let mut ci = 0;
        let mut uw = 0;
        let mut rows = m.nmaster + 1;

        /* count clients */
        let (oh, ov, ih, iv, n) = m.getgaps(globals);

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

        let mut c = Client::nexttiled(m.clients);
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

            Client::resize(
                c_ref,
                cx,
                cy,
                cw - (2 * c_ref.bw),
                ch - (2 * c_ref.bw),
                false,
                globals,
            );
            c = Client::nexttiled(c_ref.next);
            ci += 1;
        }
    }

    pub(crate) fn tile(m: &mut Monitor, globals: &mut Globals) {
        let (oh, ov, ih, iv, n) = m.getgaps(globals);
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

        let (mfacts, sfacts, mrest, srest) = m.getfacts(mh, sh);

        let mut i = 0;
        let mut c = Client::nexttiled(m.clients);
        while let Some(mut c_inner) = c {
            if i < m.nmaster {
                Client::resize(
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
                Client::resize(
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
            c = Client::nexttiled(unsafe { c_inner.as_ref() }.next);
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
            Client::resize(
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
}
