#[cfg(target_os = "android")]
extern crate android_logger;
#[cfg(target_os = "linux")]
extern crate env_logger;
extern crate jni;
extern crate log;
mod primer;
mod queue;
mod storage;
mod data;

#[cfg(target_os = "android")]
use android_logger::Config;
use jni::objects::{JClass, JObject, JValue};
use jni::sys::{jint, jsize};
use jni::JNIEnv;
use log::debug;
#[cfg(target_os = "android")]
use log::LevelFilter;
pub use primer::Primer;

// https://github.com/mozilla/rust-android-gradle

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Java_se_fzy_primevil_Primer_crunch(
    mut env: JNIEnv,
    _: JClass,
    j_chunk_size: jint,
    j_prime_crunch_listener: JObject,
) {
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(Config::default().with_max_level(LevelFilter::Trace))
    }

    let chunk_size = j_chunk_size as usize;

    Primer::new().crunch(chunk_size, |first_prime_index, primes| {
        let j_primes = env
            .new_long_array(primes.len() as jsize)
            .expect("new long array should succeed");

        env.set_long_array_region(&j_primes, 0 as jsize, primes)
            .expect("primes should copy to j_primes");

        debug!("Primes copied!");
        debug!("{:?}", j_prime_crunch_listener);

        env.call_method(
            &j_prime_crunch_listener,
            "onChunk",
            "(I[J)Z",
            &[
                JValue::Int(first_prime_index as i32),
                JValue::Object(&JObject::from(j_primes)),
            ],
        )
        .expect("jvm code should succeed")
        .z()
        .expect("return type should be bool")
    });
}
