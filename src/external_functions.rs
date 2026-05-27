#![allow(dead_code)]

use std::ffi::{c_char, c_int, c_long, c_ulong, c_uint, c_void};

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
        width: u32,
        height: u32,
        depth: u32,
    ) -> Pixmap;

    pub(crate) fn XSetLineAttributes(
        display: *mut Display,
        gc: GC,
        line_width: u32,
        line_style: i32,
        cap_style: i32,
        join_style: i32,
    ) -> i32;

    pub(crate) fn XFreePixmap(display: *mut Display, pixmap: Pixmap) -> i32;

    pub(crate) fn XFreeGC(display: *mut Display, gc: GC) -> i32;

    pub(crate) fn XSetForeground(display: *mut Display, gc: GC, foreground: c_ulong) -> c_int;
    pub(crate) fn XFillRectangle(
        display: *mut Display,
        d: Drawable,
        gc: GC,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
    ) -> i32;

    pub(crate) fn XDrawRectangle(
        display: *mut Display,
        d: Drawable,
        gc: GC,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
    ) -> i32;

    pub(crate) fn XCopyArea(
        display: *mut Display,
        src: Drawable,
        dest: Drawable,
        gc: GC,
        src_x: i32,
        src_y: i32,
        width: u32,
        height: u32,
        dest_x: i32,
        dest_y: i32,
    ) -> i32;

    pub(crate) fn XSync(display: *mut Display, discard: i32) -> i32;

    pub(crate) fn XCreateFontCursor(display: *mut Display, shape: u32) -> Cursor;
    pub(crate) fn XFreeCursor(display: *mut Display, cursor: Cursor) -> i32;

    pub(crate) fn XSupportsLocale() -> i32;

    pub(crate) fn XOpenDisplay(display_name: *const c_char) -> *mut Display;

    pub(crate) fn XSetErrorHandler(handler: XErrorHandler) -> XErrorHandler;

    pub(crate) fn XSelectInput(display: *mut Display, w: Window, event_mask: c_long) -> c_int;

    pub(crate) fn XQueryPointer(
        display: *mut Display,
        w: Window,
        root_return: *mut Window,
        child_return: *mut Window,
        root_x_return: *mut i32,
        root_y_return: *mut i32,
        win_x_return: *mut i32,
        win_y_return: *mut i32,
        mask_return: *mut u32,
    ) -> i32;

    pub(crate) fn XInternAtom(
        display: *mut Display,
        atom_name: *const i8,
        only_if_exists: i32,
    ) -> Atom;

    pub(crate) fn XCreateWindow(
        display: *mut Display,
        parent: Window,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        border_width: u32,
        depth: i32,
        class: u32,
        visual: *mut Visual,
        value_mask: c_ulong,
        attributes: *mut XSetWindowAttributes,
    ) -> Window;

    pub(crate) fn XDefineCursor(display: *mut Display, w: Window, cursor: Cursor) -> i32;
    pub(crate) fn XMapRaised(display: *mut Display, w: Window) -> i32;
    pub(crate) fn XSetClassHint(
        display: *mut Display,
        w: Window,
        class_hints: *mut XClassHint,
    ) -> i32;

    pub(crate) fn XGetTextProperty(
        display: *mut Display,
        window: Window,
        text_prop_return: *mut XTextProperty,
        property: Atom,
    ) -> Status;

    pub(crate) fn XmbTextPropertyToTextList(
        display: *mut Display,
        text_prop: *const XTextProperty,
        list_return: *mut *mut *mut i8,
        count_return: *mut i32,
    ) -> i32;

    pub(crate) fn XFreeStringList(list: *mut *mut i8);
    pub(crate) fn XFree(data: *mut c_void);

    pub(crate) fn XCreateSimpleWindow(
        display: *mut Display,
        parent: Window,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        border_width: c_uint,
        border: c_ulong,
        background: c_ulong,
    ) -> Window;

    pub(crate) fn XChangeProperty(
        display: *mut Display,
        w: Window,
        property: Atom,
        r#type: Atom,
        format: i32,
        mode: i32,
        data: *const u8,
        nelements: i32,
    ) -> i32;

    pub(crate) fn XDeleteProperty(display: *mut Display, w: Window, property: Atom) -> i32;

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
        keycode: i32,
        modifiers: u32,
        grab_window: Window,
    ) -> i32;

    pub(crate) fn XDisplayKeycodes(
        display: *mut Display,
        min_keycodes_return: *mut i32,
        max_keycodes_return: *mut i32,
    ) -> i32;

    pub(crate) fn XGetKeyboardMapping(
        display: *mut Display,
        first_keycode: KeyCode,
        keycode_count: i32,
        keysyms_per_keycode_return: *mut i32,
    ) -> *mut KeySym;

    pub(crate) fn XGrabKey(
        display: *mut Display,
        keycode: i32,
        modifiers: u32,
        grab_window: Window,
        owner_events: i32,
        pointer_mode: i32,
        keyboard_mode: i32,
    ) -> i32;

    pub(crate) fn XUngrabButton(
        display: *mut Display,
        button: u32,
        modifiers: u32,
        grab_window: Window,
    ) -> i32;

    pub(crate) fn XGrabButton(
        display: *mut Display,
        button: u32,
        modifiers: u32,
        grab_window: Window,
        owner_events: i32,
        event_mask: u32,
        pointer_mode: i32,
        keyboard_mode: i32,
        confine_to: Window,
        cursor: Cursor,
    ) -> i32;

    pub(crate) fn XSetWindowBorder(display: *mut Display, w: Window, border_pixel: c_ulong) -> c_int;
    pub(crate) fn XSetInputFocus(
        display: *mut Display,
        focus: Window,
        revert_to: i32,
        time: Time,
    ) -> i32;

    pub(crate) fn XGetWMProtocols(
        display: *mut Display,
        w: Window,
        protocols_return: *mut *mut Atom,
        count_return: *mut i32,
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
        nchildren_return: *mut u32,
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
        prop_return: *mut *mut u8,
    ) -> c_int;

    pub(crate) fn XGetClassHint(
        display: *mut Display,
        w: Window,
        class_hint_return: *mut XClassHint,
    ) -> Status;

    pub(crate) fn XConfigureWindow(
        display: *mut Display,
        w: Window,
        value_mask: u32,
        values: *mut XWindowChanges,
    ) -> i32;

    pub(crate) fn XRaiseWindow(display: *mut Display, w: Window) -> i32;

    pub(crate) fn XCheckMaskEvent(
        display: *mut Display,
        event_mask: c_long,
        event_return: *mut XEvent,
    ) -> c_int;

    pub(crate) fn XMoveWindow(display: *mut Display, w: Window, x: i32, y: i32) -> i32;

    pub(crate) fn XGetWMNormalHints(
        display: *mut Display,
        w: Window,
        hints_return: *mut XSizeHints,
        supplied_return: *mut c_long,
    ) -> Status;

    pub(crate) fn XSetWMHints(display: *mut Display, w: Window, wm_hints: *mut XWMHints) -> i32;
    pub(crate) fn XMoveResizeWindow(
        display: *mut Display,
        w: Window,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> i32;

    pub(crate) fn XMapWindow(display: *mut Display, w: Window) -> i32;
    pub(crate) fn XCloseDisplay(display: *mut Display) -> i32;

    pub(crate) fn XNextEvent(display: *mut Display, event_return: *mut XEvent) -> i32;

    pub(crate) fn XAllowEvents(display: *mut Display, event_mode: i32, time: Time) -> i32;
    pub(crate) fn XKeycodeToKeysym(display: *mut Display, keycode: KeyCode, index: i32) -> KeySym;
    pub(crate) fn XRefreshKeyboardMapping(event_map: *mut XMappingEvent) -> i32;
    pub(crate) fn XSetCloseDownMode(display: *mut Display, close_mode: i32) -> i32;
    pub(crate) fn XKillClient(display: *mut Display, resource: XID) -> i32;
    pub(crate) fn XGrabPointer(
        display: *mut Display,
        grab_window: Window,
        owner_events: i32,
        event_mask: u32,
        pointer_mode: i32,
        keyboard_mode: i32,
        confine_to: Window,
        cursor: Cursor,
        time: Time,
    ) -> i32;

    pub(crate) fn XMaskEvent(
        display: *mut Display,
        event_mask: c_long,
        event_return: *mut XEvent,
    ) -> c_int;

    pub(crate) fn XUngrabPointer(display: *mut Display, time: Time) -> i32;
    pub(crate) fn XWarpPointer(
        display: *mut Display,
        src_w: Window,
        dest_w: Window,
        src_x: i32,
        src_y: i32,
        src_width: u32,
        src_height: u32,
        dest_x: i32,
        dest_y: i32,
    ) -> i32;

    pub(crate) fn XGetWMHints(display: *mut Display, w: Window) -> *mut XWMHints;
    pub(crate) fn XGrabServer(display: *mut Display) -> c_int;
    pub(crate) fn XUngrabServer(display: *mut Display) -> c_int;
    pub(crate) fn XDestroyWindow(display: *mut Display, w: Window) -> c_int;
    pub(crate) fn XUnmapWindow(display: *mut Display, w: Window) -> c_int;
}

