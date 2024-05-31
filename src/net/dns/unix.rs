//! Implementation of `lookup` for Unix systems.
//!
//! This is a port from the crate async_dns which is itself a port from musl libc.
//!

//! - We check for AAAA addresses after checking for A addresses.
//! - Instead of manually waiting for sockets to become readable, we use several sockets
//! - We use a more structured DNS protocol implementation instead of messy raw byte manipulation.
//! - The `memchr` crate is used to optimize certain operations.
//!
//! TODO: Either make it parallel or lazy, but as it stands, it is eager and sequential.

use dns_protocol::{Flags, Message, Question, ResourceRecord, ResourceType};
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};

use memchr::memmem;

use pneuma::net::{IpAddr, SocketAddr, TcpStream, UdpSocket};
use std::cmp;
use std::convert::TryInto;
use std::fmt;
use std::io;
use std::sync::Arc;
use std::time::Duration;

pub(super) fn lookup(name: &str) -> io::Result<Vec<IpAddr>> {
    // We may be able to use the /etc/hosts resolver.
    if let Some(addr) = from_hosts(name)? {
        return Ok(vec![addr]);
    }

    // Otherwise, we need to use the manual resolver.
    let resolv = ResolvConf::load()?;
    dns_with_search(name, &resolv)
}

/// Try parsing the name from the "hosts" file.
fn from_hosts(name: &str) -> io::Result<Option<IpAddr>> {
    // Open the hosts file.
    let file = File::open("/etc/hosts")?;
    let mut file = BufReader::new(file);

    // Create a searcher for the name.
    let searcher = memmem::Finder::new(name.as_bytes());

    // Search for the line in the file.
    let mut buf = String::new();

    loop {
        let n = file.read_line(&mut buf)?;

        // If we read nothing, we reached the end of the file.
        if n == 0 {
            return Ok(None);
        }

        // Pop the newline from the end, if any.
        if buf.ends_with('\n') {
            buf.pop();
        }

        // If the line has a comment, remove it.
        if let Some(n) = memchr::memchr(b'#', buf.as_bytes()) {
            buf.truncate(n);
        }

        // The "hosts" file may contain our name.
        if let Some(index) = searcher.find(buf.as_bytes()) {
            // Get the IP address at the start.
            let ip_addr = match buf[..index].split_whitespace().next() {
                Some(ip_addr) => ip_addr,
                None => continue,
            };

            // Parse the IP address.
            if let Ok(ip_addr) = ip_addr.parse() {
                return Ok(Some(ip_addr));
            }
        }

        buf.clear();
    }
}

/// Preform a DNS lookup, considering the search variable.
fn dns_with_search(mut name: &str, resolv: &ResolvConf) -> io::Result<Vec<IpAddr>> {
    // See if we should just use global scope.
    let num_dots = memchr::Memchr::new(b'.', name.as_bytes()).count();
    let global_scope = num_dots >= resolv.ndots as usize || name.ends_with('.');

    // Remove the dots from the end of `name`, if needed.
    if name.ends_with('.') {
        name = &name[..name.len() - 1];

        // Raise an error if name still ends with a dot.
        if name.ends_with('.') {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "name ends with a dot",
            ));
        }
    }

    if global_scope {
        if let Some(search) = resolv.search.as_ref() {
            // Try the name with the search domains.
            let mut buffer = String::from(name);
            buffer.push('.');
            let name_end = buffer.len();

            // Try the name with the search domains.
            for domain in search.split_whitespace() {
                buffer.truncate(name_end);
                buffer.push_str(domain);

                if let Ok(addrs) = dns_lookup(&buffer, resolv) {
                    if !addrs.is_empty() {
                        return Ok(addrs);
                    }
                }
            }
        }
    }

    // Preform a DNS search on just the name.
    dns_lookup(name, resolv)
}

/// Preform a manual lookup for the name.
fn dns_lookup(name: &str, resolv: &ResolvConf) -> io::Result<Vec<IpAddr>> {
    match resolv.name_servers.len() {
        0 => {
            // No nameservers, so we can't do anything.
            Ok(vec![])
        }
        1 => {
            // Just poll the one nameserver.
            let addr = resolv.name_servers[0];
            query_name_and_nameserver(name, addr, resolv)
        }
        _ => {
            let mut info = Vec::with_capacity(resolv.name_servers.len());
            let name = Arc::<str>::from(name.to_string().into_boxed_str());
            let resolv = Arc::new(resolv.clone());

            for ns in resolv.name_servers.iter().copied() {
                let name = name.clone();
                let resolv = resolv.clone();

                info.append(&mut query_name_and_nameserver(&name, ns, &resolv)?);
            }

            Ok(info)
        }
    }
}

