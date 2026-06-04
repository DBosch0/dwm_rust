#![allow(dead_code)]

use std::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ulong, c_ushort, c_void};

#[link(name = "X11")]
unsafe extern "C" {
    pub(crate) fn XCreateGC(
        display: *mut Display,
        d: Drawable,
        valuemask: c_ulong,
        values: *const (),
    ) -> GC;

    pub(crate) fn XCreatePixmap(
        display: *mut Display,
        d: Drawable,
        width: c_uint,
        height: c_uint,
        depth: c_uint,
    ) -> Pixmap;

    pub(crate) fn XSetLineAttributes(
        display: *mut Display,
        gc: GC,
        line_width: c_uint,
        line_style: c_int,
        cap_style: c_int,
        join_style: c_int,
    ) -> c_int;

    pub(crate) fn XFreePixmap(display: *mut Display, pixmap: Pixmap) -> c_int;

    pub(crate) fn XFreeGC(display: *mut Display, gc: GC) -> c_int;

    pub(crate) fn XSetForeground(display: *mut Display, gc: GC, foreground: c_ulong) -> c_int;
    pub(crate) fn XFillRectangle(
        display: *mut Display,
        d: Drawable,
        gc: GC,
        x: c_int,
        y: c_int,
        w: c_uint,
        h: c_uint,
    ) -> c_int;

    pub(crate) fn XDrawRectangle(
        display: *mut Display,
        d: Drawable,
        gc: GC,
        x: c_int,
        y: c_int,
        w: c_uint,
        h: c_uint,
    ) -> c_int;

    pub(crate) fn XCopyArea(
        display: *mut Display,
        src: Drawable,
        dest: Drawable,
        gc: GC,
        src_x: c_int,
        src_y: c_int,
        width: c_uint,
        height: c_uint,
        dest_x: c_int,
        dest_y: c_int,
    ) -> c_int;

    pub(crate) fn XSync(display: *mut Display, discard: c_int) -> c_int;

    pub(crate) fn XCreateFontCursor(display: *mut Display, shape: c_uint) -> Cursor;
    pub(crate) fn XFreeCursor(display: *mut Display, cursor: Cursor) -> c_int;

    pub(crate) fn XSupportsLocale() -> c_int;

    pub(crate) fn XOpenDisplay(display_name: *const c_char) -> *mut Display;

    pub(crate) fn XSetErrorHandler(handler: XErrorHandler) -> XErrorHandler;

    pub(crate) fn XSelectInput(display: *mut Display, w: Window, event_mask: c_long) -> c_int;

    pub(crate) fn XQueryPointer(
        display: *mut Display,
        w: Window,
        root_return: *mut Window,
        child_return: *mut Window,
        root_x_return: *mut c_int,
        root_y_return: *mut c_int,
        win_x_return: *mut c_int,
        win_y_return: *mut c_int,
        mask_return: *mut c_uint,
    ) -> c_int;

    pub(crate) fn XInternAtom(
        display: *mut Display,
        atom_name: *const c_char,
        only_if_exists: c_int,
    ) -> Atom;

    pub(crate) fn XCreateWindow(
        display: *mut Display,
        parent: Window,
        x: c_int,
        y: c_int,
        width: c_uint,
        height: c_uint,
        border_width: c_uint,
        depth: c_int,
        class: c_uint,
        visual: *mut Visual,
        value_mask: c_ulong,
        attributes: *mut XSetWindowAttributes,
    ) -> Window;

    pub(crate) fn XDefineCursor(display: *mut Display, w: Window, cursor: Cursor) -> c_int;
    pub(crate) fn XMapRaised(display: *mut Display, w: Window) -> c_int;
    pub(crate) fn XSetClassHint(
        display: *mut Display,
        w: Window,
        class_hints: *mut XClassHint,
    ) -> c_int;

    pub(crate) fn XGetTextProperty(
        display: *mut Display,
        window: Window,
        text_prop_return: *mut XTextProperty,
        property: Atom,
    ) -> Status;

    pub(crate) fn XmbTextPropertyToTextList(
        display: *mut Display,
        text_prop: *const XTextProperty,
        list_return: *mut *mut *mut c_char,
        count_return: *mut c_int,
    ) -> c_int;

    pub(crate) fn XFreeStringList(list: *mut *mut c_char);
    pub(crate) fn XFree(data: *mut c_void);

    pub(crate) fn XCreateSimpleWindow(
        display: *mut Display,
        parent: Window,
        x: c_int,
        y: c_int,
        width: c_uint,
        height: c_uint,
        border_width: c_uint,
        border: c_ulong,
        background: c_ulong,
    ) -> Window;

    pub(crate) fn XChangeProperty(
        display: *mut Display,
        w: Window,
        property: Atom,
        r#type: Atom,
        format: c_int,
        mode: c_int,
        data: *const c_uchar,
        nelements: c_int,
    ) -> c_int;

    pub(crate) fn XDeleteProperty(display: *mut Display, w: Window, property: Atom) -> c_int;

    pub(crate) fn XChangeWindowAttributes(
        display: *mut Display,
        w: Window,
        value_mask: c_ulong,
        attributes: *mut XSetWindowAttributes,
    ) -> c_int;

    pub(crate) fn XGetModifierMapping(display: *mut Display) -> *mut XModifierKeymap;

    pub(crate) fn XKeysymToKeycode(display: *mut Display, keysym: KeySym) -> KeyCode;
    pub(crate) fn XFreeModifiermap(modmap: *mut XModifierKeymap);

    pub(crate) fn XUngrabKey(
        display: *mut Display,
        keycode: c_int,
        modifiers: c_uint,
        grab_window: Window,
    ) -> c_int;

    pub(crate) fn XDisplayKeycodes(
        display: *mut Display,
        min_keycodes_return: *mut c_int,
        max_keycodes_return: *mut c_int,
    ) -> c_int;

    pub(crate) fn XGetKeyboardMapping(
        display: *mut Display,
        first_keycode: KeyCode,
        keycode_count: c_int,
        keysyms_per_keycode_return: *mut c_int,
    ) -> *mut KeySym;

    pub(crate) fn XGrabKey(
        display: *mut Display,
        keycode: c_int,
        modifiers: c_uint,
        grab_window: Window,
        owner_events: c_int,
        pointer_mode: c_int,
        keyboard_mode: c_int,
    ) -> c_int;

    pub(crate) fn XUngrabButton(
        display: *mut Display,
        button: c_uint,
        modifiers: c_uint,
        grab_window: Window,
    ) -> c_int;

    pub(crate) fn XGrabButton(
        display: *mut Display,
        button: c_uint,
        modifiers: c_uint,
        grab_window: Window,
        owner_events: c_int,
        event_mask: c_uint,
        pointer_mode: c_int,
        keyboard_mode: c_int,
        confine_to: Window,
        cursor: Cursor,
    ) -> c_int;

    pub(crate) fn XSetWindowBorder(
        display: *mut Display,
        w: Window,
        border_pixel: c_ulong,
    ) -> c_int;
    pub(crate) fn XSetInputFocus(
        display: *mut Display,
        focus: Window,
        revert_to: c_int,
        time: Time,
    ) -> c_int;

    pub(crate) fn XGetWMProtocols(
        display: *mut Display,
        w: Window,
        protocols_return: *mut *mut Atom,
        count_return: *mut c_int,
    ) -> Status;

    pub(crate) fn XSendEvent(
        display: *mut Display,
        w: Window,
        propogate: c_int,
        event_mask: c_long,
        event_send: *mut XEvent,
    ) -> Status;

    pub(crate) fn XQueryTree(
        display: *mut Display,
        w: Window,
        root_return: *mut Window,
        parent_return: *mut Window,
        children_return: *mut *mut Window,
        nchildren_return: *mut c_uint,
    ) -> Status;

    pub(crate) fn XGetWindowAttributes(
        display: *mut Display,
        w: Window,
        window_attributes_return: *mut XWindowAttributes,
    ) -> Status;

    pub(crate) fn XGetTransientForHint(
        display: *mut Display,
        w: Window,
        prop_window_return: *mut Window,
    ) -> Status;

    pub(crate) fn XGetWindowProperty(
        display: *mut Display,
        w: Window,
        property: Atom,
        long_offset: c_long,
        long_length: c_long,
        delete: c_int,
        req_type: Atom,
        actual_type_return: *mut Atom,
        actual_format_return: *mut c_int,
        nitems_return: *mut c_ulong,
        bytes_after_return: *mut c_ulong,
        prop_return: *mut *mut c_uchar,
    ) -> c_int;

    pub(crate) fn XGetClassHint(
        display: *mut Display,
        w: Window,
        class_hint_return: *mut XClassHint,
    ) -> Status;

    pub(crate) fn XConfigureWindow(
        display: *mut Display,
        w: Window,
        value_mask: c_uint,
        values: *mut XWindowChanges,
    ) -> c_int;

    pub(crate) fn XRaiseWindow(display: *mut Display, w: Window) -> c_int;

    pub(crate) fn XCheckMaskEvent(
        display: *mut Display,
        event_mask: c_long,
        event_return: *mut XEvent,
    ) -> c_int;

    pub(crate) fn XMoveWindow(display: *mut Display, w: Window, x: c_int, y: c_int) -> c_int;

    pub(crate) fn XGetWMNormalHints(
        display: *mut Display,
        w: Window,
        hints_return: *mut XSizeHints,
        supplied_return: *mut c_long,
    ) -> Status;

    pub(crate) fn XSetWMHints(display: *mut Display, w: Window, wm_hints: *mut XWMHints) -> c_int;
    pub(crate) fn XMoveResizeWindow(
        display: *mut Display,
        w: Window,
        x: c_int,
        y: c_int,
        width: c_uint,
        height: c_uint,
    ) -> c_int;

    pub(crate) fn XMapWindow(display: *mut Display, w: Window) -> c_int;
    pub(crate) fn XCloseDisplay(display: *mut Display) -> c_int;

    pub(crate) fn XNextEvent(display: *mut Display, event_return: *mut XEvent) -> c_int;

    pub(crate) fn XAllowEvents(display: *mut Display, event_mode: c_int, time: Time) -> c_int;
    pub(crate) fn XKeycodeToKeysym(display: *mut Display, keycode: KeyCode, index: c_int)
    -> KeySym;
    pub(crate) fn XRefreshKeyboardMapping(event_map: *mut XMappingEvent) -> c_int;
    pub(crate) fn XSetCloseDownMode(display: *mut Display, close_mode: c_int) -> c_int;
    pub(crate) fn XKillClient(display: *mut Display, resource: XID) -> c_int;
    pub(crate) fn XGrabPointer(
        display: *mut Display,
        grab_window: Window,
        owner_events: c_int,
        event_mask: c_uint,
        pointer_mode: c_int,
        keyboard_mode: c_int,
        confine_to: Window,
        cursor: Cursor,
        time: Time,
    ) -> c_int;

    pub(crate) fn XMaskEvent(
        display: *mut Display,
        event_mask: c_long,
        event_return: *mut XEvent,
    ) -> c_int;

    pub(crate) fn XUngrabPointer(display: *mut Display, time: Time) -> c_int;
    pub(crate) fn XWarpPointer(
        display: *mut Display,
        src_w: Window,
        dest_w: Window,
        src_x: c_int,
        src_y: c_int,
        src_width: c_uint,
        src_height: c_uint,
        dest_x: c_int,
        dest_y: c_int,
    ) -> c_int;

    pub(crate) fn XGetWMHints(display: *mut Display, w: Window) -> *mut XWMHints;
    pub(crate) fn XGrabServer(display: *mut Display) -> c_int;
    pub(crate) fn XUngrabServer(display: *mut Display) -> c_int;
    pub(crate) fn XDestroyWindow(display: *mut Display, w: Window) -> c_int;
    pub(crate) fn XUnmapWindow(display: *mut Display, w: Window) -> c_int;

    pub(crate) fn XrmGetResource(
        database: XrmDatabase,
        str_name: *const c_char,
        str_class: *const c_char,
        str_type_return: *mut *mut c_char,
        value_return: *mut XrmValue,
    ) -> c_int;

    pub(crate) fn XResourceManagerString(display: *mut Display) -> *const c_char;
    pub(crate) fn XrmGetStringDatabase(data: *const c_char) -> XrmDatabase;
    pub(crate) fn XrmInitialize();
}

