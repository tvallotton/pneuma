//! # Pneuma
//!
//! Pneuma is a user-level thread library for rust. It offers three main things:
//!
//! 1. A lightweight stackful coroutine implementation ([`pneuma::thread`]).
//! 2. Asynchronous networking, file system, and synchronization primitives ([`pneuma::net`], [`pneuma::fs`], [`pneuma::sync`])
//! 3. utilities for interoperability with the future based ecosystem ([`pneuma::future`]).
//!
//!
//!

#![allow(clippy::fn_to_numeric_cast)]
#![allow(clippy::new_ret_no_self)]

extern crate self as pneuma;

#[macro_use]
mod utils;
pub mod fs;
pub mod future;
pub mod net;
mod reactor;
mod runtime;
pub mod sync;
mod sys;
pub mod thread;
pub mod time;