#[link(name = "Xinerama")]
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
    pub(crate) fn XftFontOpenName(dpy: *mut Display, screen: c_int, name: *const c_char) -> *mut XftFont;
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

pub(crate) type FcChar8 = u8;
pub(crate) type FcChar32 = u32;

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
    pub(crate) ascent: i32,
    pub(crate) descent: i32,
    pub(crate) height: i32,
    pub(crate) max_advance_width: i32,
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
    red: u16,
    green: u16,
    blue: u16,
    alpha: u16,
}

pub(crate) type GC = *mut _XGC;

pub(crate) enum _XGC {}
pub(crate) enum _XPrivate {}
pub(crate) enum _XrmHashBucketRec {}

#[repr(C)]
struct _XPrivDisplay {
    pub(crate) ext_data: *mut XExtData,
    pub(crate) private1: *mut _XPrivate,
    pub(crate) fd: i32,
    pub(crate) private2: i32,
    pub(crate) proto_major_version: i32, /* major version of server's X protocol */
    pub(crate) proto_minor_version: i32, /* minor version of servers X protocol */
    pub(crate) vendor: *mut c_char,       /* vendor of the server hardware */
    pub(crate) private3: XID,
    pub(crate) private4: XID,
    pub(crate) private5: XID,
    pub(crate) private6: i32,
    pub(crate) resource_alloc: fn(display: *mut _XDisplay) -> XID, /* allocator function */
    pub(crate) byte_order: i32, /* screen byte order, LSBFirst, MSBFirst */
    pub(crate) bitmap_unit: i32, /* padding and data requirements */
    pub(crate) bitmap_pad: i32, /* padding requirements on bitmaps */
    pub(crate) bitmap_bit_order: i32, /* LeastSignificant or MostSignificant */
    pub(crate) nformats: i32,   /* LeastSignificant or MostSignificant */
    pub(crate) pixmap_format: *mut ScreenFormat, /* pixmap format list */
    pub(crate) private8: i32,
    pub(crate) release: i32, /* release of the server */
    pub(crate) private9: *mut _XPrivate,
    pub(crate) private10: *mut _XPrivate,
    pub(crate) qlen: i32,              /* Length of input event queue */
    pub(crate) last_request_read: c_ulong, /* seq number of last event read */
    pub(crate) request: c_ulong,           /* sequence number of last request. */
    pub(crate) private11: XPointer,
    pub(crate) private12: XPointer,
    pub(crate) private13: XPointer,
    pub(crate) private14: XPointer,
    pub(crate) max_request_size: u32, /* maximum number 32 bit words in request*/
    pub(crate) db: *mut _XrmHashBucketRec,
    pub(crate) private15: fn(display: *mut _XDisplay) -> i32,
    pub(crate) display_name: *mut c_char, /* "host:display" string used on this connect*/
    pub(crate) default_screen: i32,   /* default screen for operations */
    pub(crate) nscreens: i32,         /* number of screens on this server*/
    pub(crate) screens: *mut Screen,  /* pointer to list of screens */
    pub(crate) motion_buffer: c_ulong, /* size of motion buffer */
    pub(crate) private16: c_ulong,
    pub(crate) min_keycode: i32, /* minimum defined keycode */
    pub(crate) max_keycode: i32, /* maximum defined keycode */
    pub(crate) private17: XPointer,
    pub(crate) private18: XPointer,
    pub(crate) private19: i32,
    pub(crate) xdefaults: *mut c_char, /* contents of defaults from server */
}

#[repr(C)]
pub(crate) struct XExtData {
    number: i32,
    next: *mut XExtData,
    free_private: extern "C" fn(extension: *mut XExtData),
    private_data: XPointer,
}

