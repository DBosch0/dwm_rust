use std::{
    ffi::{CStr, CString},
    io::{Write, stderr},
    mem::MaybeUninit,
    num::Wrapping,
    ptr::NonNull,
    rc::Rc,
};

use crate::{
    die,
    external_functions::*,
};

const UTF_INVALID: i64 = 0xFFFD;
fn utf8decode(s_in: *const i8) -> (i32, i64, bool) {
    #[rustfmt::skip]
    const LENS: [u8; 32] = [
        /* 0XXXX */ 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 
        /* 10XXX */ 0, 0, 0, 0, 0, 0, 0, 0, /* invalid */
        /* 110XX */ 2, 2, 2, 2, 
        /* 1110X */ 3, 3, 
        /* 11110 */ 4,
        /* 11111 */ 0, /* invalid */
    ];
    const LEADING_MASK: [u8; 4] = [0x7F, 0x1F, 0x0F, 0x07];
    const OVERLONG: [u32; 4] = [0x0, 0x80, 0x0800, 0x10000];

    let s: *const u8 = s_in.cast();
    let len: i32 = LENS[(unsafe { *s } >> 3) as usize] as i32;

    let u = UTF_INVALID;
    let err = true;

    if len == 0 {
        return (1, u, err);
    }

    let mut cp: i64 = (unsafe { *s } & LEADING_MASK[(len - 1) as usize]) as i64;
    for i in 1..len {
        if unsafe { *s.add(i as usize) } == b'\0' || (unsafe { *s.add(i as usize) } & 0xC0) != 0x80 {
            return (i, u, err);
        }
        cp = (cp << 6) | (unsafe { *s.add(i as usize) } & 0x3F) as i64;
    }

    if cp > 0x10FFFF || cp >> 11 == 0x1B || cp < OVERLONG[(len - 1) as usize] as i64 {
        return (len, u, err);
    }

    (len, cp, false)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Cur {
    pub(crate) cursor: Cursor,
}

#[derive(Debug)]
pub(crate) struct Fnt {
    dpy: NonNull<Display>,
    pub(crate) h: u32,
    pub(crate) xfont: NonNull<XftFont>,
    pub(crate) pattern: Option<NonNull<FcPattern>>,
    pub(crate) next: Option<NonNull<Fnt>>,
}

pub(crate) const COL_FG: usize = 0;
pub(crate) const COL_BG: usize = 1;
pub(crate) const COL_BORDER: usize = 2;

pub(crate) type Clr = XftColor;

#[derive(Debug)]
pub(crate) struct Drw {
    pub(crate) w: u32,
    pub(crate) h: u32,
    pub(crate) dpy: NonNull<Display>,
    pub(crate) screen: i32,
    pub(crate) root: Window,
    pub(crate) drawable: Drawable,
    pub(crate) gc: GC,
    pub(crate) scheme: Option<Rc<[Clr]>>,
    pub(crate) fonts: Option<NonNull<Fnt>>,
}

impl Drw {
    pub(crate) fn new(
        dpy: NonNull<Display>,
        screen: i32,
        root: Window,
        w: u32,
        h: u32,
    ) -> Box<Self> {
        let drw = Box::new(Self {
            w,
            h,
            dpy,
            screen,
            root,
            gc: unsafe { XCreateGC(dpy.as_ptr(), root, 0, core::ptr::null()) },
            drawable: unsafe {
                XCreatePixmap(
                    dpy.as_ptr(),
                    root,
                    w,
                    h,
                    default_depth(dpy.as_ptr(), screen) as u32,
                )
            },
            scheme: None,
            fonts: None,
        });
        unsafe { XSetLineAttributes(dpy.as_ptr(), drw.gc, 1, LINE_SOLID, CAP_BUTT, JOIN_MITER) };

        drw
    }

    pub(crate) fn resize(&mut self, w: u32, h: u32) {
        self.w = w;
        self.h = h;

        if self.drawable != 0 {
            unsafe {
                XFreePixmap(self.dpy.as_ptr(), self.drawable);
            }
        }
        self.drawable = unsafe {
            XCreatePixmap(
                self.dpy.as_ptr(),
                self.root,
                w,
                h,
                default_depth(self.dpy.as_ptr(), self.screen) as u32,
            )
        };
    }

    fn xfont_create(
        &mut self,
        fontname: &str,
        fontpattern: Option<NonNull<FcPattern>>,
    ) -> Option<NonNull<Fnt>> {
        let xfont: NonNull<XftFont>;
        let mut pattern: Option<NonNull<FcPattern>> = None;

        let fontname = CString::new(fontname).expect("name is a valid CString");

        if !fontname.is_empty() {
            /* Using the pattern found at font->xfont->pattern does not yield the
             * same substitution results as using the pattern returned by
             * FcNameParse; using the latter results in the desired fallback
             * behaviour whereas the former just results in missing-character
             * rectangles being drawn, at least with some fonts. */

            xfont = if let Some(xfont) = NonNull::new(unsafe {
                XftFontOpenName(self.dpy.as_ptr(), self.screen, fontname.as_ptr())
            }) {
                xfont
            } else {
                let _ = writeln!(
                    stderr(),
                    "error, cannot load font from name: '{}'",
                    fontname.to_string_lossy()
                );
                return None;
            };

            pattern = NonNull::new(unsafe { FcNameParse(fontname.as_ptr().cast::<FcChar8>()) });
            if pattern.is_none() {
                let _ = writeln!(
                    stderr(),
                    "error, cannot parse font name to pattern: '{}'",
                    fontname.to_string_lossy()
                );
                unsafe { XftFontClose(self.dpy.as_ptr(), xfont.as_ptr()) };
                return None;
            }
        } else if let Some(fontpattern) = fontpattern {
            xfont = if let Some(xfont) =
                NonNull::new(unsafe { XftFontOpenPattern(self.dpy.as_ptr(), fontpattern.as_ptr()) })
            {
                xfont
            } else {
                let _ = writeln!(stderr(), "error, cannot load font from pattern");
                return None;
            };
            //TODO: checkif this is correct.
            // pattern = Some(fontpattern);
        } else {
            die("no font specified.");
        }

        let font = Box::new(Fnt {
            dpy: self.dpy,
            h: unsafe { xfont.as_ref().ascent + xfont.as_ref().descent } as u32,
            xfont,
            pattern,
            next: None,
        });

        NonNull::new(Box::leak(font))
    }

    pub(crate) fn fontset_create(&mut self, fonts: &[&str]) {
        let mut prev: Option<NonNull<Fnt>> = None;

        if fonts.is_empty() {
            die("We need at least 1 font");
        }

        for i in (1..=fonts.len()).rev() {
            if let Some(mut cur) = self.xfont_create(fonts[fonts.len() - i], None) {
                unsafe { cur.as_mut() }.next = prev;
                // cur.borrow_mut().next = prev;
                prev = Some(cur);
            }
        }
        self.fonts = prev;
    }

    fn clr_create(&mut self, dest: &mut MaybeUninit<Clr>, clrname: &str) {
        let clrname = CString::new(clrname).expect("can create valid CString from &str");

        if unsafe {
            XftColorAllocName(
                self.dpy.as_ptr(),
                default_visual(self.dpy.as_ptr(), self.screen),
                default_colormap(self.dpy.as_ptr(), self.screen),
                clrname.as_ptr(),
                dest as *mut MaybeUninit<Clr> as *mut Clr,
            )
        } == 0
        {
            die(&format!(
                "error, cannot allocate color '{}'",
                clrname.to_string_lossy()
            ));
        }
    }

    pub(crate) fn scm_create(&mut self, clrnames: &[&str]) -> Rc<[Clr]> {
        // need at least 2 colors for a scheme
        if clrnames.len() < 2 {
            die("We need at least 2 colors for a scheme");
        }

        let mut ret = vec![MaybeUninit::uninit(); clrnames.len()].into_boxed_slice();

        for i in 0..clrnames.len() {
            self.clr_create(&mut ret[i], clrnames[i]);
        }
        let ret: Rc<[Clr]> = Rc::from(unsafe { ret.assume_init() });
        ret
    }

    fn clr_free(&mut self, c: &Clr) {
        unsafe {
            XftColorFree(
                self.dpy.as_ptr(),
                default_visual(self.dpy.as_ptr(), self.screen),
                default_colormap(self.dpy.as_ptr(), self.screen),
                c as *const _ as *mut _,
            )
        };
    }

    pub(crate) fn scm_free(&mut self, scm: Rc<[Clr]>) {
        let _ = self.scheme.take();

        for clr in scm.iter() {
            self.clr_free(clr);
        }
    }

    // pub(crate) fn setfontset(&mut self, fnt: NonNull<Fnt>) {
    //     self.fonts = Some(fnt);
    // }

    pub(crate) fn setscheme(&mut self, scm: Rc<[Clr]>) {
        self.scheme = Some(scm);
    }

    pub(crate) fn rect(&mut self, x: i32, y: i32, w: u32, h: u32, filled: bool, invert: bool) {
        let Some(scheme) = &self.scheme else {
            return;
        };

        unsafe {
            XSetForeground(
                self.dpy.as_ptr(),
                self.gc,
                if invert {
                    scheme[COL_BG].pixel
                } else {
                    scheme[COL_FG].pixel
                },
            )
        };
        if filled {
            unsafe { XFillRectangle(self.dpy.as_ptr(), self.drawable, self.gc, x, y, w, h) };
        } else {
            unsafe {
                XDrawRectangle(
                    self.dpy.as_ptr(),
                    self.drawable,
                    self.gc,
                    x,
                    y,
                    w - 1,
                    h - 1,
                )
            };
        }
    }

    //TODO: Double check unused variables
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn text(
        &mut self,
        mut x: i32,
        y: i32,
        mut w: u32,
        h: u32,
        lpad: u32,
        mut text: *const i8,
        invert: bool,
    ) -> i32 {
        let mut ty: i32;
        let mut ellipsis_x: i32 = 0;
        let mut tmpw: u32 = 0;
        let mut ew: u32;
        let mut ellipsis_w: u32 = 0;
        let mut ellipsis_len: u32;
        let mut hash: Wrapping<u32>;
        let mut h0: u32;
        let mut h1: u32;

        let mut d: *mut XftDraw = core::ptr::null_mut();

        let mut usedfont: NonNull<Fnt>;
        let mut curfont: Option<NonNull<Fnt>>;
        let mut nextfont: Option<NonNull<Fnt>>;

        let mut utf8strlen: i32;
        let mut utf8charlen: i32;
        let mut utf8err: bool;
        let render = x != 0 || y != 0 || w != 0 || h != 0;
        let mut utf8codepoint: i64 = 0;
        let mut utf8str: *const i8;

        let mut fccharset: *mut FcCharSet;
        let mut fcpattern: *mut FcPattern;
        let mut match_: Option<NonNull<FcPattern>>;

        let mut charexists = false;
        let mut overflow = false;

        static mut NOMATCHES: [u32; 128] = [0; 128];
        static mut ELLIPSIS_WIDTH: u32 = 0;
        static mut INVALID_WIDTH: u32 = 0;

        const FC_CHARSET: &CStr = c"charset";
        const FC_SCALABLE: &CStr = c"scalable";
        const FCTRUE: i32 = 1;

        const INVALID: &CStr = c"�";

        if (render && (self.scheme.is_none() || w == 0)) || text.is_null() || self.fonts.is_none() {
            return 0;
        }

        if !render {
            w = if invert {
                invert as u32
            } else {
                !(invert as u32)
            };
        } else {
            let scheme = self
                .scheme
                .clone()
                .expect("checked above to exist in case of render");
            unsafe {
                XSetForeground(
                    self.dpy.as_ptr(),
                    self.gc,
                    if invert {
                        scheme[COL_FG].pixel
                    } else {
                        scheme[COL_BG].pixel
                    },
                )
            };
            unsafe { XFillRectangle(self.dpy.as_ptr(), self.drawable, self.gc, x, y, w, h) };
            if w < lpad {
                return x + w as i32;
            }
            d = unsafe {
                XftDrawCreate(
                    self.dpy.as_ptr(),
                    self.drawable,
                    default_visual(self.dpy.as_ptr(), self.screen),
                    default_colormap(self.dpy.as_ptr(), self.screen),
                )
            };
            x += lpad as i32;
            w -= lpad;
        }

        usedfont = self.fonts.expect("checked above to be valid");

        // dbg!("!!");
        if unsafe { ELLIPSIS_WIDTH } == 0 && render {
            unsafe { ELLIPSIS_WIDTH = self.fontset_getwidth(c"...".as_ptr()) };
        }
        if unsafe { INVALID_WIDTH } == 0 && render {
            unsafe { INVALID_WIDTH = self.fontset_getwidth(INVALID.as_ptr()) };
        }

        loop {
            ew = 0;
            ellipsis_len = 0;
            utf8err = false;
            utf8strlen = 0;
            utf8str = text;
            nextfont = None;

            while unsafe { *text } != 0 {
                (utf8charlen, utf8codepoint, utf8err) = utf8decode(text);
                curfont = self.fonts;
                while let Some(cf) = curfont {
                    charexists |= unsafe {
                        XftCharExists(
                            self.dpy.as_ptr(),
                            cf.as_ref().xfont.as_ptr(),
                            utf8codepoint as u32,
                        )
                    } != 0;

                    if charexists {
                        font_getexts(cf, text, utf8charlen, Some(&mut tmpw), None);
                        if ew + unsafe { ELLIPSIS_WIDTH } <= w {
                            /* keep track where the ellipsis still fits */
                            ellipsis_x = x + ew as i32;
                            ellipsis_w = w - ew;
                            ellipsis_len = utf8strlen as u32;
                        }

                        if ew + tmpw > w {
                            overflow = true;
                            /* called from drw_fontset_getwidth_clamp():
                             * it wants the width AFTER the overflow */
                            if !render {
                                x += tmpw as i32;
                            } else {
                                utf8strlen = ellipsis_len as i32;
                            }
                        } else if cf == usedfont {
                            text = unsafe { text.add(utf8charlen as usize) };
                            utf8strlen += if utf8err { 0 } else { utf8charlen };
                            ew += if utf8err { 0 } else { tmpw };
                        } else {
                            nextfont = Some(cf);
                        }
                        break;
                    }
                    curfont = unsafe { cf.as_ref() }.next
                }

                if overflow || !charexists || nextfont.is_some() || utf8err {
                    break;
                } else {
                    charexists = false;
                }
            }

            if utf8strlen != 0 {
                if render {
                    let scheme = self
                        .scheme
                        .clone()
                        .expect("checked above to exist in case of render");
                    ty = y
                        + (h as i32 - unsafe { usedfont.as_ref().h as i32 }) / 2
                        + unsafe { usedfont.as_ref().xfont.as_ref() }.ascent;
                    unsafe {
                        XftDrawStringUtf8(
                            d,
                            &scheme[if invert { COL_BG } else { COL_FG }],
                            usedfont.as_ref().xfont.as_ptr(),
                            x,
                            ty,
                            utf8str as *const u8,
                            utf8strlen,
                        )
                    };
                }
                x += ew as i32;
                w -= ew;
            }

            if utf8err && (!render || unsafe { INVALID_WIDTH } < w) {
                if render {
                    self.text(x, y, w, h, 0, INVALID.as_ptr(), invert);
                }
                x += unsafe { INVALID_WIDTH } as i32;
                w -= unsafe { INVALID_WIDTH };
            }

            if render && overflow {
                self.text(ellipsis_x, y, ellipsis_w, h, 0, c"...".as_ptr(), invert);
            }

            if unsafe { *text } == 0 || overflow {
                break;
            } else if let Some(nextfont) = nextfont {
                charexists = false;
                usedfont = nextfont;
            } else {
                /* Regardless of whether or not a fallback font is found, the
                 * character must be drawn. */
                charexists = true;

                hash = Wrapping(utf8codepoint as u32);
                hash = ((hash >> 16) ^ hash) * Wrapping(0x21F0AAAD);
                hash = ((hash >> 15) ^ hash) * Wrapping(0xD35A2D97);
                h0 = ((hash.0 >> 15) ^ hash.0) % unsafe { NOMATCHES }.len() as u32;
                h1 = (hash.0 >> 17) % unsafe { NOMATCHES }.len() as u32;

                /* avoid expensive XftFontMatch call when we know we won't find a match */
                if unsafe { NOMATCHES }[h0 as usize] as i64 == utf8codepoint
                    || unsafe { NOMATCHES }[h1 as usize] as i64 == utf8codepoint
                {
                    usedfont = if let Some(font) = self.fonts {
                        font
                    } else {
                        unreachable!()
                    };
                    continue;
                }

                fccharset = unsafe { FcCharSetCreate() };
                unsafe { FcCharSetAddChar(fccharset, utf8codepoint as u32) };

                if let Some(font) = self.fonts
                    && unsafe { font.as_ref() }.pattern.is_none()
                {
                    die("the first font in the cache must be loaded from a font string.");
                }

                fcpattern = unsafe {
                    FcPatternDuplicate(
                        self.fonts
                            .expect("checked above to exist")
                            .as_ref()
                            .pattern
                            .expect("pattern should exist")
                            .as_ptr(),
                    )
                };
                unsafe { FcPatternAddCharSet(fcpattern, FC_CHARSET.as_ptr(), fccharset) };
                unsafe { FcPatternAddBool(fcpattern, FC_SCALABLE.as_ptr(), FCTRUE) };

                unsafe {
                    FcConfigSubstitute(
                        core::ptr::null_mut(),
                        fcpattern,
                        crate::external_functions::FcMatchKind::Pattern,
                    )
                };
                unsafe { FcDefaultSubstitute(fcpattern) };
                let mut res = FcResult::Match;
                match_ = NonNull::new(unsafe {
                    XftFontMatch(self.dpy.as_ptr(), self.screen, fcpattern, &mut res)
                });
                unsafe { FcCharSetDestroy(fccharset) };
                unsafe { FcPatternDestroy(fcpattern) };

                if let Some(match_) = match_ {
                    // let mut usedfont = ;
                    if let Some(uf) = self.xfont_create("", Some(match_))
                        && unsafe {
                            XftCharExists(
                                self.dpy.as_ptr(),
                                uf.as_ref().xfont.as_ptr(),
                                utf8codepoint as u32,
                            )
                        } != 0
                    {
                        curfont = self.fonts;
                        while let Some(mut cf) = curfont {
                            if unsafe { cf.as_ref().next.is_some() } {
                                curfont = unsafe { cf.as_ref().next };
                            } else {
                                unsafe { cf.as_mut().next = Some(uf) };
                                break;
                            }
                        }
                        usedfont = uf;
                    } else {
                        //TODO:xfont_free destructor;
                        // self.xfont_free(usedfont)
                        unsafe {
                            NOMATCHES[if NOMATCHES[h0 as usize] != 0 {
                                h1 as usize
                            } else {
                                h0 as usize
                            }] = utf8codepoint as u32;
                        };
                        usedfont = self.fonts.expect("known to be non-null");
                    }
                }
            }
        }

        if !d.is_null() {
            unsafe { XftDrawDestroy(d) }
        }
        x + if render { w } else { 0 } as i32
    }

    pub(crate) fn map(&mut self, win: Window, x: i32, y: i32, w: u32, h: u32) {
        unsafe {
            XCopyArea(
                self.dpy.as_ptr(),
                self.drawable,
                win,
                self.gc,
                x,
                y,
                w,
                h,
                x,
                y,
            )
        };
        unsafe { XSync(self.dpy.as_ptr(), 0) };
    }

    pub(crate) fn fontset_getwidth(&mut self, text: *const i8) -> u32 {
        if self.fonts.is_none() || text.is_null() {
            return 0;
        }

        self.text(0, 0, 0, 0, 0, text, false) as u32
    }

    // fn fontset_getwidth_clamp(&mut self, text: *const i8, n: u32) -> u32 {
    //     let mut tmp = 0u32;
    //     if self.fonts.is_some() && !text.is_null() && n > 0 {
    //         tmp = self.text(0, 0, 0, 0, 0, text, true) as u32
    //     }
    //     n.min(tmp)
    // }

    pub(crate) fn cur_create(&mut self, shape: u32) -> Cur {
        Cur {
            cursor: unsafe { XCreateFontCursor(self.dpy.as_ptr(), shape) },
        }
    }

    pub(crate) fn cur_free(&mut self, cursor: Cur) {
        unsafe { XFreeCursor(self.dpy.as_ptr(), cursor.cursor) };
    }
}

fn font_getexts(
    font: NonNull<Fnt>,
    ch: *const i8,
    len: i32,
    w: Option<&mut u32>,
    h: Option<&mut u32>,
) {
    let mut ext: MaybeUninit<XGlyphInfo> = MaybeUninit::uninit();
    unsafe {
        XftTextExtentsUtf8(
            font.as_ref().dpy.as_ptr(),
            font.as_ref().xfont.as_ptr(),
            ch as *const u8,
            len,
            (&mut ext) as *mut MaybeUninit<XGlyphInfo> as *mut XGlyphInfo,
        )
    }
    let ext = unsafe { ext.assume_init() };
    if let Some(w) = w {
        *w = ext.x_off as u32;
    }
    if let Some(h) = h {
        *h = unsafe { font.as_ref().h }
    }
}

impl Drop for Drw {
    fn drop(&mut self) {
        unsafe { XFreePixmap(self.dpy.as_ptr(), self.drawable) };
        unsafe { XFreeGC(self.dpy.as_ptr(), self.gc) };
        if let Some(fnt) = self.fonts.take() {
            let _ = unsafe { Box::from_raw(fnt.as_ptr()) };
        }
    }
}

impl Drop for Fnt {
    fn drop(&mut self) {
        if let Some(next) = self.next.take() {
            let _ = unsafe { Box::from_raw(next.as_ptr()) };
        }
        if let Some(pat) = self.pattern {
            unsafe { FcPatternDestroy(pat.as_ptr()) };
        }
        unsafe { XftFontClose(self.dpy.as_ptr(), self.xfont.as_ptr()) };
    }
}
