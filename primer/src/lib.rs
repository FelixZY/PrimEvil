extern crate jni;

use jni::objects::JClass;
use jni::sys::jlong;
use jni::JNIEnv;

// https://github.com/mozilla/rust-android-gradle

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Java_se_fzy_primevil_primer_Native_add(
    _: JNIEnv,
    _: JClass,
    left: jlong,
    right: jlong,
) -> jlong {
    left + right
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         let result = Java_se_fzy_primevil_primer_Native_add( 2, 2);
//         assert_eq!(result, 4);
//     }
// }