#[repr(C)]
pub(crate) struct XModifierKeymap {
    pub(crate) max_keypermod: i32,
    pub(crate) modifiermap: *mut KeyCode,
}

pub(crate) type KeyCode = u8;

#[repr(C)]
pub(crate) union XEvent {
    pub(crate) r#type: i32,
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
    pub(crate) r#type: i32,
    pub(crate) serial: u64,
    pub(crate) send_event: i32,
    pub(crate) display: *mut Display,
    pub(crate) window: Window,
    pub(crate) message_type: Atom,
    pub(crate) format: i32,
    pub(crate) data: XClientMessageEventData,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) union XClientMessageEventData {
    pub(crate) b: [i8; 20],
    pub(crate) s: [i16; 10],
    pub(crate) l: [c_long; 5],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XButtonEvent {
    pub(crate) r#type: i32,           /* of event */
    pub(crate) serial: c_ulong,        /* # of last request processed by server */
    pub(crate) send_event: i32,       /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) window: Window,        /* "event" window it is reported relative to */
    pub(crate) root: Window,          /* root window that the event occurred on */
    pub(crate) subwindow: Window,     /* child window */
    pub(crate) time: Time,            /* milliseconds */
    pub(crate) x: i32,                /* pointer x, y coordinates in event window */
    pub(crate) y: i32,
    pub(crate) x_root: i32, /* coordinates relative to root */
    pub(crate) y_root: i32,
    pub(crate) state: u32,       /* key or button mask */
    pub(crate) button: u32,      /* detail */
    pub(crate) same_screen: i32, /* same screen flag */
}

pub(crate) type XButtonPressedEvent = XButtonEvent;
pub(crate) type XButtonReleasedEvent = XButtonEvent;

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XConfigureRequestEvent {
    pub(crate) r#type: i32,
    pub(crate) serial: c_ulong, /* # of last request processed by server */
    pub(crate) send_event: i32, /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) parent: Window,
    pub(crate) window: Window,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) border_width: i32,
    pub(crate) above: Window,
    pub(crate) detail: i32, /* Above, Below, TopIf, BottomIf, Opposite */
    pub(crate) value_mask: c_ulong,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XDestroyWindowEvent {
    pub(crate) r#type: i32,
    pub(crate) serial: c_ulong, /* # of last request processed by server */
    pub(crate) send_event: i32, /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) event: Window,
    pub(crate) window: Window,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XCrossingEvent {
    pub(crate) r#type: i32,           /* of event */
    pub(crate) serial: c_ulong,        /* # of last request processed by server */
    pub(crate) send_event: i32,       /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) window: Window,        /* "event" window reported relative to */
    pub(crate) root: Window,          /* root window that the event occurred on */
    pub(crate) subwindow: Window,     /* child window */
    pub(crate) time: Time,            /* milliseconds */
    pub(crate) x: i32,                /* pointer x, y coordinates in event window */
    pub(crate) y: i32,
    pub(crate) x_root: i32, /* coordinates relative to root */
    pub(crate) y_root: i32,
    pub(crate) mode: i32, /* NotifyNormal, NotifyGrab, NotifyUngrab */
    pub(crate) detail: i32, /*
                           * NotifyAncestor, NotifyVirtual, NotifyInferior,
                           * NotifyNonlinear,NotifyNonlinearVirtual
                           */
    pub(crate) same_screen: i32, /* same screen flag */
    pub(crate) focus: i32,       /* boolean focus */
    pub(crate) state: u32,       /* key or button mask */
}
type XEnterWindowEvent = XCrossingEvent;
type XLeaveWindowEvent = XCrossingEvent;

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XExposeEvent {
    pub(crate) r#type: i32,
    pub(crate) serial: c_ulong, /* # of last request processed by server */
    pub(crate) send_event: i32, /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) window: Window,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) count: i32, /* if non-zero, at least this many more */
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XFocusChangeEvent {
    pub(crate) r#type: i32,           /* FocusIn or FocusOut */
    pub(crate) serial: c_ulong,        /* # of last request processed by server */
    pub(crate) send_event: i32,       /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) window: Window,        /* window of event */
    pub(crate) mode: i32,             /* NotifyNormal, NotifyWhileGrabbed,
                                      NotifyGrab, NotifyUngrab */
    pub(crate) detail: i32, /*
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
    pub(crate) r#type: i32,           /* of event */
    pub(crate) serial: c_ulong,        /* # of last request processed by server */
    pub(crate) send_event: i32,       /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) window: Window,        /* "event" window it is reported relative to */
    pub(crate) root: Window,          /* root window that the event occurred on */
    pub(crate) subwindow: Window,     /* child window */
    pub(crate) time: Time,            /* milliseconds */
    pub(crate) x: i32,                /* pointer x, y coordinates in event window */
    pub(crate) y: i32,
    pub(crate) x_root: i32, /* coordinates relative to root */
    pub(crate) y_root: i32,
    pub(crate) state: u32,       /* key or button mask */
    pub(crate) keycode: u32,     /* detail */
    pub(crate) same_screen: i32, /* same screen flag */
}

type XKeyPressedEvent = XKeyEvent;
type XKeyReleasedEvent = XKeyEvent;

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XMappingEvent {
    pub(crate) r#type: i32,           /* of event */
    pub(crate) serial: c_ulong,        /* # of last request processed by server */
    pub(crate) send_event: i32,       /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) window: Window,        /* unused */
    pub(crate) request: i32,          /* one of MappingModifier, MappingKeyboard,
                                      MappingPointer */
    pub(crate) first_keycode: i32, /* first keycode */
    pub(crate) count: i32,         /* defines range of change w. first_keycode*/
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XMapRequestEvent {
    pub(crate) r#type: i32,
    pub(crate) serial: c_ulong, /* # of last request processed by server */
    pub(crate) send_event: i32, /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) parent: Window,
    pub(crate) window: Window,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XMotionEvent {
    pub(crate) r#type: i32,
    pub(crate) serial: c_ulong, /* # of last request processed by server */
    pub(crate) send_event: i32, /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) window: Window,
    pub(crate) root: Window,
    pub(crate) subwindow: Window,
    pub(crate) time: Time,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) x_root: i32,
    pub(crate) y_root: i32,
    pub(crate) state: u32,
    pub(crate) is_hint: i8,
    pub(crate) same_screen: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XUnmapEvent {
    pub(crate) r#type: i32,
    pub(crate) serial: c_ulong, /* # of last request processed by server */
    pub(crate) send_event: i32, /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) event: Window,
    pub(crate) window: Window,
    pub(crate) from_configure: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XPropertyEvent {
    pub(crate) r#type: i32,
    pub(crate) serial: c_ulong, /* # of last request processed by server */
    pub(crate) send_event: i32, /* true if this came from a SendEvent request */
    pub(crate) display: *mut Display, /* Display the event was read from */
    pub(crate) window: Window,
    pub(crate) atom: Atom,
    pub(crate) time: Time,
    pub(crate) state: i32, /* NewValue, Deleted */
}

