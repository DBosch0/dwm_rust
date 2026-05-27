#![allow(dead_code)]
use crate::{
    Button, ClickState, Key, Layout, Rule,
    external_functions::{
        BUTTON1, BUTTON2, BUTTON3, CONTROL_MASK, MOD1_MASK, SHIFT_MASK, keycodes::*,
    },
};

/* appearance */
pub const BORDERPX: u32 = 1; /* border pixel of windows */
pub const SNAP: u32 = 32; /* snap pixel */
pub const SHOWBAR: bool = true; /* false means no bar */
pub const TOPBAR: bool = true; /* false means bottom bar */
pub const FONTS: &[&str] = &[
    "monospace:size=10",
    "NotoColorEmoji:pixelsize=10:antialias=true:autohint=true",
];
pub const DMENUFONT: &str = "monospace:size=10";
pub const COL_GRAY1: &str = "#222222";
pub const COL_GRAY2: &str = "#444444";
pub const COL_GRAY3: &str = "#bbbbbb";
pub const COL_GRAY4: &str = "#eeeeee";
pub const COL_CYAN: &str = "#005577";
pub const COLORS: &[[&str; 3]] = &[
    /*fg        bg         border   */
    [COL_GRAY3, COL_GRAY1, COL_GRAY2], // SchemNorm (0)
    [COL_GRAY4, COL_CYAN, COL_CYAN],   // SchemeSel (1)
];

//Tagging
pub const TAGS: &[&str] = &["1", "2", "3", "4", "5", "6", "7", "8", "9"];

//Rules
pub const RULES: &[Rule] = &[
    /* class      instance    title       tags mask     isfloating   monitor */
    Rule {
        class: "Gimp",
        instance: "",
        title: "",
        tags: 0,
        isfloating: true,
        monitor: -1,
    },
    Rule {
        class: "Firefox",
        instance: "",
        title: "",
        tags: 1 << 8,
        isfloating: false,
        monitor: -1,
    },
];

//layout(s)
pub const MFACT: f32 = 0.55; /* factor of master area size [0.05..0.95] */
pub const NMASTER: i32 = 1; /* number of clients in master area */
pub const RESIZEHINTS: bool = true; /* true means respect size hints in tiled resizals */
pub const LOCKFULLSCREEN: bool = true; /* true will force focus on the fullscreen window */
pub const REFRESHRATE: i32 = 120; /* refresh rate (per second) for client move/resize */

pub const LAYOUTS: &[Layout] = &[
    Layout {
        symbol: "[]=",
        arrange: Some(crate::tile),
    },
    Layout {
        symbol: "><>",
        arrange: None,
    },
    Layout {
        symbol: "[M]",
        arrange: Some(crate::monocle),
    },
];

/* key definitions */
pub const MODKEY: u32 = MOD1_MASK;

pub const DEMENUCMD: &[&str] = &[
    "dmenu_run",
    "-m",
    "PLACEHOLDER",
    "-fn",
    DMENUFONT,
    "-nb",
    COL_GRAY1,
    "-nf",
    COL_GRAY3,
    "-sb",
    COL_CYAN,
    "-sf",
    COL_GRAY4,
];

pub const TERMCD: &[&str] = &["st"];

