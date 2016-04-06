#![allow(dead_code, bad_style)]

pub const ENOMEM: i32 = 12;
pub const celix_thread_default : (i32,i32) = (0,0);

include!(concat!(env!("OUT_DIR"), "/celix_bind.rs"));
include!(concat!(env!("OUT_DIR"), "/constants.rs"));

// macros for functions needed by celix
