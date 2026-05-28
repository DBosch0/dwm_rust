#![allow(dead_code)]
use crate::{
    Button, ClickState, Key, Layout, ResourceConfig, ResourceValConfig, Rule,
    external_functions::{
        BUTTON1, BUTTON2, BUTTON3, CONTROL_MASK, MOD1_MASK, SHIFT_MASK, keycodes::*,
    },
};

/* appearance */
pub const BORDER_PX_DEFAULT: u32 = 3; /* border pixel of windows */
pub const SNAP_DEFAULT: u32 = 32; /* snap pixel */
pub const SHOW_BAR_DEFAULT: bool = true; /* false means no bar */
pub const TOP_BAR_DEFAULT: bool = true; /* false means bottom bar */
pub const FONTS: &[&str] = &[
    "monospace:size=10",
    "NotoColorEmoji:pixelsize=10:antialias=true:autohint=true",
];
pub const DMENUFONT: &str = "monospace:size=10";
pub const NORM_BG_COLOR_DEFAULT: &str = "#222222";
pub const NORM_BORDER_COLOR_DEFAULT: &str = "#444444";
pub const NORM_FG_COLOR_DEFAULT: &str = "#bbbbbb";
pub const SEL_FG_COLOR_DEFAULT: &str = "#eeeeee";
pub const SEL_BORDER_COLOR_DEFAULT: &str = "#770000";
pub const SEL_BG_COLOR_DEFAULT: &str = "#005577";
pub const GAPP_IH_DEFAULT: u32 = 20;
pub const GAPP_IV_DEFAULT: u32 = 20;
pub const GAPP_OH_DEFAULT: u32 = 20;
pub const GAPP_OV_DEFAULT: u32 = 20;
pub const SMART_GAPS_DEFAULT: bool = false;
pub const FORCE_VSPLIT: bool = true; /* nrowgrid layout: force two clients to always split vertically */

// Must be the names of the color variables, not the variables themselves.
// Will be loaded dynamically at runtime. If using pywall to set Xresouces
// those colors will be loaded instead of the defaults above.
pub const COLORS: &[[&str; 3]] = &[
    /*fg        bg         border   */
    ["NORM_FG_COLOR", "NORM_BG_COLOR", "NORM_BORDER_COLOR"], // SchemNorm (0)
    ["SEL_FG_COLOR", "SEL_BG_COLOR", "SEL_BORDER_COLOR"],    // SchemeSel (1)
];

//Tagging
pub const TAGS: &[&str] = &["1", "2", "3", "4", "5", "6", "7", "8", "9"];