#[cfg(feature = "xinerama")]
#[link(name = "Xinerama")]
unsafe extern "C" {
    pub(crate) fn XineramaIsActive(dpy: *mut Display) -> i32;
    pub(crate) fn XineramaQueryScreens(
        dpy: *mut Display,
        number: &mut i32,
    ) -> *mut XineramaScreenInfo;
}

#[link(name = "xcb")]
unsafe extern "C" {
    pub(crate) fn xcb_res_query_client_ids(
        c: *mut xcb_connection_t,
        num_specs: c_uint,
        specs: *const xcb_res_client_id_spec_t,
    ) -> xcb_res_query_client_ids_cookie_t;

    pub(crate) fn xcb_res_query_client_ids_reply(
        c: *mut xcb_connection_t,
        cookie: xcb_res_query_client_ids_cookie_t,
        e: *mut *mut xcb_generic_error_t,
    ) -> *mut xcb_res_query_client_ids_reply_t;

    pub(crate) fn xcb_res_query_client_ids_ids_iterator(
        r: *mut xcb_res_query_client_ids_reply_t,
    ) -> xcb_res_client_id_value_iterator_t;

    pub(crate) fn xcb_res_client_id_value_value(r: *mut xcb_res_client_id_value_t) -> *mut c_uint;
    pub(crate) fn xcb_res_client_id_value_next(i: *mut xcb_res_client_id_value_iterator_t);
}
#[link(name = "X11-xcb")]
unsafe extern "C" {
    pub(crate) fn XGetXCBConnection(dpy: *mut Display) -> *mut xcb_connection_t;
}
#[link(name = "xcb-res")]
unsafe extern "C" {}

#[link(name = "fontconfig")]
unsafe extern "C" {
    pub(crate) fn FcNameParse(name: *const FcChar8) -> *mut FcPattern;
    pub(crate) fn FcPatternDestroy(p: *mut FcPattern);
    pub(crate) fn FcCharSetCreate() -> *mut FcCharSet;
    pub(crate) fn FcCharSetAddChar(fcs: *mut FcCharSet, usc4: FcChar32) -> c_int;
    pub(crate) fn FcPatternDuplicate(p: *const FcPattern) -> *mut FcPattern;
    pub(crate) fn FcPatternAddCharSet(
        p: *mut FcPattern,
        object: *const c_char,
        c: *const FcCharSet,
    ) -> c_int;
    pub(crate) fn FcPatternAddBool(p: *mut FcPattern, object: *const c_char, b: c_int) -> c_int;
    pub(crate) fn FcConfigSubstitute(
        config: *mut FcConfig,
        p: *mut FcPattern,
        kind: FcMatchKind,
    ) -> c_int;
    pub(crate) fn FcDefaultSubstitute(pattern: *mut FcPattern);
    pub(crate) fn FcCharSetDestroy(fcs: *mut FcCharSet);
}

#[link(name = "Xft")]
unsafe extern "C" {
    pub(crate) fn XftFontOpenName(
        dpy: *mut Display,
        screen: c_int,
        name: *const c_char,
    ) -> *mut XftFont;
    pub(crate) fn XftFontOpenPattern(dpy: *mut Display, pattern: *mut FcPattern) -> *mut XftFont;
    pub(crate) fn XftColorAllocName(
        dpy: *mut Display,
        visual: *const Visual,
        cmap: Colormap,
        name: *const c_char,
        result: *mut XftColor,
    ) -> c_int;
    pub(crate) fn XftColorFree(
        dpy: *mut Display,
        visual: *mut Visual,
        cmap: Colormap,
        color: *mut XftColor,
    );
    pub(crate) fn XftFontClose(dpy: *mut Display, font: *mut XftFont);
    pub(crate) fn XftDrawCreate(
        dpy: *mut Display,
        drawable: Drawable,
        visual: *mut Visual,
        colormap: Colormap,
    ) -> *mut XftDraw;

    pub(crate) fn XftCharExists(dpy: *mut Display, font: *mut XftFont, usc4: FcChar32) -> c_int;

    pub(crate) fn XftTextExtentsUtf8(
        dpy: *mut Display,
        font: *mut XftFont,
        string: *const FcChar8,
        len: c_int,
        extents: *mut XGlyphInfo,
    );
    pub(crate) fn XftFontMatch(
        dpy: *mut Display,
        screen: c_int,
        pattern: *const FcPattern,
        result: *mut FcResult,
    ) -> *mut FcPattern;

    pub(crate) fn XftDrawDestroy(draw: *mut XftDraw);

    pub(crate) fn XftDrawStringUtf8(
        draw: *mut XftDraw,
        color: *const XftColor,
        r#pub: *mut XftFont,
        x: c_int,
        y: c_int,
        string: *const FcChar8,
        len: c_int,
    );
}

pub(crate) type FcChar8 = c_uchar;
pub(crate) type FcChar32 = c_uint;

#[allow(clippy::upper_case_acronyms)]
pub(crate) type XID = c_ulong;
pub(crate) type Cursor = XID;
pub(crate) type Drawable = XID;
pub(crate) type Window = XID;
pub(crate) type Pixmap = XID;
pub(crate) type Colormap = XID;
pub(crate) type KeySym = XID;
pub(crate) type Status = c_int;
pub(crate) type Time = c_ulong;

pub(crate) type Display = _XDisplay;
pub(crate) enum _XDisplay {}

#[repr(C)]
pub(crate) struct XftFont {
    pub(crate) ascent: c_int,
    pub(crate) descent: c_int,
    pub(crate) height: c_int,
    pub(crate) max_advance_width: c_int,
    pub(crate) charset: *mut FcCharSet,
    pub(crate) pattern: *mut FcPattern,
}

pub(crate) type FcCharSet = _FcCharSet;
pub(crate) enum _FcCharSet {}

pub(crate) type FcPattern = _FcPattern;
pub(crate) enum _FcPattern {}

pub(crate) type FcConfig = _FcConfig;
pub(crate) enum _FcConfig {}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct XftColor {
    pub(crate) pixel: c_ulong,
    pub(crate) color: XRenderColor,
}

pub(crate) type XftDraw = _XftDraw;
pub(crate) enum _XftDraw {}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct XRenderColor {
    red: c_ushort,
    green: c_ushort,
    blue: c_ushort,
    alpha: c_ushort,
}

pub(crate) type GC = *mut _XGC;

pub(crate) enum _XGC {}
pub(crate) enum _XPrivate {}
pub(crate) enum _XrmHashBucketRec {}
pub(crate) type XrmDatabase = *mut _XrmHashBucketRec;

#[allow(non_camel_case_types)]
pub(crate) enum xcb_connection_t {}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct xcb_res_client_id_spec_t {
    pub(crate) client: c_uint,
    pub(crate) mask: c_uint,
}

#[repr(C)]
pub(crate) struct xcb_generic_error_t {
    pub(crate) response_type: c_uchar, /*< Type of the response */
    pub(crate) error_code: c_uchar,    /*< Error code */
    pub(crate) sequence: c_ushort,     /*< Sequence number */
    pub(crate) resource_id: c_uint,    /* < Resource ID for requests with side effects only */
    pub(crate) minor_code: c_ushort,   /* < Minor opcode of the failed request */
    pub(crate) major_code: c_uchar,    /* < Major opcode of the failed request */
    pub(crate) pad0: c_uchar,
    pub(crate) pad: [c_uint; 5],      /*< Padding */
    pub(crate) full_sequence: c_uint, /*< full sequence */
}

#[repr(C)]
pub(crate) struct xcb_res_query_client_ids_cookie_t {
    pub(crate) sequence: c_uint,
}