#[repr(C)]
pub(crate) struct XWindowChanges {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) border_width: i32,
    pub(crate) sibling: Window,
    pub(crate) stack_mode: i32,
}

pub(crate) type XPointer = *mut c_char;
pub(crate) type Atom = c_ulong;

#[repr(C)]
struct ScreenFormat {
    ext_data: *mut XExtData,
    depth: i32,
    bits_per_pixel: i32,
    scanline_pad: i32,
}

#[repr(C)]
pub(crate) struct Screen {
    pub(crate) ext_data: *mut XExtData, /* hook for extension to hang data */
    pub(crate) display: *mut Display,   /* back pointer to display structure */
    pub(crate) root: Window,            /* Root window id. */
    pub(crate) width: i32,              /* width and height of screen */
    pub(crate) height: i32,
    pub(crate) mwidth: i32, /* width and height of  in millimeters */
    pub(crate) mheight: i32,
    pub(crate) ndepths: i32,             /* number of depths possible */
    pub(crate) depths: *mut Depth,       /* list of allowable depths on the screen */
    pub(crate) root_depth: i32,          /* bits per pixel */
    pub(crate) root_visual: *mut Visual, /* root visual */
    pub(crate) default_gc: GC,           /* GC for the root root visual */
    pub(crate) cmap: Colormap,           /* default color map */
    pub(crate) white_pixel: c_ulong,
    pub(crate) black_pixel: c_ulong, /* White and Black pixel values */
    pub(crate) max_maps: i32,    /* max and min color maps */
    pub(crate) min_maps: i32,
    pub(crate) backing_store: i32, /* Never, WhenMapped, Always */
    pub(crate) save_unders: i32,
    pub(crate) root_input_mask: c_long, /* initial root input mask */
}

/*
 * Depth structure; contains information for each possible depth.
 */
#[repr(C)]
pub(crate) struct Depth {
    depth: i32,           /* this depth (Z) of the depth */
    nvisuals: i32,        /* number of Visual types at this depth */
    visuals: *mut Visual, /* list of visuals possible at this depth */
}

/*
 * Visual structure; contains information about colormapping possible.
 */
#[repr(C)]
pub(crate) struct Visual {
    ext_data: *mut XExtData, /* hook for extension to hang data */
    visualid: VisualID,      /* visual id of this visual */
    class: i32,              /* class of screen (monochrome, etc.) */
    red_mask: c_ulong,   /* mask values */
    green_mask: c_ulong,
    blue_mask: c_ulong,
    bits_per_rgb: i32, /* log base 2 of distinct color values */
    map_entries: i32,  /* color map entries */
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
pub(crate) const LINE_SOLID: i32 = 0;

/// Cap Style
pub(crate) const CAP_BUTT: i32 = 1;

/// Join Style
pub(crate) const JOIN_MITER: i32 = 0;

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
    pub(crate) width: u16,  /* Glyph width. */
    pub(crate) height: u16, /* Glyph height. */
    pub(crate) x: i16,      /* Horizontal Glyph center offset relative to the upper-left corner. */
    pub(crate) y: i16,      /* Vertical Glyph center offset relative to the upper-left corner. */
    pub(crate) x_off: i16,  /* Horizontal margin to the next Glyph. */
    pub(crate) y_off: i16,  /* Vertical margin to the next Glyph. */
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
    r#type: i32,
    display: *mut Display,       /* Display the event was read from */
    resourceid: XID,             /* resource id */
    serial: c_ulong,             /* serial number of failed request */
    pub(crate) error_code: u8,   /* error code of failed request */
    pub(crate) request_code: u8, /* Major op-code of failed request */
    minor_code: u8,              /* Minor op-code of failed request */
}

#[repr(C)]
pub(crate) struct XTextProperty {
    pub(crate) value: *mut u8,
    pub(crate) encoding: Atom,
    pub(crate) format: i32,
    pub(crate) nitems: c_ulong,
}

pub(crate) type XErrorHandler =
    extern "C" fn(display: *mut Display, error_event: *mut XErrorEvent) -> i32;

#[repr(C)]
#[derive(Debug)]
pub(crate) struct XSetWindowAttributes {
    pub(crate) background_pixmap: Pixmap, /* background or None or ParentRelative */
    pub(crate) background_pixel: c_ulong,  /* background pixel */
    pub(crate) border_pixmap: Pixmap,      /* border of the window */
    pub(crate) border_pixel: c_ulong,      /* border pixel value */
    pub(crate) bit_gravity: i32,           /* one of bit gravity values */
    pub(crate) win_gravity: i32,           /* one of the window gravity values */
    pub(crate) backing_store: i32,         /* NotUseful, WhenMapped, Always */
    pub(crate) backing_planes: c_ulong,    /* planes to be preserved if possible */
    pub(crate) backing_pixel: c_ulong,     /* value to use in restoring planes */
    pub(crate) save_under: i32,            /* should bits under be saved? (popups) */
    pub(crate) event_mask: c_long,         /* set of events that should be saved */
    pub(crate) do_not_propogate_mask: c_long, /* set of events that should not propagate */
    pub(crate) override_redirect: i32,    /* boolean value for override-redirect */
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
    pub(crate) input: i32,   /* does this application rely on the window manager to get keyboard input? */
    pub(crate) initial_state: i32, /* see below */
    pub(crate) icon_pixmap: Pixmap, /* pixmap to be used as icon */
    pub(crate) icon_window: Window, /* window to be used as icon */
    pub(crate) icon_x: i32,
    pub(crate) icon_y: i32,       /* initial position of icon */
    pub(crate) icon_mask: Pixmap, /* icon mask bitmap */
    pub(crate) window_group: XID, /* id of related window group */
                                  /* this structure may be extended in the future */
}