/// Poll for the name on the given nameserver.
fn query_name_and_nameserver(
    name: &str,
    nameserver: IpAddr,
    resolv: &ResolvConf,
) -> io::Result<Vec<IpAddr>> {
    // Try to poll for an IPv4 address first.
    let mut addrs =
        query_question_and_nameserver(Question::new(name, ResourceType::A, 1), nameserver, resolv)?;

    // If we didn't get any addresses, try an IPv6 address.
    if addrs.is_empty() {
        addrs = query_question_and_nameserver(
            Question::new(name, ResourceType::AAAA, 1),
            nameserver,
            resolv,
        )?;
    }

    Ok(addrs)
}

/// Poll for a DNS response on the given nameserver.
fn query_question_and_nameserver(
    question: Question<'_>,
    nameserver: IpAddr,
    resolv: &ResolvConf,
) -> io::Result<Vec<IpAddr>> {
    // Create the DNS query.
    // I'd like to use two questions at once, but at least the DNS system I use just drops the packet.
    let id = fastrand::u16(..);
    let mut questions = [question];
    let message = Message::new(
        id,
        Flags::standard_query(),
        &mut questions,
        &mut [],
        &mut [],
        &mut [],
    );

    // Serialize it to a buffer.
    let mut stack_buffer = [0; 512];
    let mut heap_buffer;
    let needed = message.space_needed();

    // Use the stack if we can, but switch to the heap if it's not enough.
    let buf = if needed > stack_buffer.len() {
        heap_buffer = vec![0; needed];
        heap_buffer.as_mut_slice()
    } else {
        &mut stack_buffer[..]
    };

    let len = message
        .write(buf)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, ErrWrap(err)))?;

    // The query may be too large, so we need to use TCP.
    if len <= 512 {
        if let Some(addrs) = question_with_udp(id, &buf[..len], nameserver, resolv)? {
            return Ok(addrs);
        }
    }

    // We were unable to complete the query over UDP, use TCP instead.
    question_with_tcp(id, &buf[..len], nameserver)
}

/// Query a nameserver for the given question, using the UDP protocol.
///
/// Returns `None` if the UDP query failed and TCP should be used instead.
fn question_with_udp(
    id: u16,
    query: &[u8],
    nameserver: IpAddr,
    resolv: &ResolvConf,
) -> io::Result<Option<Vec<IpAddr>>> {
    const RECORD_BUFSIZE: usize = 16;

    let mut addrs = vec![];

    // Write the query to the nameserver address.
    let mut socket = UdpSocket::bind("0.0.0.0:0")?;
    let foreign_addr = SocketAddr::new(nameserver, 53);

    // UDP queries are limited to 512 bytes.
    let mut buf = [0; 512];

    for _ in 0..resolv.attempts {
        socket.send_to(query, &foreign_addr)?;

        // Wait for `timeout` seconds for a response.
        let timeout = Duration::from_secs(resolv.timeout.into());

        socket.set_read_timeout(Some(timeout))?;

        // Get the length of the packet we're reading.
        let len = match socket.recv(&mut buf) {
            Ok(len) => len,
            Err(err) if err.kind() == ErrorKind::TimedOut => {
                // Try again. Use yield_now() to give other tasks time if we're in an executor.
                pneuma::uthread::yield_now();
                continue;
            }
            Err(err) => return Err(err),
        };

        // Buffers for DNS results.
        let mut q_buf = [Question::default(); 1];
        let mut answers = [ResourceRecord::default(); RECORD_BUFSIZE];
        let mut authority = [ResourceRecord::default(); RECORD_BUFSIZE];
        let mut additional = [ResourceRecord::default(); RECORD_BUFSIZE];

        // Parse the packet.
        let message = Message::read(
            &buf[..len],
            &mut q_buf,
            &mut answers,
            &mut authority,
            &mut additional,
        )
        .map_err(|err| io::Error::new(io::ErrorKind::Other, ErrWrap(err)))?;

        // Check the ID.
        if message.id() != id {
            // Try again.
            pneuma::uthread::yield_now();
            continue;
        }

        // If the reply was truncated, it's too large for UDP.
        if message.flags().truncated() {
            return Ok(None);
        }

        // Parse the resulting answer.
        parse_answers(&message, &mut addrs);

        // We got a response, so we're done.
        return Ok(Some(addrs));
    }

    // We did not receive a response.
    Ok(None)
}