//Rules
pub const RULES: &[Rule] = &[
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
pub const M_FACT_DEFAULT: f32 = 0.55; /* factor of master area size [0.05..0.95] */
pub const N_MASTER_DEFAULT: u32 = 1; /* number of clients in master area */
pub const RESIZE_HINTS_DEFAULT: bool = false; /* true means respect size hints in tiled resizals */
pub const LOCK_FULLSCREEN: bool = true; /* true will force focus on the fullscreen window */
pub const REFRESH_RATE: u32 = 120; /* refresh rate (per second) for client move/resize */

pub const LAYOUTS: &[Layout] = &[
    Layout {
        symbol: "[]=",
        arrange: Some(crate::vanitygaps::tile),
    },
    Layout {
        symbol: "[M]",
        arrange: Some(crate::monocle),
    },
    Layout {
        symbol: "[@]",
        arrange: Some(crate::vanitygaps::spiral),
    },
    Layout {
        symbol: "[\\]",
        arrange: Some(crate::vanitygaps::dwindle),
    },
    Layout {
        symbol: "H[]",
        arrange: Some(crate::vanitygaps::deck),
    },
    Layout {
        symbol: "TTT",
        arrange: Some(crate::vanitygaps::bstack),
    },
    Layout {
        symbol: "===",
        arrange: Some(crate::vanitygaps::bstackhoriz),
    },
    Layout {
        symbol: "HHH",
        arrange: Some(crate::vanitygaps::grid),
    },
    Layout {
        symbol: "###",
        arrange: Some(crate::vanitygaps::nrowgrid),
    },
    Layout {
        symbol: "---",
        arrange: Some(crate::vanitygaps::horizgrid),
    },
    Layout {
        symbol: ":::",
        arrange: Some(crate::vanitygaps::gaplessgrid),
    },
    Layout {
        symbol: "|M|",
        arrange: Some(crate::vanitygaps::centeredmaster),
    },
    Layout {
        symbol: ">M>",
        arrange: Some(crate::vanitygaps::centeredfloatingmaster),
    },
    Layout {
        symbol: "><>",
        arrange: None,
    },
];

/* key definitions */
pub const MODKEY: u32 = MOD1_MASK;

//Underscored values will be replaced with dynmically loaded XResources
pub const DMENUCMD: &[&str] = &[
    "dmenu_run",
    "-m",
    "__DMENU_MONITOR_PLACEHOLDER",
    "-fn",
    DMENUFONT,
    "-nb",
    "__NORM_BG_COLOR",
    "-nf",
    "__NORM_FG_COLOR",
    "-sb",
    "__SEL_BORDER_COLOR",
    "-sf",
    "__SEL_FG_COLOR",
];

pub const TERMCD: &[&str] = &["st"];

pub const RESOURCE_MAPPING: &[ResourceConfig] = &[
    ResourceConfig {
        name: "NORM_BORDER_COLOR",
        x_resource_name: "color0",
        default_value: ResourceValConfig::String(NORM_BORDER_COLOR_DEFAULT),
    },
    ResourceConfig {
        name: "SEL_BORDER_COLOR",
        x_resource_name: "color8",
        default_value: ResourceValConfig::String(SEL_BORDER_COLOR_DEFAULT),
    },
    ResourceConfig {
        name: "NORM_BG_COLOR",
        x_resource_name: "color0",
        default_value: ResourceValConfig::String(NORM_BG_COLOR_DEFAULT),
    },
    ResourceConfig {
        name: "NORM_FG_COLOR",
        x_resource_name: "color4",
        default_value: ResourceValConfig::String(NORM_FG_COLOR_DEFAULT),
    },
    ResourceConfig {
        name: "SEL_FG_COLOR",
        x_resource_name: "color0",
        default_value: ResourceValConfig::String(SEL_FG_COLOR_DEFAULT),
    },
    ResourceConfig {
        name: "SEL_BG_COLOR",
        x_resource_name: "color4",
        default_value: ResourceValConfig::String(SEL_BG_COLOR_DEFAULT),
    },
    ResourceConfig {
        name: "BORDER_PX",
        x_resource_name: "borderpx",
        default_value: ResourceValConfig::Integer(BORDER_PX_DEFAULT),
    },
    ResourceConfig {
        name: "SNAP",
        x_resource_name: "snap",
        default_value: ResourceValConfig::Integer(SNAP_DEFAULT),
    },
    ResourceConfig {
        name: "SHOW_BAR",
        x_resource_name: "showbar",
        default_value: ResourceValConfig::Bool(SHOW_BAR_DEFAULT),
    },
    ResourceConfig {
        name: "TOP_BAR",
        x_resource_name: "topbar",
        default_value: ResourceValConfig::Bool(TOP_BAR_DEFAULT),
    },
    ResourceConfig {
        name: "N_MASTER",
        x_resource_name: "nmaster",
        default_value: ResourceValConfig::Integer(N_MASTER_DEFAULT),
    },
    ResourceConfig {
        name: "RESIZE_HINTS",
        x_resource_name: "resizehints",
        default_value: ResourceValConfig::Bool(RESIZE_HINTS_DEFAULT),
    },
    ResourceConfig {
        name: "M_FACT",
        x_resource_name: "mfact",
        default_value: ResourceValConfig::Float(M_FACT_DEFAULT),
    },
    ResourceConfig {
        name: "GAPP_IH",
        x_resource_name: "gappih",
        default_value: ResourceValConfig::Integer(GAPP_IH_DEFAULT),
    },
    ResourceConfig {
        name: "GAPP_IV",
        x_resource_name: "gappiv",
        default_value: ResourceValConfig::Integer(GAPP_IV_DEFAULT),
    },
    ResourceConfig {
        name: "GAPP_OH",
        x_resource_name: "gappoh",
        default_value: ResourceValConfig::Integer(GAPP_OH_DEFAULT),
    },
    ResourceConfig {
        name: "GAPP_OV",
        x_resource_name: "gappov",
        default_value: ResourceValConfig::Integer(GAPP_OV_DEFAULT),
    },
    ResourceConfig {
        name: "SMART_GAPS",
        x_resource_name: "smartgaps",
        default_value: ResourceValConfig::Bool(SMART_GAPS_DEFAULT),
    },
    // ["SWALLOW_FLOATING","swallowfloating"],
];

pub const KEYS: &[Key] = &[
    Key {
        r#mod: MODKEY,
        keysym: XK_p,
        func: Some(crate::spawn),
        arg: crate::Arg::Command(DMENUCMD),
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
        func: Some(crate::togglefullscreen),
        arg: crate::Arg::I(0),
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
    Key {
        r#mod: MODKEY,
        keysym: XK_F5,
        func: Some(crate::xrdb),
        arg: crate::Arg::I(0),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_s,
        func: Some(crate::togglesticky),
        arg: crate::Arg::I(0),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_j,
        func: Some(crate::focusstack),
        arg: crate::Arg::I(1 + 2000),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_k,
        func: Some(crate::focusstack),
        arg: crate::Arg::I(-1 + 2000),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_v,
        func: Some(crate::focusstack),
        arg: crate::Arg::I(0),
    },
    Key {
        r#mod: MODKEY | SHIFT_MASK,
        keysym: XK_j,
        func: Some(crate::pushstack),
        arg: crate::Arg::I(1 + 2000),
    },
    Key {
        r#mod: MODKEY | SHIFT_MASK,
        keysym: XK_k,
        func: Some(crate::pushstack),
        arg: crate::Arg::I(-1 + 2000),
    },
    Key {
        r#mod: MODKEY | SHIFT_MASK,
        keysym: XK_v,
        func: Some(crate::pushstack),
        arg: crate::Arg::I(0),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_g,
        func: Some(crate::shiftviewclients),
        arg: crate::Arg::I(1),
    },
    Key {
        r#mod: MODKEY | SHIFT_MASK,
        keysym: XK_g,
        func: Some(crate::shifttagclients),
        arg: crate::Arg::I(1),
    },
    Key {
        r#mod: MODKEY,
        keysym: XK_semicolon,
        func: Some(crate::shiftviewclients),
        arg: crate::Arg::I(-1),
    },
    Key {
        r#mod: MODKEY | SHIFT_MASK,
        keysym: XK_semicolon,
        func: Some(crate::shifttagclients),
        arg: crate::Arg::I(-1),
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
        arg: crate::Arg::Layout(None),
    },
    Button {
        click: ClickState::LtSymbol,
        mask: 0,
        button: BUTTON3,
        func: Some(crate::setlayout),
        arg: crate::Arg::Layout(Some(&LAYOUTS[LAYOUTS.len() - 1])),
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