#[repr(C)]
pub(crate) struct XWindowAttributes {
    pub(crate) x: i32, /* location of window */
    pub(crate) y: i32,
    pub(crate) width: i32, /* width and height of window */
    pub(crate) height: i32,
    pub(crate) border_width: i32,          /* border width of window */
    pub(crate) depth: i32,                 /* depth of window */
    pub(crate) visual: *mut Visual,        /* the associated visual structure */
    pub(crate) root: Window,               /* root of screen containing window */
    pub(crate) class: i32,                 /* InputOutput, InputOnly*/
    pub(crate) bit_gravity: i32,              /* one of bit gravity values */
    pub(crate) win_gravity: i32,             /* one of the window gravity values */
    pub(crate) backing_store: i32,           /* NotUseful, WhenMapped, Always */
    pub(crate) backing_planes: c_ulong,      /* planes to be preserved if possible */
    pub(crate) backing_pixel: c_ulong,       /* value to be used when restoring planes */
    pub(crate) save_under: i32,              /* boolean, should bits under be saved? */
    pub(crate) colormap: Colormap,           /* color map to be associated with window */
    pub(crate) map_installed: i32,           /* boolean, is color map currently installed*/
    pub(crate) map_state: i32,               /* IsUnmapped, IsUnviewable, IsViewable */
    pub(crate) all_event_mask: c_long,       /* set of events all people have interest in*/
    pub(crate) your_event_mask: c_long,      /* my event mask */
    pub(crate) do_not_propogate_mask: c_long, /* set of events that should not propagate */
    pub(crate) override_redirect: i32,     /* boolean value for override-redirect */
    pub(crate) screen: *mut Screen,        /* back pointer to correct screen */
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct XConfigureEvent {
    pub(crate) r#type: i32,
    pub(crate) serial: c_ulong,
    pub(crate) send_event: i32,
    pub(crate) display: *mut Display,
    pub(crate) event: Window,
    pub(crate) window: Window,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) border_width: i32,
    pub(crate) above: Window,
    pub(crate) override_redirect: i32,
}

#[repr(C)]
pub(crate) struct XSizeHints {
    pub(crate) flags: c_long, /* marks which fields in this structure are defined */
    pub(crate) x: i32,        /* obsolete for new window mgrs, but clients */
    pub(crate) y: i32,
    pub(crate) width: i32, /* should set so old wm's don't mess up */
    pub(crate) height: i32,
    pub(crate) min_width: i32,
    pub(crate) min_height: i32,
    pub(crate) max_width: i32,
    pub(crate) max_height: i32,
    pub(crate) width_inc: i32,
    pub(crate) height_inc: i32,
    pub(crate) min_aspect: XSizeHintsAspect,
    pub(crate) max_aspect: XSizeHintsAspect,
    pub(crate) base_width: i32,  /* added by ICCCM version 1 */
    pub(crate) base_height: i32, /* added by ICCCM version 1 */
    pub(crate) win_gravity: i32, /* added by ICCCM version 1 */
}

#[repr(C)]
pub(crate) struct XSizeHintsAspect {
    pub(crate) x: i32, /* numerator */
    pub(crate) y: i32, /* denominator */
}

pub(crate) const REVERT_TO_POINTER_ROOT: i32 = 1;
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
pub(crate) const SUCCESS: u8 = 0;
pub(crate) const BAD_WINDOW: u8 = 3;
pub(crate) const BAD_MATCH: u8 = 8;
pub(crate) const BAD_DRAWABLE: u8 = 9;
pub(crate) const BAD_ACCESS: u8 = 10;