#[repr(C)]
pub(crate) struct xcb_res_query_client_ids_reply_t {
    response_type: c_uchar,
    pad0: c_uchar,
    sequence: c_ushort,
    length: c_uint,
    num_ids: c_uint,
    pad1: [c_uchar; 20],
}

#[repr(C)]
pub(crate) struct xcb_res_client_id_value_t {
    pub(crate) spec: xcb_res_client_id_spec_t,
    pub(crate) length: c_uint,
}

#[repr(C)]
pub(crate) struct xcb_res_client_id_value_iterator_t {
    pub(crate) data: *mut xcb_res_client_id_value_t,
    pub(crate) rem: c_int,
    pub(crate) index: c_int,
}

#[cfg(feature = "xinerama")]
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XineramaScreenInfo {
    pub(crate) screen_number: i32,
    pub(crate) x_org: i16,
    pub(crate) y_org: i16,
    pub(crate) width: i16,
    pub(crate) height: i16,
}

#[repr(C)]
struct _XPrivDisplay {
    pub(crate) ext_data: *mut XExtData,
    pub(crate) private1: *mut _XPrivate,
    pub(crate) fd: c_int,
    pub(crate) private2: c_int,
    pub(crate) proto_major_version: c_int, /* major version of server's X protocol */
    pub(crate) proto_minor_version: c_int, /* minor version of servers X protocol */
    pub(crate) vendor: *mut c_char,        /* vendor of the server hardware */
    pub(crate) private3: XID,
    pub(crate) private4: XID,
    pub(crate) private5: XID,
    pub(crate) private6: c_int,
    pub(crate) resource_alloc: fn(display: *mut _XDisplay) -> XID, /* allocator function */
    pub(crate) byte_order: c_int, /* screen byte order, LSBFirst, MSBFirst */
    pub(crate) bitmap_unit: c_int, /* padding and data requirements */
    pub(crate) bitmap_pad: c_int, /* padding requirements on bitmaps */
    pub(crate) bitmap_bit_order: c_int, /* LeastSignificant or MostSignificant */
    pub(crate) nformats: c_int,   /* LeastSignificant or MostSignificant */
    pub(crate) pixmap_format: *mut ScreenFormat, /* pixmap format list */
    pub(crate) private8: c_int,
    pub(crate) release: c_int, /* release of the server */
    pub(crate) private9: *mut _XPrivate,
    pub(crate) private10: *mut _XPrivate,
    pub(crate) qlen: c_int,                /* Length of input event queue */
    pub(crate) last_request_read: c_ulong, /* seq number of last event read */
    pub(crate) request: c_ulong,           /* sequence number of last request. */
    pub(crate) private11: XPointer,
    pub(crate) private12: XPointer,
    pub(crate) private13: XPointer,
    pub(crate) private14: XPointer,
    pub(crate) max_request_size: c_uint, /* maximum number 32 bit words in request*/
    pub(crate) db: *mut _XrmHashBucketRec,
    pub(crate) private15: fn(display: *mut _XDisplay) -> c_int,
    pub(crate) display_name: *mut c_char, /* "host:display" string used on this connect*/
    pub(crate) default_screen: c_int,     /* default screen for operations */
    pub(crate) nscreens: c_int,           /* number of screens on this server*/
    pub(crate) screens: *mut Screen,      /* pointer to list of screens */
    pub(crate) motion_buffer: c_ulong,    /* size of motion buffer */
    pub(crate) private16: c_ulong,
    pub(crate) min_keycode: c_int, /* minimum defined keycode */
    pub(crate) max_keycode: c_int, /* maximum defined keycode */
    pub(crate) private17: XPointer,
    pub(crate) private18: XPointer,
    pub(crate) private19: c_int,
    pub(crate) xdefaults: *mut c_char, /* contents of defaults from server */
}

#[repr(C)]
pub(crate) struct XExtData {
    number: c_int,
    next: *mut XExtData,
    free_private: extern "C" fn(extension: *mut XExtData),
    private_data: XPointer,
}

#[repr(C)]
pub(crate) struct XModifierKeymap {
    pub(crate) max_keypermod: c_int,
    pub(crate) modifiermap: *mut KeyCode,
}

pub(crate) type KeyCode = c_uchar;

#[repr(C)]
pub(crate) union XEvent {
    pub(crate) r#type: c_int,
    pub(crate) xclient: XClientMessageEvent,
    pub(crate) xbutton: XButtonEvent,
    pub(crate) xconfigurerequest: XConfigureRequestEvent,
    pub(crate) xconfigure: XConfigureEvent,
    pub(crate) xdestroywindow: XDestroyWindowEvent,
    pub(crate) xcrossing: XCrossingEvent,
    pub(crate) xexpose: XExposeEvent,
    pub(crate) xfocus: XFocusChangeEvent,
    pub(crate) xkey: XKeyEvent,
    pub(crate) xmapping: XMappingEvent,
    pub(crate) xmaprequest: XMapRequestEvent,
    pub(crate) xmotion: XMotionEvent,
    pub(crate) xunmap: XUnmapEvent,
    pub(crate) xproperty: XPropertyEvent,
    pad: [c_long; 24],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XClientMessageEvent {
    pub(crate) r#type: c_int,
    pub(crate) serial: c_ulong,
    pub(crate) send_event: c_int,
    pub(crate) display: *mut Display,
    pub(crate) window: Window,
    pub(crate) message_type: Atom,
    pub(crate) format: c_int,
    pub(crate) data: XClientMessageEventData,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) union XClientMessageEventData {
    pub(crate) b: [c_char; 20],
    pub(crate) s: [c_short; 10],
    pub(crate) l: [c_long; 5],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XButtonEvent {
    pub(crate) r#type: c_int,         /* of event */
    pub(crate) serial: c_ulong,       /* # of last request processed by server */
    pub(crate) send_event: c_int,     /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) window: Window,        /* "event" window it is reported relative to */
    pub(crate) root: Window,          /* root window that the event occurred on */
    pub(crate) subwindow: Window,     /* child window */
    pub(crate) time: Time,            /* milliseconds */
    pub(crate) x: c_int,              /* pointer x, y coordinates in event window */
    pub(crate) y: c_int,
    pub(crate) x_root: c_int, /* coordinates relative to root */
    pub(crate) y_root: c_int,
    pub(crate) state: c_uint,      /* key or button mask */
    pub(crate) button: c_uint,     /* detail */
    pub(crate) same_screen: c_int, /* same screen flag */
}

pub(crate) type XButtonPressedEvent = XButtonEvent;
pub(crate) type XButtonReleasedEvent = XButtonEvent;

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XConfigureRequestEvent {
    pub(crate) r#type: c_int,
    pub(crate) serial: c_ulong, /* # of last request processed by server */
    pub(crate) send_event: c_int, /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) parent: Window,
    pub(crate) window: Window,
    pub(crate) x: c_int,
    pub(crate) y: c_int,
    pub(crate) width: c_int,
    pub(crate) height: c_int,
    pub(crate) border_width: c_int,
    pub(crate) above: Window,
    pub(crate) detail: c_int, /* Above, Below, TopIf, BottomIf, Opposite */
    pub(crate) value_mask: c_ulong,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XDestroyWindowEvent {
    pub(crate) r#type: c_int,
    pub(crate) serial: c_ulong, /* # of last request processed by server */
    pub(crate) send_event: c_int, /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) event: Window,
    pub(crate) window: Window,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XCrossingEvent {
    pub(crate) r#type: c_int,         /* of event */
    pub(crate) serial: c_ulong,       /* # of last request processed by server */
    pub(crate) send_event: c_int,     /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) window: Window,        /* "event" window reported relative to */
    pub(crate) root: Window,          /* root window that the event occurred on */
    pub(crate) subwindow: Window,     /* child window */
    pub(crate) time: Time,            /* milliseconds */
    pub(crate) x: c_int,              /* pointer x, y coordinates in event window */
    pub(crate) y: c_int,
    pub(crate) x_root: c_int, /* coordinates relative to root */
    pub(crate) y_root: c_int,
    pub(crate) mode: c_int, /* NotifyNormal, NotifyGrab, NotifyUngrab */
    pub(crate) detail: c_int, /*
                             * NotifyAncestor, NotifyVirtual, NotifyInferior,
                             * NotifyNonlinear,NotifyNonlinearVirtual
                             */
    pub(crate) same_screen: c_int, /* same screen flag */
    pub(crate) focus: c_int,       /* boolean focus */
    pub(crate) state: c_uint,      /* key or button mask */
}
type XEnterWindowEvent = XCrossingEvent;
type XLeaveWindowEvent = XCrossingEvent;

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XExposeEvent {
    pub(crate) r#type: c_int,
    pub(crate) serial: c_ulong, /* # of last request processed by server */
    pub(crate) send_event: c_int, /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) window: Window,
    pub(crate) x: c_int,
    pub(crate) y: c_int,
    pub(crate) width: c_int,
    pub(crate) height: c_int,
    pub(crate) count: c_int, /* if non-zero, at least this many more */
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XFocusChangeEvent {
    pub(crate) r#type: c_int,         /* FocusIn or FocusOut */
    pub(crate) serial: c_ulong,       /* # of last request processed by server */
    pub(crate) send_event: c_int,     /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) window: Window,        /* window of event */
    pub(crate) mode: c_int,           /* NotifyNormal, NotifyWhileGrabbed,
                                      NotifyGrab, NotifyUngrab */
    pub(crate) detail: c_int, /*
                               * NotifyAncestor, NotifyVirtual, NotifyInferior,
                               * NotifyNonlinear,NotifyNonlinearVirtual, NotifyPointer,
                               * NotifyPointerRoot, NotifyDetailNone
                               */
}
type XFocusInEvent = XFocusChangeEvent;
type XFocusOutEvent = XFocusChangeEvent;

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XKeyEvent {
    pub(crate) r#type: c_int,         /* of event */
    pub(crate) serial: c_ulong,       /* # of last request processed by server */
    pub(crate) send_event: c_int,     /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) window: Window,        /* "event" window it is reported relative to */
    pub(crate) root: Window,          /* root window that the event occurred on */
    pub(crate) subwindow: Window,     /* child window */
    pub(crate) time: Time,            /* milliseconds */
    pub(crate) x: c_int,              /* pointer x, y coordinates in event window */
    pub(crate) y: c_int,
    pub(crate) x_root: c_int, /* coordinates relative to root */
    pub(crate) y_root: c_int,
    pub(crate) state: c_uint,      /* key or button mask */
    pub(crate) keycode: c_uint,    /* detail */
    pub(crate) same_screen: c_int, /* same screen flag */
}

