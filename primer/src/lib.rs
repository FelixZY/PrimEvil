#[cfg(target_os = "android")]
extern crate android_logger;
#[cfg(target_os = "linux")]
extern crate env_logger;
extern crate jni;
#[macro_use]
extern crate log;
mod primer;
mod queue;
mod storage;

#[cfg(target_os = "android")]
use android_logger::Config;
use jni::objects::JClass;
use jni::sys::{jint, jlong};
use jni::JNIEnv;
#[cfg(target_os = "android")]
use log::LevelFilter;
use std::cell::Cell;

pub use primer::Primer;

// https://github.com/mozilla/rust-android-gradle

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Java_se_fzy_primevil_Primer_crunch(_: JNIEnv, _: JClass, n: jint) -> jlong {
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(Config::default().with_max_level(LevelFilter::Trace))
    }

    let rustN = n as usize;
    let prime_count = Cell::new(0usize);
    let mut prime = 2i64;

    Primer::new().crunch(
        || prime_count.get() < rustN,
        |_, p| {
            prime_count.set(prime_count.get() + 1);
            prime = p;
        },
    );
    prime as jlong
}
