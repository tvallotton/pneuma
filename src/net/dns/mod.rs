//!
//!
//! Asynchronous DNS lookups.
//!
//! This crate provides asynchronous DNS lookups. It uses the following mechanisms
//! to resolve hostnames:
//!
//! - On `cfg(unix)`, it uses a custom implementation based on [`async-fs`] for reading
//!   files, [`async-io`] for communication with the server, and [`dns-protocol`] for the
//!   protocol implementation.
//! - On `cfg(windows)`, it uses the [`DnsQueryEx`] function to make asynchronous queries.
//!
//! [`DnsQueryEx`]: https://docs.microsoft.com/en-us/windows/win32/api/windns/nf-windns-dnsqueryex
//! [`async-fs`]: https://crates.io/crates/async-fs
//! [`async-io`]: https://crates.io/crates/async-io
//! [`dns-protocol`]: https://crates.io/crates/dns-protocol

// Non-windows platforms use no unsafe code.
#![cfg_attr(not(windows), forbid(unsafe_code))]
#![forbid(missing_docs, future_incompatible)]

#[cfg(unix)]
mod unix;
#[cfg(unix)]
use unix as sys;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows as sys;

use std::io;
use std::iter::FusedIterator;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Preform a DNS lookup, retrieving the IP addresses and other necessary information.
pub fn lookup(name: &str) -> io::Result<impl Iterator<Item = IpAddr>> {
    // Try to parse the name as an IP address.
    if let Ok(ip) = name.parse::<Ipv4Addr>() {
        return Ok(OneOrMany::One(Some(ip.into())));
    }

    if let Ok(ip) = name.parse::<Ipv6Addr>() {
        return Ok(OneOrMany::One(Some(ip.into())));
    }

    // Perform the actual DNS lookup.
    sys::lookup(name).map(|v| OneOrMany::Many(v.into_iter()))
}

/// Either an iterator or a single value.
enum OneOrMany<I> {
    One(Option<IpAddr>),
    Many(I),
}

impl<I: Iterator<Item = IpAddr>> Iterator for OneOrMany<I> {
    type Item = IpAddr;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            OneOrMany::One(v) => v.take(),
            OneOrMany::Many(v) => v.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            OneOrMany::One(v) => (v.is_some() as usize, Some(v.is_some() as usize)),
            OneOrMany::Many(v) => v.size_hint(),
        }
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        match self {
            OneOrMany::One(v) => {
                if let Some(v) = v {
                    f(init, v)
                } else {
                    init
                }
            }
            OneOrMany::Many(v) => v.fold(init, f),
        }
    }
}

impl<I: FusedIterator<Item = IpAddr>> FusedIterator for OneOrMany<I> {}

impl<I: ExactSizeIterator<Item = IpAddr>> ExactSizeIterator for OneOrMany<I> {}

impl<I: DoubleEndedIterator<Item = IpAddr>> DoubleEndedIterator for OneOrMany<I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            OneOrMany::One(v) => v.take(),
            OneOrMany::Many(v) => v.next_back(),
        }
    }
}

fn _assert_threadsafe() {
    fn _assertion<F: Send + Sync>(_: F) {}
    _assertion(lookup("foobar"));
}