/// Query a nameserver for the given question, using the TCP protocol.
#[cold]
fn question_with_tcp(id: u16, query: &[u8], nameserver: IpAddr) -> io::Result<Vec<IpAddr>> {
    const RECORD_BUFSIZE: usize = 16;

    if query.len() > u16::MAX as usize {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "query too large for TCP",
        ));
    }

    // Open the socket to the server.
    let mut socket = TcpStream::connect((nameserver, 53))?;

    // Write the length of the query.
    let len_bytes = (query.len() as u16).to_be_bytes();
    socket.write_all(&len_bytes)?;

    // Write the query.
    socket.write_all(query)?;

    // Read the length of the response.
    let mut len_bytes = [0; 2];
    socket.read_exact(&mut len_bytes)?;
    let len = u16::from_be_bytes(len_bytes) as usize;

    // Read the response.
    let mut stack_buffer = [0; 1024];
    let mut heap_buffer;
    let buf = if len > stack_buffer.len() {
        // Initialize the heap buffer and return a pointer to it.
        heap_buffer = vec![0; len];
        heap_buffer.as_mut_slice()
    } else {
        &mut stack_buffer
    };

    socket.read_exact(buf)?;

    // Parse the response.
    let mut q_buf = [Question::default(); 1];
    let mut answers = [ResourceRecord::default(); RECORD_BUFSIZE];
    let mut authority = [ResourceRecord::default(); RECORD_BUFSIZE];
    let mut additional = [ResourceRecord::default(); RECORD_BUFSIZE];

    let message = Message::read(
        &buf[..len],
        &mut q_buf,
        &mut answers,
        &mut authority,
        &mut additional,
    )
    .map_err(|err| io::Error::new(io::ErrorKind::Other, ErrWrap(err)))?;

    if message.id() != id {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "invalid ID in response",
        ));
    }

    // Parse the answers as address info.
    let mut addrs = vec![];
    parse_answers(&message, &mut addrs);
    Ok(addrs)
}

/// Append address information to the vector, given the DNS response.
fn parse_answers(response: &Message<'_, '_>, addrs: &mut Vec<IpAddr>) {
    addrs.extend(response.answers().iter().filter_map(|answer| {
        let data = answer.data();

        // Parse the data as an IP address.
        match data.len() {
            4 => {
                let data: [u8; 4] = data.try_into().unwrap();
                Some(IpAddr::V4(data.into()))
            }
            16 => {
                let data: [u8; 16] = data.try_into().unwrap();
                Some(IpAddr::V6(data.into()))
            }
            _ => None,
        }
    }));
}

/// Structural form of `resolv.conf`.
#[derive(Clone)]
struct ResolvConf {
    /// The list of name servers.
    name_servers: Vec<IpAddr>,

    /// Maximum number of segments in the domain name.
    ndots: u8,

    /// Maximum timeout in seconds.
    timeout: u8,

    /// Maximum number of retries.
    attempts: u8,

    /// The search domain to use.
    search: Option<String>,
}

impl ResolvConf {
    /// Load the current configuration from /etc/resolv.conf.
    fn load() -> io::Result<Self> {
        // Open the file.
        let file = File::open("/etc/resolv.conf")?;
        let mut file = BufReader::new(file);

        // Default configuration values.
        let mut config = ResolvConf {
            name_servers: vec![],
            ndots: 1,
            timeout: 5,
            attempts: 2,
            search: None,
        };

        // Begin reading lines.
        let mut buf = String::new();

        loop {
            // Read a line.
            buf.clear();
            let n = file.read_line(&mut buf)?;

            // If we read nothing, we reached the end of the file.
            if n == 0 {
                break;
            }

            // Pop the newline from the end, if any.
            if buf.ends_with('\n') {
                buf.pop();
            }

            // If there is a comment, remove it.
            if let Some(n) = memchr::memchr(b'#', buf.as_bytes()) {
                buf.truncate(n);
                if buf.is_empty() {
                    continue;
                }
            }

            if let Some(ns) = buf.strip_prefix("nameserver") {
                // Parse the IP address.
                if let Ok(ip_addr) = ns.trim().parse() {
                    config.name_servers.push(ip_addr);
                }

                continue;
            } else if let Some(options) = buf.strip_prefix("options") {
                // Try to find the options.
                if let Some(ndots_index) = memmem::find(options.as_bytes(), b"ndots:") {
                    // Parse the number of dots.
                    if let Ok(ndots) = options[ndots_index + 6..].trim().parse() {
                        config.ndots = cmp::min(ndots, 15);
                    }

                    continue;
                } else if let Some(timeout_index) = memmem::find(options.as_bytes(), b"timeout:") {
                    // Parse the timeout.
                    if let Ok(timeout) = options[timeout_index + 8..].trim().parse() {
                        config.timeout = cmp::min(timeout, 60);
                    }

                    continue;
                } else if let Some(attempts_index) = memmem::find(options.as_bytes(), b"attempts:")
                {
                    // Parse the number of attempts.
                    if let Ok(attempts) = options[attempts_index + 9..].trim().parse() {
                        config.attempts = cmp::min(attempts, 10);
                    }

                    continue;
                }
            }

            // See if we have a search domain.
            let search = match buf.strip_prefix("search") {
                Some(search) => search,
                None => match buf.strip_prefix("domain") {
                    Some(search) => search,
                    None => continue,
                },
            };

            // Parse the search domain.
            config.search = Some(search.trim().to_string());
        }

        Ok(config)
    }
}

/// Wraps `dns_protocol::Error` so that outside types can't access it, preventing `dns_protocol` from being a public dependency.
struct ErrWrap(dns_protocol::Error);

impl From<dns_protocol::Error> for ErrWrap {
    fn from(err: dns_protocol::Error) -> Self {
        ErrWrap(err)
    }
}

impl fmt::Debug for ErrWrap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for ErrWrap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl std::error::Error for ErrWrap {}
