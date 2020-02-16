#![allow(dead_code)]

use super::*;
use std::sync::{
    Arc,
    Mutex,
    atomic::{AtomicBool, Ordering}
};

pub struct OneshotPlayer {
    
}