type XKeyPressedEvent = XKeyEvent;
type XKeyReleasedEvent = XKeyEvent;

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XMappingEvent {
    pub(crate) r#type: c_int,         /* of event */
    pub(crate) serial: c_ulong,       /* # of last request processed by server */
    pub(crate) send_event: c_int,     /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) window: Window,        /* unused */
    pub(crate) request: c_int,        /* one of MappingModifier, MappingKeyboard,
                                      MappingPointer */
    pub(crate) first_keycode: c_int, /* first keycode */
    pub(crate) count: c_int,         /* defines range of change w. first_keycode*/
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XMapRequestEvent {
    pub(crate) r#type: c_int,
    pub(crate) serial: c_ulong, /* # of last request processed by server */
    pub(crate) send_event: c_int, /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) parent: Window,
    pub(crate) window: Window,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XMotionEvent {
    pub(crate) r#type: c_int,
    pub(crate) serial: c_ulong, /* # of last request processed by server */
    pub(crate) send_event: c_int, /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) window: Window,
    pub(crate) root: Window,
    pub(crate) subwindow: Window,
    pub(crate) time: Time,
    pub(crate) x: c_int,
    pub(crate) y: c_int,
    pub(crate) x_root: c_int,
    pub(crate) y_root: c_int,
    pub(crate) state: c_uint,
    pub(crate) is_hint: c_char,
    pub(crate) same_screen: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XUnmapEvent {
    pub(crate) r#type: c_int,
    pub(crate) serial: c_ulong, /* # of last request processed by server */
    pub(crate) send_event: c_int, /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) event: Window,
    pub(crate) window: Window,
    pub(crate) from_configure: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XPropertyEvent {
    pub(crate) r#type: c_int,
    pub(crate) serial: c_ulong, /* # of last request processed by server */
    pub(crate) send_event: c_int, /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) window: Window,
    pub(crate) atom: Atom,
    pub(crate) time: Time,
    pub(crate) state: c_int, /* NewValue, Deleted */
}

#[repr(C)]
pub(crate) struct XWindowChanges {
    pub(crate) x: c_int,
    pub(crate) y: c_int,
    pub(crate) width: c_int,
    pub(crate) height: c_int,
    pub(crate) border_width: c_int,
    pub(crate) sibling: Window,
    pub(crate) stack_mode: c_int,
}

pub(crate) type XPointer = *mut c_char;
pub(crate) type Atom = c_ulong;

#[repr(C)]
struct ScreenFormat {
    ext_data: *mut XExtData,
    depth: c_int,
    bits_per_pixel: c_int,
    scanline_pad: c_int,
}

#[repr(C)]
pub(crate) struct Screen {
    pub(crate) ext_data: *mut XExtData, /* hook for extension to hang data */
    pub(crate) display: *mut Display,   /* back pointer to display structure */
    pub(crate) root: Window,            /* Root window id. */
    pub(crate) width: c_int,            /* width and height of screen */
    pub(crate) height: c_int,
    pub(crate) mwidth: c_int, /* width and height of  in millimeters */
    pub(crate) mheight: c_int,
    pub(crate) ndepths: c_int,           /* number of depths possible */
    pub(crate) depths: *mut Depth,       /* list of allowable depths on the screen */
    pub(crate) root_depth: c_int,        /* bits per pixel */
    pub(crate) root_visual: *mut Visual, /* root visual */
    pub(crate) default_gc: GC,           /* GC for the root root visual */
    pub(crate) cmap: Colormap,           /* default color map */
    pub(crate) white_pixel: c_ulong,
    pub(crate) black_pixel: c_ulong, /* White and Black pixel values */
    pub(crate) max_maps: c_int,      /* max and min color maps */
    pub(crate) min_maps: c_int,
    pub(crate) backing_store: c_int, /* Never, WhenMapped, Always */
    pub(crate) save_unders: c_int,
    pub(crate) root_input_mask: c_long, /* initial root input mask */
}

/*
 * Depth structure; contains information for each possible depth.
 */
#[repr(C)]
pub(crate) struct Depth {
    depth: c_int,         /* this depth (Z) of the depth */
    nvisuals: c_int,      /* number of Visual types at this depth */
    visuals: *mut Visual, /* list of visuals possible at this depth */
}

/*
 * Visual structure; contains information about colormapping possible.
 */
#[repr(C)]
pub(crate) struct Visual {
    ext_data: *mut XExtData, /* hook for extension to hang data */
    visualid: VisualID,      /* visual id of this visual */
    class: c_int,            /* class of screen (monochrome, etc.) */
    red_mask: c_ulong,       /* mask values */
    green_mask: c_ulong,
    blue_mask: c_ulong,
    bits_per_rgb: c_int, /* log base 2 of distinct color values */
    map_entries: c_int,  /* color map entries */
}

type VisualID = c_ulong;

pub(crate) unsafe fn screen_of_display(dpy: *mut Display, src: i32) -> *mut Screen {
    assert!(src >= 0, "src cannot be negative");
    let priv_dpy: *mut _XPrivDisplay = dpy.cast();
    unsafe { (*priv_dpy).screens.add(src as usize) }
}

pub(crate) unsafe fn default_depth(dpy: *mut Display, src: i32) -> i32 {
    (unsafe { &*screen_of_display(dpy, src) }).root_depth
}

pub(crate) unsafe fn default_visual(dpy: *mut Display, src: i32) -> *mut Visual {
    (unsafe { &*screen_of_display(dpy, src) }).root_visual
}

pub(crate) unsafe fn default_colormap(dpy: *mut Display, src: i32) -> Colormap {
    (unsafe { &*screen_of_display(dpy, src) }).cmap
}

pub(crate) unsafe fn default_width(dpy: *mut Display, src: i32) -> i32 {
    (unsafe { &*screen_of_display(dpy, src) }).width
}

pub(crate) unsafe fn default_height(dpy: *mut Display, src: i32) -> i32 {
    (unsafe { &*screen_of_display(dpy, src) }).height
}

pub(crate) unsafe fn root_window(dpy: *mut Display, src: i32) -> Window {
    (unsafe { &*screen_of_display(dpy, src) }).root
}

pub(crate) unsafe fn default_screen(dpy: *mut Display) -> i32 {
    let priv_dpy: *mut _XPrivDisplay = dpy.cast();
    { unsafe { &*priv_dpy } }.default_screen
}

pub(crate) unsafe fn default_root_window(dpy: *mut Display) -> Window {
    (unsafe { &*screen_of_display(dpy, default_screen(dpy)) }).root
}

pub(crate) unsafe fn connection_number(dpy: *mut Display) -> i32 {
    let priv_dpy: *mut _XPrivDisplay = dpy.cast();
    unsafe { &*priv_dpy }.fd
}

/// Line Style
pub(crate) const LINE_SOLID: c_int = 0;

/// Cap Style
pub(crate) const CAP_BUTT: c_int = 1;

/// Join Style
pub(crate) const JOIN_MITER: c_int = 0;

pub(crate) type XftResult = FcResult;
#[repr(C)]
pub(crate) enum FcResult {
    Match,
    NoMatch,
    TypeMismatch,
    NoId,
    OutOfMemory,
}

#[repr(C)]
pub(crate) struct XGlyphInfo {
    pub(crate) width: c_ushort,  /* Glyph width. */
    pub(crate) height: c_ushort, /* Glyph height. */
    pub(crate) x: c_short, /* Horizontal Glyph center offset relative to the upper-left corner. */
    pub(crate) y: c_short, /* Vertical Glyph center offset relative to the upper-left corner. */
    pub(crate) x_off: c_short, /* Horizontal margin to the next Glyph. */
    pub(crate) y_off: c_short, /* Vertical margin to the next Glyph. */
}

#[repr(C)]
pub(crate) enum FcMatchKind {
    Pattern = 0,
    Font,
    Scan,
    KindEnd,
    KindBegin,
}

#[repr(C)]
pub(crate) struct XErrorEvent {
    r#type: c_int,
    display: *mut Display,            /* Display the event was read from */
    resourceid: XID,                  /* resource id */
    serial: c_ulong,                  /* serial number of failed request */
    pub(crate) error_code: c_uchar,   /* error code of failed request */
    pub(crate) request_code: c_uchar, /* Major op-code of failed request */
    minor_code: c_uchar,              /* Minor op-code of failed request */
}

#[repr(C)]
pub(crate) struct XTextProperty {
    pub(crate) value: *mut c_uchar,
    pub(crate) encoding: Atom,
    pub(crate) format: c_int,
    pub(crate) nitems: c_ulong,
}

pub(crate) type XErrorHandler =
    extern "C" fn(display: *mut Display, error_event: *mut XErrorEvent) -> c_int;

#[repr(C)]
#[derive(Debug)]
pub(crate) struct XSetWindowAttributes {
    pub(crate) background_pixmap: Pixmap, /* background or None or ParentRelative */
    pub(crate) background_pixel: c_ulong, /* background pixel */
    pub(crate) border_pixmap: Pixmap,     /* border of the window */
    pub(crate) border_pixel: c_ulong,     /* border pixel value */
    pub(crate) bit_gravity: c_int,        /* one of bit gravity values */
    pub(crate) win_gravity: c_int,        /* one of the window gravity values */
    pub(crate) backing_store: c_int,      /* NotUseful, WhenMapped, Always */
    pub(crate) backing_planes: c_ulong,   /* planes to be preserved if possible */
    pub(crate) backing_pixel: c_ulong,    /* value to use in restoring planes */
    pub(crate) save_under: c_int,         /* should bits under be saved? (popups) */
    pub(crate) event_mask: c_long,        /* set of events that should be saved */
    pub(crate) do_not_propogate_mask: c_long, /* set of events that should not propagate */
    pub(crate) override_redirect: c_int,  /* boolean value for override-redirect */
    pub(crate) colormap: Colormap,        /* color map to be associated with window */
    pub(crate) cursor: Cursor,            /* cursor to be displayed (or None) */
}

