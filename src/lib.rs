#![allow(dead_code, bad_style)]

pub const ENOMEM: i32 = 12;

include!(concat!(env!("OUT_DIR"), "/celix_bind.rs"));
include!(concat!(env!("OUT_DIR"), "/constants.rs"));