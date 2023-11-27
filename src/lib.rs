#![feature(file_create_new)]

pub mod cli;
pub mod services;
pub mod web;
use std::sync::{atomic::AtomicBool, Arc};

lazy_static::lazy_static! {
    pub static ref RUNNING: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
}

pub const DEFAULT_CONFIG_PATH: &str = "./testing/pmrs.toml";

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