#[repr(C)]
pub(crate) struct XClassHint {
    pub(crate) res_name: *const c_char,
    pub(crate) res_class: *const c_char,
}

#[repr(C)]
pub(crate) struct XWMHints {
    pub(crate) flags: c_long, /* marks which fields in this structure are defined */
    pub(crate) input: c_int, /* does this application rely on the window manager to get keyboard input? */
    pub(crate) initial_state: c_int, /* see below */
    pub(crate) icon_pixmap: Pixmap, /* pixmap to be used as icon */
    pub(crate) icon_window: Window, /* window to be used as icon */
    pub(crate) icon_x: c_int,
    pub(crate) icon_y: c_int,     /* initial position of icon */
    pub(crate) icon_mask: Pixmap, /* icon mask bitmap */
    pub(crate) window_group: XID, /* id of related window group */
                                  /* this structure may be extended in the future */
}

#[repr(C)]
pub(crate) struct XWindowAttributes {
    pub(crate) x: c_int, /* location of window */
    pub(crate) y: c_int,
    pub(crate) width: c_int, /* width and height of window */
    pub(crate) height: c_int,
    pub(crate) border_width: c_int,     /* border width of window */
    pub(crate) depth: c_int,            /* depth of window */
    pub(crate) visual: *mut Visual,     /* the associated visual structure */
    pub(crate) root: Window,            /* root of screen containing window */
    pub(crate) class: c_int,            /* InputOutput, InputOnly*/
    pub(crate) bit_gravity: c_int,      /* one of bit gravity values */
    pub(crate) win_gravity: c_int,      /* one of the window gravity values */
    pub(crate) backing_store: c_int,    /* NotUseful, WhenMapped, Always */
    pub(crate) backing_planes: c_ulong, /* planes to be preserved if possible */
    pub(crate) backing_pixel: c_ulong,  /* value to be used when restoring planes */
    pub(crate) save_under: c_int,       /* boolean, should bits under be saved? */
    pub(crate) colormap: Colormap,      /* color map to be associated with window */
    pub(crate) map_installed: c_int,    /* boolean, is color map currently installed*/
    pub(crate) map_state: c_int,        /* IsUnmapped, IsUnviewable, IsViewable */
    pub(crate) all_event_mask: c_long,  /* set of events all people have interest in*/
    pub(crate) your_event_mask: c_long, /* my event mask */
    pub(crate) do_not_propogate_mask: c_long, /* set of events that should not propagate */
    pub(crate) override_redirect: c_int, /* boolean value for override-redirect */
    pub(crate) screen: *mut Screen,     /* back pointer to correct screen */
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XConfigureEvent {
    pub(crate) r#type: c_int,
    pub(crate) serial: c_ulong,
    pub(crate) send_event: c_int,
    pub(crate) display: *mut Display,
    pub(crate) event: Window,
    pub(crate) window: Window,
    pub(crate) x: c_int,
    pub(crate) y: c_int,
    pub(crate) width: c_int,
    pub(crate) height: c_int,
    pub(crate) border_width: c_int,
    pub(crate) above: Window,
    pub(crate) override_redirect: c_int,
}

#[repr(C)]
pub(crate) struct XSizeHints {
    pub(crate) flags: c_long, /* marks which fields in this structure are defined */
    pub(crate) x: c_int,      /* obsolete for new window mgrs, but clients */
    pub(crate) y: c_int,
    pub(crate) width: c_int, /* should set so old wm's don't mess up */
    pub(crate) height: c_int,
    pub(crate) min_width: c_int,
    pub(crate) min_height: c_int,
    pub(crate) max_width: c_int,
    pub(crate) max_height: c_int,
    pub(crate) width_inc: c_int,
    pub(crate) height_inc: c_int,
    pub(crate) min_aspect: XSizeHintsAspect,
    pub(crate) max_aspect: XSizeHintsAspect,
    pub(crate) base_width: c_int,  /* added by ICCCM version 1 */
    pub(crate) base_height: c_int, /* added by ICCCM version 1 */
    pub(crate) win_gravity: c_int, /* added by ICCCM version 1 */
}

#[repr(C)]
pub(crate) struct XSizeHintsAspect {
    pub(crate) x: c_int, /* numerator */
    pub(crate) y: c_int, /* denominator */
}

#[repr(C)]
pub(crate) struct XrmValue {
    pub(crate) size: c_uint,
    pub(crate) addr: XPointer,
}

pub(crate) const REVERT_TO_POINTER_ROOT: c_int = 1;
pub(crate) const CURRENT_TIME: Time = 0;

// X RESERVED RESOURCE AND CONSTANT DEFINITIONS
pub(crate) const PARENT_RELATIVE: c_ulong = 1;
pub(crate) const COPY_FROM_PARENT: c_ulong = 0;

// Window Attributes for create window
pub(crate) const CW_BACK_PIXMAP: c_ulong = 1 << 0;
pub(crate) const CW_BACK_PIXEL: c_ulong = 1 << 1;
pub(crate) const CW_BORDER_PIXMAP: c_ulong = 1 << 2;
pub(crate) const CW_BORDER_PIXEL: c_ulong = 1 << 3;
pub(crate) const CW_BIT_GRAVITY: c_ulong = 1 << 4;
pub(crate) const CW_WIN_GRAVITY: c_ulong = 1 << 5;
pub(crate) const CW_BACKING_STORE: c_ulong = 1 << 6;
pub(crate) const CW_BACKING_PLANES: c_ulong = 1 << 7;
pub(crate) const CW_BACKING_PIXEL: c_ulong = 1 << 8;
pub(crate) const CW_OVERRIDE_REDIRECT: c_ulong = 1 << 9;
pub(crate) const CW_SAVE_UNDER: c_ulong = 1 << 10;
pub(crate) const CW_EVENT_MASK: c_ulong = 1 << 11;
pub(crate) const CW_DONT_PROPAGATE: c_ulong = 1 << 12;
pub(crate) const CW_COLORMAP: c_ulong = 1 << 13;
pub(crate) const CW_CURSOR: c_ulong = 1 << 14;

//EVENT DEFINITIONS
pub(crate) const NO_EVENT_MASK: c_long = 0;
pub(crate) const KEY_PRESS_MASK: c_long = 1 << 0;
pub(crate) const KEY_RELEASE_MASK: c_long = 1 << 1;
pub(crate) const BUTTON_PRESS_MASK: c_long = 1 << 2;
pub(crate) const BUTTON_RELEASE_MASK: c_long = 1 << 3;
pub(crate) const ENTER_WINDOW_MASK: c_long = 1 << 4;
pub(crate) const LEAVE_WINDOW_MASK: c_long = 1 << 5;
pub(crate) const POINTER_MOTION_MASK: c_long = 1 << 6;
pub(crate) const POINTER_MOTION_HINT_MASK: c_long = 1 << 7;
pub(crate) const BUTTON1_MOTION_MASK: c_long = 1 << 8;
pub(crate) const BUTTON2_MOTION_MASK: c_long = 1 << 9;
pub(crate) const BUTTON3_MOTION_MASK: c_long = 1 << 10;
pub(crate) const BUTTON4_MOTION_MASK: c_long = 1 << 11;
pub(crate) const BUTTON5_MOTION_MASK: c_long = 1 << 12;
pub(crate) const BUTTON_MOTION_MASK: c_long = 1 << 13;
pub(crate) const KEYMAP_STATE_MASK: c_long = 1 << 14;
pub(crate) const EXPOSURE_MASK: c_long = 1 << 15;
pub(crate) const VISIBILITY_CHANGE_MASK: c_long = 1 << 16;
pub(crate) const STRUCTURE_NOTIFY_MASK: c_long = 1 << 17;
pub(crate) const RESIZE_REDIRECT_MASK: c_long = 1 << 18;
pub(crate) const SUBSTRUCTURE_NOTIFY_MASK: c_long = 1 << 19;
pub(crate) const SUBSTRUCTURE_REDIRECT_MASK: c_long = 1 << 20;
pub(crate) const FOCUS_CHANGE_MASK: c_long = 1 << 21;
pub(crate) const PROPERTY_CHANGE_MASK: c_long = 1 << 22;
pub(crate) const COLORMAP_CHANGE_MASK: c_long = 1 << 23;
pub(crate) const OWNER_GRAB_BUTTON_MASK: c_long = 1 << 24;

// Event Names

pub(crate) const KEY_PRESS: c_int = 2;
pub(crate) const KEY_RELEASE: c_int = 3;
pub(crate) const BUTTON_PRESS: c_int = 4;
pub(crate) const BUTTON_RELEASE: c_int = 5;
pub(crate) const MOTION_NOTIFY: c_int = 6;
pub(crate) const ENTER_NOTIFY: c_int = 7;
pub(crate) const LEAVE_NOTIFY: c_int = 8;
pub(crate) const FOCUS_IN: c_int = 9;
pub(crate) const FOCUS_OUT: c_int = 10;
pub(crate) const KEYMAP_NOTIFY: c_int = 11;
pub(crate) const EXPOSE: c_int = 12;
pub(crate) const GRAPHICS_EXPOSE: c_int = 13;
pub(crate) const NO_EXPOSE: c_int = 14;
pub(crate) const VISIBILITY_NOTIFY: c_int = 15;
pub(crate) const CREATE_NOTIFY: c_int = 16;
pub(crate) const DESTROY_NOTIFY: c_int = 17;
pub(crate) const UNMAP_NOTIFY: c_int = 18;
pub(crate) const MAP_NOTIFY: c_int = 19;
pub(crate) const MAP_REQUEST: c_int = 20;
pub(crate) const REPARENT_NOTIFY: c_int = 21;
pub(crate) const CONFIGURE_NOTIFY: c_int = 22;
pub(crate) const CONFIGURE_REQUEST: c_int = 23;
pub(crate) const GRAVITY_NOTIFY: c_int = 24;
pub(crate) const RESIZE_REQUEST: c_int = 25;
pub(crate) const CIRCULATE_NOTIFY: c_int = 26;
pub(crate) const CIRCULATE_REQUEST: c_int = 27;
pub(crate) const PROPERTY_NOTIFY: c_int = 28;
pub(crate) const SELECTION_CLEAR: c_int = 29;
pub(crate) const SELECTION_REQUEST: c_int = 30;
pub(crate) const SELECTION_NOTIFY: c_int = 31;
pub(crate) const COLORMAP_NOTIFY: c_int = 32;
pub(crate) const CLIENT_MESSAGE: c_int = 33;
pub(crate) const MAPPING_NOTIFY: c_int = 34;
pub(crate) const GENERIC_EVENT: c_int = 35;
pub(crate) const LAST_EVENT: c_int = 36; /* must be bigger than any event # */

// Error Codes
pub(crate) const SUCCESS: c_uchar = 0;
pub(crate) const BAD_WINDOW: c_uchar = 3;
pub(crate) const BAD_MATCH: c_uchar = 8;
pub(crate) const BAD_DRAWABLE: c_uchar = 9;
pub(crate) const BAD_ACCESS: c_uchar = 10;

// Request Codes
pub(crate) const X_CREATE_WINDOW: c_uchar = 1;
pub(crate) const X_CHANGE_WINDOW_ATTRIBUTES: c_uchar = 2;
pub(crate) const X_GET_WINDOW_ATTRIBUTES: c_uchar = 3;
pub(crate) const X_DESTROY_WINDOW: c_uchar = 4;
pub(crate) const X_DESTROY_SUBWINDOWS: c_uchar = 5;
pub(crate) const X_CHANGE_SAVE_SET: c_uchar = 6;
pub(crate) const X_REPARENTWINDOW: c_uchar = 7;
pub(crate) const X_MAPWINDOW: c_uchar = 8;
pub(crate) const X_MAPSUBWINDOWS: c_uchar = 9;
pub(crate) const X_UNMAPWINDOW: c_uchar = 10;
pub(crate) const X_UNMAPSUBWINDOWS: c_uchar = 11;
pub(crate) const X_CONFIGUREWINDOW: c_uchar = 12;
pub(crate) const X_CIRCULATEWINDOW: c_uchar = 13;
pub(crate) const X_GETGEOMETRY: c_uchar = 14;
pub(crate) const X_QUERYTREE: c_uchar = 15;
pub(crate) const X_INTERNATOM: c_uchar = 16;
pub(crate) const X_GETATOMNAME: c_uchar = 17;
pub(crate) const X_CHANGEPROPERTY: c_uchar = 18;
pub(crate) const X_DELETEPROPERTY: c_uchar = 19;
pub(crate) const X_GETPROPERTY: c_uchar = 20;
pub(crate) const X_LISTPROPERTIES: c_uchar = 21;
pub(crate) const X_SETSELECTIONOWNER: c_uchar = 22;
pub(crate) const X_GETSELECTIONOWNER: c_uchar = 23;
pub(crate) const X_CONVERTSELECTION: c_uchar = 24;
pub(crate) const X_SENDEVENT: c_uchar = 25;
pub(crate) const X_GRABPOINTER: c_uchar = 26;
pub(crate) const X_UNGRABPOINTER: c_uchar = 27;
pub(crate) const X_GRABBUTTON: c_uchar = 28;
pub(crate) const X_UNGRABBUTTON: c_uchar = 29;
pub(crate) const X_CHANGEACTIVEPOINTERGRAB: c_uchar = 30;
pub(crate) const X_GRABKEYBOARD: c_uchar = 31;
pub(crate) const X_UNGRABKEYBOARD: c_uchar = 32;
pub(crate) const X_GRABKEY: c_uchar = 33;
pub(crate) const X_UNGRABKEY: c_uchar = 34;
pub(crate) const X_ALLOWEVENTS: c_uchar = 35;
pub(crate) const X_GRABSERVER: c_uchar = 36;
pub(crate) const X_UNGRABSERVER: c_uchar = 37;
pub(crate) const X_QUERYPOINTER: c_uchar = 38;
pub(crate) const X_GETMOTIONEVENTS: c_uchar = 39;
pub(crate) const X_TRANSLATECOORDS: c_uchar = 40;
pub(crate) const X_WARPPOINTER: c_uchar = 41;
pub(crate) const X_SETINPUTFOCUS: c_uchar = 42;
pub(crate) const X_GETINPUTFOCUS: c_uchar = 43;
pub(crate) const X_QUERYKEYMAP: c_uchar = 44;
pub(crate) const X_OPENFONT: c_uchar = 45;
pub(crate) const X_CLOSEFONT: c_uchar = 46;
pub(crate) const X_QUERYFONT: c_uchar = 47;
pub(crate) const X_QUERYTEXTEXTENTS: c_uchar = 48;
pub(crate) const X_LISTFONTS: c_uchar = 49;
pub(crate) const X_LISTFONTSWITHINFO: c_uchar = 50;
pub(crate) const X_SETFONTPATH: c_uchar = 51;
pub(crate) const X_GETFONTPATH: c_uchar = 52;
pub(crate) const X_CREATEPIXMAP: c_uchar = 53;
pub(crate) const X_FREEPIXMAP: c_uchar = 54;
pub(crate) const X_CREATEGC: c_uchar = 55;
pub(crate) const X_CHANGEGC: c_uchar = 56;
pub(crate) const X_COPYGC: c_uchar = 57;
pub(crate) const X_SETDASHES: c_uchar = 58;
pub(crate) const X_SETCLIPRECTANGLES: c_uchar = 59;
pub(crate) const X_FREEGC: c_uchar = 60;
pub(crate) const X_CLEARAREA: c_uchar = 61;
pub(crate) const X_COPYAREA: c_uchar = 62;
pub(crate) const X_COPYPLANE: c_uchar = 63;
pub(crate) const X_POLYPOINT: c_uchar = 64;
pub(crate) const X_POLYLINE: c_uchar = 65;
pub(crate) const X_POLYSEGMENT: c_uchar = 66;
pub(crate) const X_POLYRECTANGLE: c_uchar = 67;
pub(crate) const X_POLYARC: c_uchar = 68;
pub(crate) const X_FILLPOLY: c_uchar = 69;
pub(crate) const X_POLYFILLRECTANGLE: c_uchar = 70;
pub(crate) const X_POLYFILLARC: c_uchar = 71;
pub(crate) const X_PUTIMAGE: c_uchar = 72;
pub(crate) const X_GETIMAGE: c_uchar = 73;
pub(crate) const X_POLYTEXT8: c_uchar = 74;
pub(crate) const X_POLYTEXT16: c_uchar = 75;
pub(crate) const X_IMAGETEXT8: c_uchar = 76;
pub(crate) const X_IMAGETEXT16: c_uchar = 77;
pub(crate) const X_CREATECOLORMAP: c_uchar = 78;
pub(crate) const X_FREECOLORMAP: c_uchar = 79;
pub(crate) const X_COPYCOLORMAPANDFREE: c_uchar = 80;
pub(crate) const X_INSTALLCOLORMAP: c_uchar = 81;
pub(crate) const X_UNINSTALLCOLORMAP: c_uchar = 82;
pub(crate) const X_LISTINSTALLEDCOLORMAPS: c_uchar = 83;
pub(crate) const X_ALLOCCOLOR: c_uchar = 84;
pub(crate) const X_ALLOCNAMEDCOLOR: c_uchar = 85;
pub(crate) const X_ALLOCCOLORCELLS: c_uchar = 86;
pub(crate) const X_ALLOCCOLORPLANES: c_uchar = 87;
pub(crate) const X_FREECOLORS: c_uchar = 88;
pub(crate) const X_STORECOLORS: c_uchar = 89;
pub(crate) const X_STORENAMEDCOLOR: c_uchar = 90;
pub(crate) const X_QUERYCOLORS: c_uchar = 91;
pub(crate) const X_LOOKUPCOLOR: c_uchar = 92;
pub(crate) const X_CREATECURSOR: c_uchar = 93;
pub(crate) const X_CREATEGLYPHCURSOR: c_uchar = 94;
pub(crate) const X_FREECURSOR: c_uchar = 95;
pub(crate) const X_RECOLORCURSOR: c_uchar = 96;
pub(crate) const X_QUERYBESTSIZE: c_uchar = 97;
pub(crate) const X_QUERYEXTENSION: c_uchar = 98;
pub(crate) const X_LISTEXTENSIONS: c_uchar = 99;
pub(crate) const X_CHANGEKEYBOARDMAPPING: c_uchar = 100;
pub(crate) const X_GETKEYBOARDMAPPING: c_uchar = 101;
pub(crate) const X_CHANGEKEYBOARDCONTROL: c_uchar = 102;
pub(crate) const X_GETKEYBOARDCONTROL: c_uchar = 103;
pub(crate) const X_BELL: c_uchar = 104;
pub(crate) const X_CHANGEPOINTERCONTROL: c_uchar = 105;
pub(crate) const X_GETPOINTERCONTROL: c_uchar = 106;
pub(crate) const X_SETSCREENSAVER: c_uchar = 107;
pub(crate) const X_GETSCREENSAVER: c_uchar = 108;
pub(crate) const X_CHANGEHOSTS: c_uchar = 109;
pub(crate) const X_LISTHOSTS: c_uchar = 110;
pub(crate) const X_SETACCESSCONTROL: c_uchar = 111;
pub(crate) const X_SETCLOSEDOWNMODE: c_uchar = 112;
pub(crate) const X_KILLCLIENT: c_uchar = 113;
pub(crate) const X_ROTATEPROPERTIES: c_uchar = 114;
pub(crate) const X_FORCESCREENSAVER: c_uchar = 115;
pub(crate) const X_SETPOINTERMAPPING: c_uchar = 116;
pub(crate) const X_GETPOINTERMAPPING: c_uchar = 117;
pub(crate) const X_SETMODIFIERMAPPING: c_uchar = 118;
pub(crate) const X_GETMODIFIERMAPPING: c_uchar = 119;
pub(crate) const X_NOOPERATION: c_uchar = 127;

pub(crate) const XC_FLEUR: c_uint = 52;
pub(crate) const XC_LEFT_PTR: c_uint = 68;
pub(crate) const XC_SIZING: c_uint = 120;

// ATOM values
pub(crate) const XA_ATOM: Atom = 4;
pub(crate) const XA_STRING: Atom = 31;
pub(crate) const XA_WM_NAME: Atom = 39;
pub(crate) const XA_WINDOW: Atom = 33;
pub(crate) const XA_WM_HINTS: Atom = 35;
pub(crate) const XA_WM_NORMAL_HINTS: Atom = 40;
pub(crate) const XA_WM_TRANSIENT_FOR: Atom = 68;

//property Modes
pub(crate) const PROP_MODE_REPLACE: c_int = 0;
pub(crate) const PROP_MODE_PREPEND: c_int = 1;
pub(crate) const PROP_MODE_APPEND: c_int = 2;

/* Key masks. Used as modifiers to GrabButton and GrabKey, results of QueryPointer,
state in various key-, mouse-, and button-related events. */

pub(crate) const SHIFT_MASK: c_uint = 1 << 0;
pub(crate) const LOCK_MASK: c_uint = 1 << 1;
pub(crate) const CONTROL_MASK: c_uint = 1 << 2;
pub(crate) const MOD1_MASK: c_uint = 1 << 3;
pub(crate) const MOD2_MASK: c_uint = 1 << 4;
pub(crate) const MOD3_MASK: c_uint = 1 << 5;
pub(crate) const MOD4_MASK: c_uint = 1 << 6;
pub(crate) const MOD5_MASK: c_uint = 1 << 7;

// KEY Codes
pub(crate) mod keycodes {
    #![allow(non_upper_case_globals)]

