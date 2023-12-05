#![feature(file_create_new)]

pub mod caddy;
pub mod cli;
pub mod services;
pub mod sysinfo_wrappers;
pub mod web;

use crate::services::Service;
use parking_lot::RwLock;
use std::fs::File;
use std::sync::{atomic::AtomicBool, Arc};

lazy_static::lazy_static! {
    pub static ref DEFAULT_CONFIG_PATH: &'static str = {
        if cfg!(debug_assertions) {
            "./testing/pmrs.toml"
        } else {
            "/etc/pmrs/pmrs.toml"
        }
    };
    pub static ref DASHBOARD_BUILD_PATH: &'static str = {
        if cfg!(debug_assertions) {
            "./dashboard/build/index.js"
        } else {
            "/usr/share/pmrs/dashboard/build/index.js"
        }
    };

    pub static ref PORT_ROCKET: isize = 8000;
    pub static ref PORT_DASHBOARD: isize = 5173;
    pub static ref PORT_CADDY: isize = 2019;

    pub static ref RUNNING: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    pub static ref SERVICES: Arc<Vec<Arc<RwLock<Service>>>> = Service::init(File::open(*DEFAULT_CONFIG_PATH).expect("the config file")).expect("a valid service");
}

#[macro_export]
macro_rules! function_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        name.strip_suffix("::f").unwrap()
    }};
}
