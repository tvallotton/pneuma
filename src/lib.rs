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
//! Spawning a uthread is significatly cheaper than spawning a kernel thread.
//! Additionally, context switching between them does not require a change of the address space.
//!
//!
//!
//!
//!
//!
#![allow(clippy::option_map_unit_fn)]
#![allow(clippy::fn_to_numeric_cast)]
#![allow(clippy::new_ret_no_self)]
#![allow(clippy::len_without_is_empty)]
#![feature(thread_id_value)]

extern crate self as pneuma;

pub(crate) mod runtime;
pub(crate) mod sys;

pub mod fd;
pub mod fs;
pub mod future;
pub mod net;
pub mod reactor;
pub mod sync;
pub mod time;
pub mod uthread;
pub mod utils;