    use std::ffi::c_ulong;
    pub(crate) const XK_space: c_ulong = 0x0020; /* U+0020 SPACE */
    pub(crate) const XK_exclam: c_ulong = 0x0021; /* U+0021 EXCLAMATION MARK */
    pub(crate) const XK_quotedbl: c_ulong = 0x0022; /* U+0022 QUOTATION MARK */
    pub(crate) const XK_numbersign: c_ulong = 0x0023; /* U+0023 NUMBER SIGN */
    pub(crate) const XK_dollar: c_ulong = 0x0024; /* U+0024 DOLLAR SIGN */
    pub(crate) const XK_percent: c_ulong = 0x0025; /* U+0025 PERCENT SIGN */
    pub(crate) const XK_ampersand: c_ulong = 0x0026; /* U+0026 AMPERSAND */
    pub(crate) const XK_apostrophe: c_ulong = 0x0027; /* U+0027 APOSTROPHE */
    pub(crate) const XK_quoteright: c_ulong = 0x0027; /* deprecated */
    pub(crate) const XK_parenleft: c_ulong = 0x0028; /* U+0028 LEFT PARENTHESIS */
    pub(crate) const XK_parenright: c_ulong = 0x0029; /* U+0029 RIGHT PARENTHESIS */
    pub(crate) const XK_asterisk: c_ulong = 0x002a; /* U+002A ASTERISK */
    pub(crate) const XK_plus: c_ulong = 0x002b; /* U+002B PLUS SIGN */
    pub(crate) const XK_comma: c_ulong = 0x002c; /* U+002C COMMA */
    pub(crate) const XK_minus: c_ulong = 0x002d; /* U+002D HYPHEN-MINUS */
    pub(crate) const XK_period: c_ulong = 0x002e; /* U+002E FULL STOP */
    pub(crate) const XK_slash: c_ulong = 0x002f; /* U+002F SOLIDUS */
    pub(crate) const XK_0: c_ulong = 0x0030; /* U+0030 DIGIT ZERO */
    pub(crate) const XK_1: c_ulong = 0x0031; /* U+0031 DIGIT ONE */
    pub(crate) const XK_2: c_ulong = 0x0032; /* U+0032 DIGIT TWO */
    pub(crate) const XK_3: c_ulong = 0x0033; /* U+0033 DIGIT THREE */
    pub(crate) const XK_4: c_ulong = 0x0034; /* U+0034 DIGIT FOUR */
    pub(crate) const XK_5: c_ulong = 0x0035; /* U+0035 DIGIT FIVE */
    pub(crate) const XK_6: c_ulong = 0x0036; /* U+0036 DIGIT SIX */
    pub(crate) const XK_7: c_ulong = 0x0037; /* U+0037 DIGIT SEVEN */
    pub(crate) const XK_8: c_ulong = 0x0038; /* U+0038 DIGIT EIGHT */
    pub(crate) const XK_9: c_ulong = 0x0039; /* U+0039 DIGIT NINE */
    pub(crate) const XK_colon: c_ulong = 0x003a; /* U+003A COLON */
    pub(crate) const XK_semicolon: c_ulong = 0x003b; /* U+003B SEMICOLON */
    pub(crate) const XK_less: c_ulong = 0x003c; /* U+003C LESS-THAN SIGN */
    pub(crate) const XK_equal: c_ulong = 0x003d; /* U+003D EQUALS SIGN */
    pub(crate) const XK_greater: c_ulong = 0x003e; /* U+003E GREATER-THAN SIGN */
    pub(crate) const XK_question: c_ulong = 0x003f; /* U+003F QUESTION MARK */
    pub(crate) const XK_at: c_ulong = 0x0040; /* U+0040 COMMERCIAL AT */
    pub(crate) const XK_A: c_ulong = 0x0041; /* U+0041 LATIN CAPITAL LETTER A */
    pub(crate) const XK_B: c_ulong = 0x0042; /* U+0042 LATIN CAPITAL LETTER B */
    pub(crate) const XK_C: c_ulong = 0x0043; /* U+0043 LATIN CAPITAL LETTER C */
    pub(crate) const XK_D: c_ulong = 0x0044; /* U+0044 LATIN CAPITAL LETTER D */
    pub(crate) const XK_E: c_ulong = 0x0045; /* U+0045 LATIN CAPITAL LETTER E */
    pub(crate) const XK_F: c_ulong = 0x0046; /* U+0046 LATIN CAPITAL LETTER F */
    pub(crate) const XK_G: c_ulong = 0x0047; /* U+0047 LATIN CAPITAL LETTER G */
    pub(crate) const XK_H: c_ulong = 0x0048; /* U+0048 LATIN CAPITAL LETTER H */
    pub(crate) const XK_I: c_ulong = 0x0049; /* U+0049 LATIN CAPITAL LETTER I */
    pub(crate) const XK_J: c_ulong = 0x004a; /* U+004A LATIN CAPITAL LETTER J */
    pub(crate) const XK_K: c_ulong = 0x004b; /* U+004B LATIN CAPITAL LETTER K */
    pub(crate) const XK_L: c_ulong = 0x004c; /* U+004C LATIN CAPITAL LETTER L */
    pub(crate) const XK_M: c_ulong = 0x004d; /* U+004D LATIN CAPITAL LETTER M */
    pub(crate) const XK_N: c_ulong = 0x004e; /* U+004E LATIN CAPITAL LETTER N */
    pub(crate) const XK_O: c_ulong = 0x004f; /* U+004F LATIN CAPITAL LETTER O */
    pub(crate) const XK_P: c_ulong = 0x0050; /* U+0050 LATIN CAPITAL LETTER P */
    pub(crate) const XK_Q: c_ulong = 0x0051; /* U+0051 LATIN CAPITAL LETTER Q */
    pub(crate) const XK_R: c_ulong = 0x0052; /* U+0052 LATIN CAPITAL LETTER R */
    pub(crate) const XK_S: c_ulong = 0x0053; /* U+0053 LATIN CAPITAL LETTER S */
    pub(crate) const XK_T: c_ulong = 0x0054; /* U+0054 LATIN CAPITAL LETTER T */
    pub(crate) const XK_U: c_ulong = 0x0055; /* U+0055 LATIN CAPITAL LETTER U */
    pub(crate) const XK_V: c_ulong = 0x0056; /* U+0056 LATIN CAPITAL LETTER V */
    pub(crate) const XK_W: c_ulong = 0x0057; /* U+0057 LATIN CAPITAL LETTER W */
    pub(crate) const XK_X: c_ulong = 0x0058; /* U+0058 LATIN CAPITAL LETTER X */
    pub(crate) const XK_Y: c_ulong = 0x0059; /* U+0059 LATIN CAPITAL LETTER Y */
    pub(crate) const XK_Z: c_ulong = 0x005a; /* U+005A LATIN CAPITAL LETTER Z */
    pub(crate) const XK_bracketleft: c_ulong = 0x005b; /* U+005B LEFT SQUARE BRACKET */
    pub(crate) const XK_backslash: c_ulong = 0x005c; /* U+005C REVERSE SOLIDUS */
    pub(crate) const XK_bracketright: c_ulong = 0x005d; /* U+005D RIGHT SQUARE BRACKET */
    pub(crate) const XK_asciicircum: c_ulong = 0x005e; /* U+005E CIRCUMFLEX ACCENT */
    pub(crate) const XK_underscore: c_ulong = 0x005f; /* U+005F LOW LINE */
    pub(crate) const XK_grave: c_ulong = 0x0060; /* U+0060 GRAVE ACCENT */
    pub(crate) const XK_quoteleft: c_ulong = 0x0060; /* deprecated */
    pub(crate) const XK_a: c_ulong = 0x0061; /* U+0061 LATIN SMALL LETTER A */
    pub(crate) const XK_b: c_ulong = 0x0062; /* U+0062 LATIN SMALL LETTER B */
    pub(crate) const XK_c: c_ulong = 0x0063; /* U+0063 LATIN SMALL LETTER C */
    pub(crate) const XK_d: c_ulong = 0x0064; /* U+0064 LATIN SMALL LETTER D */
    pub(crate) const XK_e: c_ulong = 0x0065; /* U+0065 LATIN SMALL LETTER E */
    pub(crate) const XK_f: c_ulong = 0x0066; /* U+0066 LATIN SMALL LETTER F */
    pub(crate) const XK_g: c_ulong = 0x0067; /* U+0067 LATIN SMALL LETTER G */
    pub(crate) const XK_h: c_ulong = 0x0068; /* U+0068 LATIN SMALL LETTER H */
    pub(crate) const XK_i: c_ulong = 0x0069; /* U+0069 LATIN SMALL LETTER I */
    pub(crate) const XK_j: c_ulong = 0x006a; /* U+006A LATIN SMALL LETTER J */
    pub(crate) const XK_k: c_ulong = 0x006b; /* U+006B LATIN SMALL LETTER K */
    pub(crate) const XK_l: c_ulong = 0x006c; /* U+006C LATIN SMALL LETTER L */
    pub(crate) const XK_m: c_ulong = 0x006d; /* U+006D LATIN SMALL LETTER M */
    pub(crate) const XK_n: c_ulong = 0x006e; /* U+006E LATIN SMALL LETTER N */
    pub(crate) const XK_o: c_ulong = 0x006f; /* U+006F LATIN SMALL LETTER O */
    pub(crate) const XK_p: c_ulong = 0x0070; /* U+0070 LATIN SMALL LETTER P */
    pub(crate) const XK_q: c_ulong = 0x0071; /* U+0071 LATIN SMALL LETTER Q */
    pub(crate) const XK_r: c_ulong = 0x0072; /* U+0072 LATIN SMALL LETTER R */
    pub(crate) const XK_s: c_ulong = 0x0073; /* U+0073 LATIN SMALL LETTER S */
    pub(crate) const XK_t: c_ulong = 0x0074; /* U+0074 LATIN SMALL LETTER T */
    pub(crate) const XK_u: c_ulong = 0x0075; /* U+0075 LATIN SMALL LETTER U */
    pub(crate) const XK_v: c_ulong = 0x0076; /* U+0076 LATIN SMALL LETTER V */
    pub(crate) const XK_w: c_ulong = 0x0077; /* U+0077 LATIN SMALL LETTER W */
    pub(crate) const XK_x: c_ulong = 0x0078; /* U+0078 LATIN SMALL LETTER X */
    pub(crate) const XK_y: c_ulong = 0x0079; /* U+0079 LATIN SMALL LETTER Y */
    pub(crate) const XK_z: c_ulong = 0x007a; /* U+007A LATIN SMALL LETTER Z */
    pub(crate) const XK_braceleft: c_ulong = 0x007b; /* U+007B LEFT CURLY BRACKET */
    pub(crate) const XK_bar: c_ulong = 0x007c; /* U+007C VERTICAL LINE */
    pub(crate) const XK_braceright: c_ulong = 0x007d; /* U+007D RIGHT CURLY BRACKET */
    pub(crate) const XK_asciitilde: c_ulong = 0x007e; /* U+007E TILDE */