// Request Codes
pub(crate) const X_CREATE_WINDOW: u8 = 1;
pub(crate) const X_CHANGE_WINDOW_ATTRIBUTES: u8 = 2;
pub(crate) const X_GET_WINDOW_ATTRIBUTES: u8 = 3;
pub(crate) const X_DESTROY_WINDOW: u8 = 4;
pub(crate) const X_DESTROY_SUBWINDOWS: u8 = 5;
pub(crate) const X_CHANGE_SAVE_SET: u8 = 6;
pub(crate) const X_REPARENTWINDOW: u8 = 7;
pub(crate) const X_MAPWINDOW: u8 = 8;
pub(crate) const X_MAPSUBWINDOWS: u8 = 9;
pub(crate) const X_UNMAPWINDOW: u8 = 10;
pub(crate) const X_UNMAPSUBWINDOWS: u8 = 11;
pub(crate) const X_CONFIGUREWINDOW: u8 = 12;
pub(crate) const X_CIRCULATEWINDOW: u8 = 13;
pub(crate) const X_GETGEOMETRY: u8 = 14;
pub(crate) const X_QUERYTREE: u8 = 15;
pub(crate) const X_INTERNATOM: u8 = 16;
pub(crate) const X_GETATOMNAME: u8 = 17;
pub(crate) const X_CHANGEPROPERTY: u8 = 18;
pub(crate) const X_DELETEPROPERTY: u8 = 19;
pub(crate) const X_GETPROPERTY: u8 = 20;
pub(crate) const X_LISTPROPERTIES: u8 = 21;
pub(crate) const X_SETSELECTIONOWNER: u8 = 22;
pub(crate) const X_GETSELECTIONOWNER: u8 = 23;
pub(crate) const X_CONVERTSELECTION: u8 = 24;
pub(crate) const X_SENDEVENT: u8 = 25;
pub(crate) const X_GRABPOINTER: u8 = 26;
pub(crate) const X_UNGRABPOINTER: u8 = 27;
pub(crate) const X_GRABBUTTON: u8 = 28;
pub(crate) const X_UNGRABBUTTON: u8 = 29;
pub(crate) const X_CHANGEACTIVEPOINTERGRAB: u8 = 30;
pub(crate) const X_GRABKEYBOARD: u8 = 31;
pub(crate) const X_UNGRABKEYBOARD: u8 = 32;
pub(crate) const X_GRABKEY: u8 = 33;
pub(crate) const X_UNGRABKEY: u8 = 34;
pub(crate) const X_ALLOWEVENTS: u8 = 35;
pub(crate) const X_GRABSERVER: u8 = 36;
pub(crate) const X_UNGRABSERVER: u8 = 37;
pub(crate) const X_QUERYPOINTER: u8 = 38;
pub(crate) const X_GETMOTIONEVENTS: u8 = 39;
pub(crate) const X_TRANSLATECOORDS: u8 = 40;
pub(crate) const X_WARPPOINTER: u8 = 41;
pub(crate) const X_SETINPUTFOCUS: u8 = 42;
pub(crate) const X_GETINPUTFOCUS: u8 = 43;
pub(crate) const X_QUERYKEYMAP: u8 = 44;
pub(crate) const X_OPENFONT: u8 = 45;
pub(crate) const X_CLOSEFONT: u8 = 46;
pub(crate) const X_QUERYFONT: u8 = 47;
pub(crate) const X_QUERYTEXTEXTENTS: u8 = 48;
pub(crate) const X_LISTFONTS: u8 = 49;
pub(crate) const X_LISTFONTSWITHINFO: u8 = 50;
pub(crate) const X_SETFONTPATH: u8 = 51;
pub(crate) const X_GETFONTPATH: u8 = 52;
pub(crate) const X_CREATEPIXMAP: u8 = 53;
pub(crate) const X_FREEPIXMAP: u8 = 54;
pub(crate) const X_CREATEGC: u8 = 55;
pub(crate) const X_CHANGEGC: u8 = 56;
pub(crate) const X_COPYGC: u8 = 57;
pub(crate) const X_SETDASHES: u8 = 58;
pub(crate) const X_SETCLIPRECTANGLES: u8 = 59;
pub(crate) const X_FREEGC: u8 = 60;
pub(crate) const X_CLEARAREA: u8 = 61;
pub(crate) const X_COPYAREA: u8 = 62;
pub(crate) const X_COPYPLANE: u8 = 63;
pub(crate) const X_POLYPOINT: u8 = 64;
pub(crate) const X_POLYLINE: u8 = 65;
pub(crate) const X_POLYSEGMENT: u8 = 66;
pub(crate) const X_POLYRECTANGLE: u8 = 67;
pub(crate) const X_POLYARC: u8 = 68;
pub(crate) const X_FILLPOLY: u8 = 69;
pub(crate) const X_POLYFILLRECTANGLE: u8 = 70;
pub(crate) const X_POLYFILLARC: u8 = 71;
pub(crate) const X_PUTIMAGE: u8 = 72;
pub(crate) const X_GETIMAGE: u8 = 73;
pub(crate) const X_POLYTEXT8: u8 = 74;
pub(crate) const X_POLYTEXT16: u8 = 75;
pub(crate) const X_IMAGETEXT8: u8 = 76;
pub(crate) const X_IMAGETEXT16: u8 = 77;
pub(crate) const X_CREATECOLORMAP: u8 = 78;
pub(crate) const X_FREECOLORMAP: u8 = 79;
pub(crate) const X_COPYCOLORMAPANDFREE: u8 = 80;
pub(crate) const X_INSTALLCOLORMAP: u8 = 81;
pub(crate) const X_UNINSTALLCOLORMAP: u8 = 82;
pub(crate) const X_LISTINSTALLEDCOLORMAPS: u8 = 83;
pub(crate) const X_ALLOCCOLOR: u8 = 84;
pub(crate) const X_ALLOCNAMEDCOLOR: u8 = 85;
pub(crate) const X_ALLOCCOLORCELLS: u8 = 86;
pub(crate) const X_ALLOCCOLORPLANES: u8 = 87;
pub(crate) const X_FREECOLORS: u8 = 88;
pub(crate) const X_STORECOLORS: u8 = 89;
pub(crate) const X_STORENAMEDCOLOR: u8 = 90;
pub(crate) const X_QUERYCOLORS: u8 = 91;
pub(crate) const X_LOOKUPCOLOR: u8 = 92;
pub(crate) const X_CREATECURSOR: u8 = 93;
pub(crate) const X_CREATEGLYPHCURSOR: u8 = 94;
pub(crate) const X_FREECURSOR: u8 = 95;
pub(crate) const X_RECOLORCURSOR: u8 = 96;
pub(crate) const X_QUERYBESTSIZE: u8 = 97;
pub(crate) const X_QUERYEXTENSION: u8 = 98;
pub(crate) const X_LISTEXTENSIONS: u8 = 99;
pub(crate) const X_CHANGEKEYBOARDMAPPING: u8 = 100;
pub(crate) const X_GETKEYBOARDMAPPING: u8 = 101;
pub(crate) const X_CHANGEKEYBOARDCONTROL: u8 = 102;
pub(crate) const X_GETKEYBOARDCONTROL: u8 = 103;
pub(crate) const X_BELL: u8 = 104;
pub(crate) const X_CHANGEPOINTERCONTROL: u8 = 105;
pub(crate) const X_GETPOINTERCONTROL: u8 = 106;
pub(crate) const X_SETSCREENSAVER: u8 = 107;
pub(crate) const X_GETSCREENSAVER: u8 = 108;
pub(crate) const X_CHANGEHOSTS: u8 = 109;
pub(crate) const X_LISTHOSTS: u8 = 110;
pub(crate) const X_SETACCESSCONTROL: u8 = 111;
pub(crate) const X_SETCLOSEDOWNMODE: u8 = 112;
pub(crate) const X_KILLCLIENT: u8 = 113;
pub(crate) const X_ROTATEPROPERTIES: u8 = 114;
pub(crate) const X_FORCESCREENSAVER: u8 = 115;
pub(crate) const X_SETPOINTERMAPPING: u8 = 116;
pub(crate) const X_GETPOINTERMAPPING: u8 = 117;
pub(crate) const X_SETMODIFIERMAPPING: u8 = 118;
pub(crate) const X_GETMODIFIERMAPPING: u8 = 119;
pub(crate) const X_NOOPERATION: u8 = 127;

pub(crate) const XC_FLEUR: u32 = 52;
pub(crate) const XC_LEFT_PTR: u32 = 68;
pub(crate) const XC_SIZING: u32 = 120;

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

pub(crate) const SHIFT_MASK: u32 = 1 << 0;
pub(crate) const LOCK_MASK: u32 = 1 << 1;
pub(crate) const CONTROL_MASK: u32 = 1 << 2;
pub(crate) const MOD1_MASK: u32 = 1 << 3;
pub(crate) const MOD2_MASK: u32 = 1 << 4;
pub(crate) const MOD3_MASK: u32 = 1 << 5;
pub(crate) const MOD4_MASK: u32 = 1 << 6;
pub(crate) const MOD5_MASK: u32 = 1 << 7;