pub const KEYS: &[Key] = &[
    Key {
        r#mod: MODKEY,
        keysym: XK_p,
        func: Some(crate::spawn),
        arg: crate::Arg::Command(DEMENUCMD),
    },
    Key {
        r#mod: MODKEY | SHIFT_MASK,
        keysym: XK_Return,
        func: Some(crate::spawn),
        arg: crate::Arg::Command(TERMCD),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_b,
        func: Some(crate::togglebar),
        arg: crate::Arg::I(0),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_j,
        func: Some(crate::focusstack),
        arg: crate::Arg::I(1),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_k,
        func: Some(crate::focusstack),
        arg: crate::Arg::I(-1),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_i,
        func: Some(crate::incnmaster),
        arg: crate::Arg::I(1),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_d,
        func: Some(crate::incnmaster),
        arg: crate::Arg::I(-1),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_h,
        func: Some(crate::setmfact),
        arg: crate::Arg::F(-0.05),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_l,
        func: Some(crate::setmfact),
        arg: crate::Arg::F(0.05),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_Return,
        func: Some(crate::zoom),
        arg: crate::Arg::I(0),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_Tab,
        func: Some(crate::view),
        arg: crate::Arg::I(0),
    },
    Key {
        r#mod: MODKEY | SHIFT_MASK,
        keysym: XK_c,
        func: Some(crate::killclient),
        arg: crate::Arg::I(1),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_t,
        func: Some(crate::setlayout),
        arg: crate::Arg::Layout(Some(&LAYOUTS[0])),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_f,
        func: Some(crate::setlayout),
        arg: crate::Arg::Layout(Some(&LAYOUTS[1])),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_m,
        func: Some(crate::setlayout),
        arg: crate::Arg::Layout(Some(&LAYOUTS[2])),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_space,
        func: Some(crate::setlayout),
        arg: crate::Arg::Layout(None),
    },
    Key {
        r#mod: MODKEY | SHIFT_MASK,
        keysym: XK_space,
        func: Some(crate::togglefloating),
        arg: crate::Arg::I(0),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_0,
        func: Some(crate::view),
        arg: crate::Arg::Ui(!0),
    },
    Key {
        r#mod: MODKEY | SHIFT_MASK,
        keysym: XK_0,
        func: Some(crate::tag),
        arg: crate::Arg::Ui(!0),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_comma,
        func: Some(crate::focusmon),
        arg: crate::Arg::I(-1),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_period,
        func: Some(crate::focusmon),
        arg: crate::Arg::I(1),
    },
    Key {
        r#mod: MODKEY | SHIFT_MASK,
        keysym: XK_comma,
        func: Some(crate::tagmon),
        arg: crate::Arg::I(-1),
    },
    Key {
        r#mod: MODKEY | SHIFT_MASK,
        keysym: XK_period,
        func: Some(crate::tagmon),
        arg: crate::Arg::I(1),
    },
    Key {
        r#mod: MODKEY | SHIFT_MASK,
        keysym: XK_q,
        func: Some(crate::quit),
        arg: crate::Arg::I(0),
    },
    // The '1' key
    Key {
        r#mod: MODKEY,
        keysym: XK_1,
        func: Some(crate::view),
        arg: crate::Arg::Ui(1 << 0),
    },
    Key {
        r#mod: MODKEY | CONTROL_MASK,
        keysym: XK_1,
        func: Some(crate::toggleview),
        arg: crate::Arg::Ui(1 << 0),
    },
    Key {
        r#mod: MODKEY | SHIFT_MASK,
        keysym: XK_1,
        func: Some(crate::tag),
        arg: crate::Arg::Ui(1 << 0),
    },
    Key {
        r#mod: MODKEY | CONTROL_MASK | SHIFT_MASK,
        keysym: XK_1,
        func: Some(crate::toggletag),
        arg: crate::Arg::Ui(1 << 0),
    },
    // 2 key
    Key {
        r#mod: MODKEY,
        keysym: XK_2,
        func: Some(crate::view),
        arg: crate::Arg::Ui(1 << 1),
    },
    Key {
        r#mod: MODKEY | CONTROL_MASK,
        keysym: XK_2,
        func: Some(crate::toggleview),
        arg: crate::Arg::Ui(1 << 1),
    },
    Key {
        r#mod: MODKEY | SHIFT_MASK,
        keysym: XK_2,
        func: Some(crate::tag),
        arg: crate::Arg::Ui(1 << 1),
    },
    Key {
        r#mod: MODKEY | CONTROL_MASK | SHIFT_MASK,
        keysym: XK_2,
        func: Some(crate::toggletag),
        arg: crate::Arg::Ui(1 << 1),
    },
    // 3
    Key {
        r#mod: MODKEY,
        keysym: XK_3,
        func: Some(crate::view),
        arg: crate::Arg::Ui(1 << 2),
    },
    Key {
        r#mod: MODKEY | CONTROL_MASK,
        keysym: XK_3,
        func: Some(crate::toggleview),
        arg: crate::Arg::Ui(1 << 2),
    },
    Key {
        r#mod: MODKEY | SHIFT_MASK,
        keysym: XK_3,
        func: Some(crate::tag),
        arg: crate::Arg::Ui(1 << 2),
    },
    Key {
        r#mod: MODKEY | CONTROL_MASK | SHIFT_MASK,
        keysym: XK_3,
        func: Some(crate::toggletag),
        arg: crate::Arg::Ui(1 << 2),
    },
    // 4
    Key {
        r#mod: MODKEY,
        keysym: XK_4,
        func: Some(crate::view),
        arg: crate::Arg::Ui(1 << 3),
    },
    Key {
        r#mod: MODKEY | CONTROL_MASK,
        keysym: XK_4,
        func: Some(crate::toggleview),
        arg: crate::Arg::Ui(1 << 3),
    },
    Key {
        r#mod: MODKEY | SHIFT_MASK,
        keysym: XK_4,
        func: Some(crate::tag),
        arg: crate::Arg::Ui(1 << 3),
    },
    Key {
        r#mod: MODKEY | CONTROL_MASK | SHIFT_MASK,
        keysym: XK_4,
        func: Some(crate::toggletag),
        arg: crate::Arg::Ui(1 << 3),
    },
    // 5
    Key {
        r#mod: MODKEY,
        keysym: XK_5,
        func: Some(crate::view),
        arg: crate::Arg::Ui(1 << 4),
    },
    Key {
        r#mod: MODKEY | CONTROL_MASK,
        keysym: XK_5,
        func: Some(crate::toggleview),
        arg: crate::Arg::Ui(1 << 4),
    },
    Key {
        r#mod: MODKEY | SHIFT_MASK,
        keysym: XK_5,
        func: Some(crate::tag),
        arg: crate::Arg::Ui(1 << 4),
    },
    Key {
        r#mod: MODKEY | CONTROL_MASK | SHIFT_MASK,
        keysym: XK_5,
        func: Some(crate::toggletag),
        arg: crate::Arg::Ui(1 << 4),
    },
    // 6
    Key {
        r#mod: MODKEY,
        keysym: XK_6,
        func: Some(crate::view),
        arg: crate::Arg::Ui(1 << 5),
    },
    Key {
        r#mod: MODKEY | CONTROL_MASK,
        keysym: XK_6,
        func: Some(crate::toggleview),
        arg: crate::Arg::Ui(1 << 5),
    },
    Key {
        r#mod: MODKEY | SHIFT_MASK,
        keysym: XK_6,
        func: Some(crate::tag),
        arg: crate::Arg::Ui(1 << 5),
    },
    Key {
        r#mod: MODKEY | CONTROL_MASK | SHIFT_MASK,
        keysym: XK_6,
        func: Some(crate::toggletag),
        arg: crate::Arg::Ui(1 << 5),
    },
    // 7
    Key {
        r#mod: MODKEY,
        keysym: XK_7,
        func: Some(crate::view),
        arg: crate::Arg::Ui(1 << 6),
    },
    Key {
        r#mod: MODKEY | CONTROL_MASK,
        keysym: XK_7,
        func: Some(crate::toggleview),
        arg: crate::Arg::Ui(1 << 6),
    },
    Key {
        r#mod: MODKEY | SHIFT_MASK,
        keysym: XK_7,
        func: Some(crate::tag),
        arg: crate::Arg::Ui(1 << 6),
    },
    Key {
        r#mod: MODKEY | CONTROL_MASK | SHIFT_MASK,
        keysym: XK_7,
        func: Some(crate::toggletag),
        arg: crate::Arg::Ui(1 << 6),
    },
    // 8
    Key {
        r#mod: MODKEY,
        keysym: XK_8,
        func: Some(crate::view),
        arg: crate::Arg::Ui(1 << 7),
    },
    Key {
        r#mod: MODKEY | CONTROL_MASK,
        keysym: XK_8,
        func: Some(crate::toggleview),
        arg: crate::Arg::Ui(1 << 7),
    },
    Key {
        r#mod: MODKEY | SHIFT_MASK,
        keysym: XK_8,
        func: Some(crate::tag),
        arg: crate::Arg::Ui(1 << 7),
    },
    Key {
        r#mod: MODKEY | CONTROL_MASK | SHIFT_MASK,
        keysym: XK_8,
        func: Some(crate::toggletag),
        arg: crate::Arg::Ui(1 << 7),
    },
    // 9
    Key {
        r#mod: MODKEY,
        keysym: XK_9,
        func: Some(crate::view),
        arg: crate::Arg::Ui(1 << 8),
    },
    Key {
        r#mod: MODKEY | CONTROL_MASK,
        keysym: XK_9,
        func: Some(crate::toggleview),
        arg: crate::Arg::Ui(1 << 8),
    },
    Key {
        r#mod: MODKEY | SHIFT_MASK,
        keysym: XK_9,
        func: Some(crate::tag),
        arg: crate::Arg::Ui(1 << 8),
    },
    Key {
        r#mod: MODKEY | CONTROL_MASK | SHIFT_MASK,
        keysym: XK_9,
        func: Some(crate::toggletag),
        arg: crate::Arg::Ui(1 << 8),
    },
];