    pub(crate) const XK_BackSpace: c_ulong = 0xff08; /* U+0008 BACKSPACE */
    pub(crate) const XK_Tab: c_ulong = 0xff09; /* U+0009 CHARACTER TABULATION */
    pub(crate) const XK_Linefeed: c_ulong = 0xff0a; /* U+000A LINE FEED */
    pub(crate) const XK_Clear: c_ulong = 0xff0b; /* U+000B LINE TABULATION */
    pub(crate) const XK_Return: c_ulong = 0xff0d; /* U+000D CARRIAGE RETURN */
    pub(crate) const XK_Pause: c_ulong = 0xff13; /* Pause, hold */
    pub(crate) const XK_Scroll_Lock: c_ulong = 0xff14;
    pub(crate) const XK_Sys_Req: c_ulong = 0xff15;
    pub(crate) const XK_Escape: c_ulong = 0xff1b; /* U+001B ESCAPE */
    pub(crate) const XK_Delete: c_ulong = 0xffff; /* U+007F DELETE */

    pub(crate) const XK_F1: c_ulong = 0xffbe;
    pub(crate) const XK_F2: c_ulong = 0xffbf;
    pub(crate) const XK_F3: c_ulong = 0xffc0;
    pub(crate) const XK_F4: c_ulong = 0xffc1;
    pub(crate) const XK_F5: c_ulong = 0xffc2;
    pub(crate) const XK_F6: c_ulong = 0xffc3;
    pub(crate) const XK_F7: c_ulong = 0xffc4;
    pub(crate) const XK_F8: c_ulong = 0xffc5;
    pub(crate) const XK_F9: c_ulong = 0xffc6;
    pub(crate) const XK_F10: c_ulong = 0xffc7;
    pub(crate) const XK_F11: c_ulong = 0xffc8;
    pub(crate) const XK_F12: c_ulong = 0xffc9;