// KEY Codes
pub(crate) mod keycodes {
    #![allow(non_upper_case_globals)]
    pub(crate) const XK_space: u64 = 0x0020; /* U+0020 SPACE */
    pub(crate) const XK_exclam: u64 = 0x0021; /* U+0021 EXCLAMATION MARK */
    pub(crate) const XK_quotedbl: u64 = 0x0022; /* U+0022 QUOTATION MARK */
    pub(crate) const XK_numbersign: u64 = 0x0023; /* U+0023 NUMBER SIGN */
    pub(crate) const XK_dollar: u64 = 0x0024; /* U+0024 DOLLAR SIGN */
    pub(crate) const XK_percent: u64 = 0x0025; /* U+0025 PERCENT SIGN */
    pub(crate) const XK_ampersand: u64 = 0x0026; /* U+0026 AMPERSAND */
    pub(crate) const XK_apostrophe: u64 = 0x0027; /* U+0027 APOSTROPHE */
    pub(crate) const XK_quoteright: u64 = 0x0027; /* deprecated */
    pub(crate) const XK_parenleft: u64 = 0x0028; /* U+0028 LEFT PARENTHESIS */
    pub(crate) const XK_parenright: u64 = 0x0029; /* U+0029 RIGHT PARENTHESIS */
    pub(crate) const XK_asterisk: u64 = 0x002a; /* U+002A ASTERISK */
    pub(crate) const XK_plus: u64 = 0x002b; /* U+002B PLUS SIGN */
    pub(crate) const XK_comma: u64 = 0x002c; /* U+002C COMMA */
    pub(crate) const XK_minus: u64 = 0x002d; /* U+002D HYPHEN-MINUS */
    pub(crate) const XK_period: u64 = 0x002e; /* U+002E FULL STOP */
    pub(crate) const XK_slash: u64 = 0x002f; /* U+002F SOLIDUS */
    pub(crate) const XK_0: u64 = 0x0030; /* U+0030 DIGIT ZERO */
    pub(crate) const XK_1: u64 = 0x0031; /* U+0031 DIGIT ONE */
    pub(crate) const XK_2: u64 = 0x0032; /* U+0032 DIGIT TWO */
    pub(crate) const XK_3: u64 = 0x0033; /* U+0033 DIGIT THREE */
    pub(crate) const XK_4: u64 = 0x0034; /* U+0034 DIGIT FOUR */
    pub(crate) const XK_5: u64 = 0x0035; /* U+0035 DIGIT FIVE */
    pub(crate) const XK_6: u64 = 0x0036; /* U+0036 DIGIT SIX */
    pub(crate) const XK_7: u64 = 0x0037; /* U+0037 DIGIT SEVEN */
    pub(crate) const XK_8: u64 = 0x0038; /* U+0038 DIGIT EIGHT */
    pub(crate) const XK_9: u64 = 0x0039; /* U+0039 DIGIT NINE */
    pub(crate) const XK_colon: u64 = 0x003a; /* U+003A COLON */
    pub(crate) const XK_semicolon: u64 = 0x003b; /* U+003B SEMICOLON */
    pub(crate) const XK_less: u64 = 0x003c; /* U+003C LESS-THAN SIGN */
    pub(crate) const XK_equal: u64 = 0x003d; /* U+003D EQUALS SIGN */
    pub(crate) const XK_greater: u64 = 0x003e; /* U+003E GREATER-THAN SIGN */
    pub(crate) const XK_question: u64 = 0x003f; /* U+003F QUESTION MARK */
    pub(crate) const XK_at: u64 = 0x0040; /* U+0040 COMMERCIAL AT */
    pub(crate) const XK_A: u64 = 0x0041; /* U+0041 LATIN CAPITAL LETTER A */
    pub(crate) const XK_B: u64 = 0x0042; /* U+0042 LATIN CAPITAL LETTER B */
    pub(crate) const XK_C: u64 = 0x0043; /* U+0043 LATIN CAPITAL LETTER C */
    pub(crate) const XK_D: u64 = 0x0044; /* U+0044 LATIN CAPITAL LETTER D */
    pub(crate) const XK_E: u64 = 0x0045; /* U+0045 LATIN CAPITAL LETTER E */
    pub(crate) const XK_F: u64 = 0x0046; /* U+0046 LATIN CAPITAL LETTER F */
    pub(crate) const XK_G: u64 = 0x0047; /* U+0047 LATIN CAPITAL LETTER G */
    pub(crate) const XK_H: u64 = 0x0048; /* U+0048 LATIN CAPITAL LETTER H */
    pub(crate) const XK_I: u64 = 0x0049; /* U+0049 LATIN CAPITAL LETTER I */
    pub(crate) const XK_J: u64 = 0x004a; /* U+004A LATIN CAPITAL LETTER J */
    pub(crate) const XK_K: u64 = 0x004b; /* U+004B LATIN CAPITAL LETTER K */
    pub(crate) const XK_L: u64 = 0x004c; /* U+004C LATIN CAPITAL LETTER L */
    pub(crate) const XK_M: u64 = 0x004d; /* U+004D LATIN CAPITAL LETTER M */
    pub(crate) const XK_N: u64 = 0x004e; /* U+004E LATIN CAPITAL LETTER N */
    pub(crate) const XK_O: u64 = 0x004f; /* U+004F LATIN CAPITAL LETTER O */
    pub(crate) const XK_P: u64 = 0x0050; /* U+0050 LATIN CAPITAL LETTER P */
    pub(crate) const XK_Q: u64 = 0x0051; /* U+0051 LATIN CAPITAL LETTER Q */
    pub(crate) const XK_R: u64 = 0x0052; /* U+0052 LATIN CAPITAL LETTER R */
    pub(crate) const XK_S: u64 = 0x0053; /* U+0053 LATIN CAPITAL LETTER S */
    pub(crate) const XK_T: u64 = 0x0054; /* U+0054 LATIN CAPITAL LETTER T */
    pub(crate) const XK_U: u64 = 0x0055; /* U+0055 LATIN CAPITAL LETTER U */
    pub(crate) const XK_V: u64 = 0x0056; /* U+0056 LATIN CAPITAL LETTER V */
    pub(crate) const XK_W: u64 = 0x0057; /* U+0057 LATIN CAPITAL LETTER W */
    pub(crate) const XK_X: u64 = 0x0058; /* U+0058 LATIN CAPITAL LETTER X */
    pub(crate) const XK_Y: u64 = 0x0059; /* U+0059 LATIN CAPITAL LETTER Y */
    pub(crate) const XK_Z: u64 = 0x005a; /* U+005A LATIN CAPITAL LETTER Z */
    pub(crate) const XK_bracketleft: u64 = 0x005b; /* U+005B LEFT SQUARE BRACKET */
    pub(crate) const XK_backslash: u64 = 0x005c; /* U+005C REVERSE SOLIDUS */
    pub(crate) const XK_bracketright: u64 = 0x005d; /* U+005D RIGHT SQUARE BRACKET */
    pub(crate) const XK_asciicircum: u64 = 0x005e; /* U+005E CIRCUMFLEX ACCENT */
    pub(crate) const XK_underscore: u64 = 0x005f; /* U+005F LOW LINE */
    pub(crate) const XK_grave: u64 = 0x0060; /* U+0060 GRAVE ACCENT */
    pub(crate) const XK_quoteleft: u64 = 0x0060; /* deprecated */
    pub(crate) const XK_a: u64 = 0x0061; /* U+0061 LATIN SMALL LETTER A */
    pub(crate) const XK_b: u64 = 0x0062; /* U+0062 LATIN SMALL LETTER B */
    pub(crate) const XK_c: u64 = 0x0063; /* U+0063 LATIN SMALL LETTER C */
    pub(crate) const XK_d: u64 = 0x0064; /* U+0064 LATIN SMALL LETTER D */
    pub(crate) const XK_e: u64 = 0x0065; /* U+0065 LATIN SMALL LETTER E */
    pub(crate) const XK_f: u64 = 0x0066; /* U+0066 LATIN SMALL LETTER F */
    pub(crate) const XK_g: u64 = 0x0067; /* U+0067 LATIN SMALL LETTER G */
    pub(crate) const XK_h: u64 = 0x0068; /* U+0068 LATIN SMALL LETTER H */
    pub(crate) const XK_i: u64 = 0x0069; /* U+0069 LATIN SMALL LETTER I */
    pub(crate) const XK_j: u64 = 0x006a; /* U+006A LATIN SMALL LETTER J */
    pub(crate) const XK_k: u64 = 0x006b; /* U+006B LATIN SMALL LETTER K */
    pub(crate) const XK_l: u64 = 0x006c; /* U+006C LATIN SMALL LETTER L */
    pub(crate) const XK_m: u64 = 0x006d; /* U+006D LATIN SMALL LETTER M */
    pub(crate) const XK_n: u64 = 0x006e; /* U+006E LATIN SMALL LETTER N */
    pub(crate) const XK_o: u64 = 0x006f; /* U+006F LATIN SMALL LETTER O */
    pub(crate) const XK_p: u64 = 0x0070; /* U+0070 LATIN SMALL LETTER P */
    pub(crate) const XK_q: u64 = 0x0071; /* U+0071 LATIN SMALL LETTER Q */
    pub(crate) const XK_r: u64 = 0x0072; /* U+0072 LATIN SMALL LETTER R */
    pub(crate) const XK_s: u64 = 0x0073; /* U+0073 LATIN SMALL LETTER S */
    pub(crate) const XK_t: u64 = 0x0074; /* U+0074 LATIN SMALL LETTER T */
    pub(crate) const XK_u: u64 = 0x0075; /* U+0075 LATIN SMALL LETTER U */
    pub(crate) const XK_v: u64 = 0x0076; /* U+0076 LATIN SMALL LETTER V */
    pub(crate) const XK_w: u64 = 0x0077; /* U+0077 LATIN SMALL LETTER W */
    pub(crate) const XK_x: u64 = 0x0078; /* U+0078 LATIN SMALL LETTER X */
    pub(crate) const XK_y: u64 = 0x0079; /* U+0079 LATIN SMALL LETTER Y */
    pub(crate) const XK_z: u64 = 0x007a; /* U+007A LATIN SMALL LETTER Z */
    pub(crate) const XK_braceleft: u64 = 0x007b; /* U+007B LEFT CURLY BRACKET */
    pub(crate) const XK_bar: u64 = 0x007c; /* U+007C VERTICAL LINE */
    pub(crate) const XK_braceright: u64 = 0x007d; /* U+007D RIGHT CURLY BRACKET */
    pub(crate) const XK_asciitilde: u64 = 0x007e; /* U+007E TILDE */

