//! # Pneuma
//!
//! Pneuma is a user-level thread library for rust. It offers three main things:
//!
//! 1. A lightweight stackful coroutine implementation ([`pneuma::uthread`]).
//! 2. Asynchronous networking, file system, and synchronization primitives ([`pneuma::net`], [`pneuma::fs`], [`pneuma::sync`])
//! 3. Utilities for interoperability with the future based ecosystem ([`pneuma::future`]).
//!
//! # UThreads
//!
//! Pneuma offers uthreads, which are lightweight stackful coroutines.
//!
//!
#![allow(clippy::option_map_unit_fn)]
#![allow(clippy::fn_to_numeric_cast)]
#![allow(clippy::new_ret_no_self)]
#![allow(clippy::len_without_is_empty)]

extern crate self as pneuma;

pub(crate) mod runtime;
pub(crate) mod sys;

pub mod fs;
pub mod future;
pub mod net;
pub mod reactor;
pub mod sync;
pub mod time;
pub mod uthread;

// pub mod fs;
// pub mod net;
// mod reactor;
// mod runtime;
// pub mod sync;
// mod sys;
// pub mod thread;
// pub mod time;