    pub(crate) const XF86XK_Standby: c_ulong = 0x1008ff10; /* System into standby mode   */
    pub(crate) const XF86XK_AudioLowerVolume: c_ulong = 0x1008ff11; /* Volume control down        */
    pub(crate) const XF86XK_AudioMute: c_ulong = 0x1008ff12; /* Mute sound from the system */
    pub(crate) const XF86XK_AudioRaiseVolume: c_ulong = 0x1008ff13; /* Volume control up          */
    pub(crate) const XF86XK_AudioPlay: c_ulong = 0x1008ff14; /* Start playing of audio >   */
    pub(crate) const XF86XK_AudioStop: c_ulong = 0x1008ff15; /* Stop playing audio         */
    pub(crate) const XF86XK_AudioPrev: c_ulong = 0x1008ff16; /* Previous track             */
    pub(crate) const XF86XK_AudioNext: c_ulong = 0x1008ff17; /* Next track                 */
    pub(crate) const XF86XK_HomePage: c_ulong = 0x1008ff18; /* Display user's home page   */
    pub(crate) const XF86XK_Mail: c_ulong = 0x1008ff19; /* Invoke user's mail program */
    pub(crate) const XF86XK_Start: c_ulong = 0x1008ff1a; /* Start application          */
    pub(crate) const XF86XK_Search: c_ulong = 0x1008ff1b; /* Search                     */
    pub(crate) const XF86XK_AudioRecord: c_ulong = 0x1008ff1c; /* Record audio application   */
    pub(crate) const XF86XK_AudioPause: c_ulong = 0x1008ff31; /* Pause audio playing        */
    pub(crate) const XF86XK_AudioRewind: c_ulong = 0x1008ff3e; /* "rewind" audio track        */
    pub(crate) const XF86XK_AudioForward: c_ulong = 0x1008ff97; /* fast-forward audio track    */
    pub(crate) const XF86XK_AudioMedia: c_ulong = 0x1008ff32; /* Launch media collection app */
    pub(crate) const XF86XK_AudioMicMute: c_ulong = 0x1008ffb2; /* Mute the Mic from the system */
    pub(crate) const XF86XK_Calculator: c_ulong = 0x1008ff1d; /* Invoke calculator program  */
    pub(crate) const XF86XK_Sleep: c_ulong = 0x1008ff2f; /* Put system to sleep        */
    pub(crate) const XF86XK_WWW: c_ulong = 0x1008ff2e; /* Invoke web browser         */
    pub(crate) const XF86XK_DOS: c_ulong = 0x1008ff5a; /* Launch DOS (emulation)      */
    pub(crate) const XF86XK_ScreenSaver: c_ulong = 0x1008ff2d; /* Invoke screensaver         */
    pub(crate) const XF86XK_TaskPane: c_ulong = 0x1008ff7f; /* Show tasks */
    pub(crate) const XF86XK_MyComputer: c_ulong = 0x1008ff33; /* Display "My Computer" window */
    pub(crate) const XF86XK_Battery: c_ulong = 0x1008ff93; /* Display battery information */
    pub(crate) const XF86XK_Launch1: c_ulong = 0x1008ff41; /* Launch Application          */
    pub(crate) const XF86XK_TouchpadToggle: c_ulong = 0x1008ffa9; /* Toggle between touchpad/trackstick */
    pub(crate) const XF86XK_TouchpadOn: c_ulong = 0x1008ffb0; /* The touchpad got switched on */
    pub(crate) const XF86XK_TouchpadOff: c_ulong = 0x1008ffb1; /* The touchpad got switched off */
    pub(crate) const XF86XK_MonBrightnessUp: c_ulong = 0x1008ff02; /* Monitor/panel brightness */
    pub(crate) const XF86XK_MonBrightnessDown: c_ulong = 0x1008ff03; /* Monitor/panel brightness */
}

//button names:
pub(crate) const BUTTON1: c_uint = 1;
pub(crate) const BUTTON2: c_uint = 2;
pub(crate) const BUTTON3: c_uint = 3;
pub(crate) const BUTTON4: c_uint = 4;
pub(crate) const BUTTON5: c_uint = 5;

// ConfigureWindow Structure
pub(crate) const CWX: c_uint = 1 << 0;
pub(crate) const CWY: c_uint = 1 << 1;
pub(crate) const CW_WIDTH: c_uint = 1 << 2;
pub(crate) const CW_HEIGHT: c_uint = 1 << 3;
pub(crate) const CW_BORDER_WIDTH: c_uint = 1 << 4;
pub(crate) const CW_SIBLING: c_uint = 1 << 5;
pub(crate) const CW_STACK_MODE: c_uint = 1 << 6;

/* definitions for initial window state */
pub(crate) const WITHDRAWN_STATE: c_int = 0; /* for windows that are not mapped */
pub(crate) const NORMAL_STATE: c_int = 1; /* most applications want to start this way */
pub(crate) const ICONIC_STATE: c_int = 3; /* application wants to start as an icon */

pub(crate) const ANY_BUTTON: c_uint = 0;
pub(crate) const ANY_MODIFIER: c_uint = 1 << 15;
pub(crate) const GRAB_MODE_SYNC: c_int = 0;
pub(crate) const GRAB_MODE_ASYNC: c_int = 1;