pub const BUTTONS: &[Button] = &[
    Button {
        click: ClickState::LtSymbol,
        mask: 0,
        button: BUTTON1,
        func: Some(crate::setlayout),
        arg: crate::Arg::I(0),
    },
    Button {
        click: ClickState::LtSymbol,
        mask: 0,
        button: BUTTON3,
        func: Some(crate::setlayout),
        arg: crate::Arg::Layout(Some(&LAYOUTS[2])),
    },
    Button {
        click: ClickState::WinTitle,
        mask: 0,
        button: BUTTON2,
        func: Some(crate::zoom),
        arg: crate::Arg::I(0),
    },
    Button {
        click: ClickState::StatusText,
        mask: 0,
        button: BUTTON2,
        func: Some(crate::spawn),
        arg: crate::Arg::Command(TERMCD),
    },
    Button {
        click: ClickState::ClientWin,
        mask: MODKEY,
        button: BUTTON1,
        func: Some(crate::movemouse),
        arg: crate::Arg::I(0),
    },
    Button {
        click: ClickState::ClientWin,
        mask: MODKEY,
        button: BUTTON2,
        func: Some(crate::togglefloating),
        arg: crate::Arg::I(0),
    },
    Button {
        click: ClickState::ClientWin,
        mask: MODKEY,
        button: BUTTON3,
        func: Some(crate::resizemouse),
        arg: crate::Arg::I(0),
    },
    Button {
        click: ClickState::TagBar,
        mask: 0,
        button: BUTTON1,
        func: Some(crate::view),
        arg: crate::Arg::I(0),
    },
    Button {
        click: ClickState::TagBar,
        mask: 0,
        button: BUTTON3,
        func: Some(crate::toggleview),
        arg: crate::Arg::I(0),
    },
    Button {
        click: ClickState::TagBar,
        mask: MODKEY,
        button: BUTTON1,
        func: Some(crate::tag),
        arg: crate::Arg::I(0),
    },
    Button {
        click: ClickState::TagBar,
        mask: MODKEY,
        button: BUTTON3,
        func: Some(crate::toggletag),
        arg: crate::Arg::I(0),
    },
];
