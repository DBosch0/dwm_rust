use std::{
    collections::HashMap,
    ffi::{CStr, CString},
};

use crate::external_functions::{
    XCloseDisplay, XOpenDisplay, XResourceManagerString, XrmDatabase, XrmGetResource,
    XrmGetStringDatabase, XrmValue,
};

pub(crate) type Resources = HashMap<&'static str, ResourceVal>;

#[derive(Debug)]
pub(crate) enum ResourceVal {
    String(String),
    Integer(u32),
    Bool(bool),
    Float(f32),
}

#[derive(Debug)]
pub(crate) enum ResourceValConfig {
    String(&'static str),
    Integer(u32),
    Bool(bool),
    Float(f32),
}

impl ResourceValConfig {
    pub(crate) fn to_resource_val(&self) -> ResourceVal {
        match self {
            ResourceValConfig::String(s) => ResourceVal::String((*s).to_owned()),
            ResourceValConfig::Integer(i) => ResourceVal::Integer(*i),
            ResourceValConfig::Bool(b) => ResourceVal::Bool(*b),
            ResourceValConfig::Float(f) => ResourceVal::Float(*f),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ResourceConfig {
    pub(crate) name: &'static str,
    pub(crate) x_resource_name: &'static str,
    pub(crate) default_value: ResourceValConfig,
}

macro_rules! load_resource {
    ($name:expr, $globals:expr, $variant:ident) => {{
        let $crate::resource::ResourceVal::$variant(value) = $globals
            .resources
            .get($name)
            .unwrap_or_else(|| panic!("{} is not in the resources map", $name))
        else {
            unreachable!("invalid type of variable {} in Resources map", $name);
        };
        *value
    }};
}

pub(crate) use load_resource;

macro_rules! borrow_resource {
    ($name:expr, $globals:expr, $variant:ident) => {{
        let $crate::resource::ResourceVal::$variant(value) = $globals
            .resources
            .get($name)
            .unwrap_or_else(|| panic!("{} is not in the resources map", $name))
        else {
            unreachable!("invalid type of variable {} in Resources map", $name);
        };
        value
    }};
}

pub(crate) use borrow_resource;

fn resource_load(db: XrmDatabase, name: &str, value: &mut ResourceVal) {
    let mut fullname: [i8; 256] = [0; 256];
    //NOTE: `type` points into XrmDatabase's internal memory — must not be freed.
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
}

pub(crate) fn load_xresources() -> Resources {
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
    } in crate::config::RESOURCE_MAPPING
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
