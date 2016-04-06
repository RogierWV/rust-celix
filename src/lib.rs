#![allow(dead_code, bad_style)]

pub const ENOMEM: i32 = 12;
pub const celix_thread_default : celix_thread_t = celix_thread_t {
    threadInitialized: 0,
    thread: 0,
};

include!(concat!(env!("OUT_DIR"), "/celix_bind.rs"));
include!(concat!(env!("OUT_DIR"), "/constants.rs"));

// TODO: macros for functions needed by celix