    pub(crate) const XK_BackSpace: u64 = 0xff08; /* U+0008 BACKSPACE */
    pub(crate) const XK_Tab: u64 = 0xff09; /* U+0009 CHARACTER TABULATION */
    pub(crate) const XK_Linefeed: u64 = 0xff0a; /* U+000A LINE FEED */
    pub(crate) const XK_Clear: u64 = 0xff0b; /* U+000B LINE TABULATION */
    pub(crate) const XK_Return: u64 = 0xff0d; /* U+000D CARRIAGE RETURN */
    pub(crate) const XK_Pause: u64 = 0xff13; /* Pause, hold */
    pub(crate) const XK_Scroll_Lock: u64 = 0xff14;
    pub(crate) const XK_Sys_Req: u64 = 0xff15;
    pub(crate) const XK_Escape: u64 = 0xff1b; /* U+001B ESCAPE */
    pub(crate) const XK_Delete: u64 = 0xffff; /* U+007F DELETE */
}

//button names:
pub(crate) const BUTTON1: u32 = 1;
pub(crate) const BUTTON2: u32 = 2;
pub(crate) const BUTTON3: u32 = 3;
pub(crate) const BUTTON4: u32 = 4;
pub(crate) const BUTTON5: u32 = 5;

// ConfigureWindow Structure
pub(crate) const CWX: u32 = 1 << 0;
pub(crate) const CWY: u32 = 1 << 1;
pub(crate) const CW_WIDTH: u32 = 1 << 2;
pub(crate) const CW_HEIGHT: u32 = 1 << 3;
pub(crate) const CW_BORDER_WIDTH: u32 = 1 << 4;
pub(crate) const CW_SIBLING: u32 = 1 << 5;
pub(crate) const CW_STACK_MODE: u32 = 1 << 6;

/* definitions for initial window state */
pub(crate) const WITHDRAWN_STATE: i32 = 0; /* for windows that are not mapped */
pub(crate) const NORMAL_STATE: i32 = 1; /* most applications want to start this way */
pub(crate) const ICONIC_STATE: i32 = 3; /* application wants to start as an icon */

pub(crate) const ANY_BUTTON: u32 = 0;
pub(crate) const ANY_MODIFIER: u32 = 1 << 15;
pub(crate) const GRAB_MODE_SYNC: i32 = 0;
pub(crate) const GRAB_MODE_ASYNC: i32 = 1;
