//! A small bounded HTTP probe. Reachable means this endpoint answered, not that
//! every internet service works. DNS resolution has a separate five-second cap.
use std::{
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};
const TIMEOUT: Duration = Duration::from_secs(5);
const LIMIT: usize = 64 * 1024;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProbeStatus {
    Unknown,
    Reachable { latency: Duration },
    Captive { host: Option<String> },
    DnsFailure,
    ConnectFailure,
    ReadFailure,
    UnexpectedResponse,
}
impl ProbeStatus {
    pub fn is_failure(&self) -> bool {
        matches!(
            self,
            Self::DnsFailure | Self::ConnectFailure | Self::ReadFailure
        )
    }
}
pub fn parse(response: &[u8], latency: Duration) -> ProbeStatus {
    let Some(end) = response.windows(4).position(|w| w == b"\r\n\r\n") else {
        return ProbeStatus::UnexpectedResponse;
    };
    let Ok(header) = std::str::from_utf8(&response[..end]) else {
        return ProbeStatus::UnexpectedResponse;
    };
    let mut lines = header.split("\r\n");
    let mut status = lines.next().unwrap_or("").split_whitespace();
    if !matches!(status.next(), Some("HTTP/1.0" | "HTTP/1.1")) {
        return ProbeStatus::UnexpectedResponse;
    }
    let Some(code) = status.next().and_then(|s| s.parse::<u16>().ok()) else {
        return ProbeStatus::UnexpectedResponse;
    };
    let mut location = None;
    let mut length = None;
    for line in lines {
        let Some((key, value)) = line.split_once(':') else {
            return ProbeStatus::UnexpectedResponse;
        };
        if key.eq_ignore_ascii_case("location") {
            location = redirect_host(value.trim());
        }
        if key.eq_ignore_ascii_case("transfer-encoding") {
            return ProbeStatus::UnexpectedResponse;
        }
        if key.eq_ignore_ascii_case("content-length") {
            let Ok(n) = value.trim().parse::<usize>() else {
                return ProbeStatus::UnexpectedResponse;
            };
            if length.replace(n).is_some() {
                return ProbeStatus::UnexpectedResponse;
            }
        }
    }
    let body = &response[end + 4..];
    if length.is_some_and(|n| n != body.len()) {
        return ProbeStatus::ReadFailure;
    }
    if (300..400).contains(&code) {
        return ProbeStatus::Captive { host: location };
    }
    if code != 200 {
        return ProbeStatus::UnexpectedResponse;
    }
    let Ok(body) = std::str::from_utf8(body) else {
        return ProbeStatus::UnexpectedResponse;
    };
    // Match the expected page, not a stray "Success" in a portal's script.
    if body.trim() == "<HTML><HEAD><TITLE>Success</TITLE></HEAD><BODY>Success</BODY></HTML>" {
        ProbeStatus::Reachable { latency }
    } else {
        ProbeStatus::Captive { host: None }
    }
}
fn redirect_host(url: &str) -> Option<String> {
    let rest = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))?;
    let authority = rest.split(['/', '?', '#']).next()?;
    if authority.is_empty() || authority.contains('@') || authority.chars().any(char::is_whitespace)
    {
        None
    } else {
        Some(authority.to_owned())
    }
}
pub fn run() -> ProbeStatus {
    let start = Instant::now();
    // libc resolution is not cancellable. A timed-out resolver continues to own
    // this guard, preventing successive scheduled probes from leaking threads.
    static RESOLVING: AtomicBool = AtomicBool::new(false);
    if RESOLVING.swap(true, Ordering::AcqRel) {
        return ProbeStatus::Unknown;
    }
    let (tx, rx) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let result = ("captive.apple.com", 80)
            .to_socket_addrs()
            .map(|a| a.collect::<Vec<_>>());
        RESOLVING.store(false, Ordering::Release);
        let _ = tx.send(result);
    });
    let Ok(Ok(addresses)) = rx.recv_timeout(TIMEOUT) else {
        return ProbeStatus::DnsFailure;
    };
    let mut stream = None;
    let deadline = Instant::now() + TIMEOUT;
    for addr in addresses {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        if let Ok(s) = TcpStream::connect_timeout(&addr, remaining) {
            stream = Some(s);
            break;
        }
    }
    let Some(mut stream) = stream else {
        return ProbeStatus::ConnectFailure;
    };
    if stream.set_write_timeout(Some(TIMEOUT)).is_err() {
        return ProbeStatus::ConnectFailure;
    }
    if stream.write_all(b"GET /hotspot-detect.html HTTP/1.1\r\nHost: captive.apple.com\r\nConnection: close\r\nAccept-Encoding: identity\r\n\r\n").is_err() { return ProbeStatus::ConnectFailure; }
    let deadline = Instant::now() + TIMEOUT;
    let mut response = Vec::new();
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() || stream.set_read_timeout(Some(remaining)).is_err() {
            return ProbeStatus::ReadFailure;
        }
        let mut buf = [0u8; 4096];
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                response.extend_from_slice(&buf[..n]);
                if response.len() > LIMIT {
                    return ProbeStatus::UnexpectedResponse;
                }
            }
            Err(_) => return ProbeStatus::ReadFailure,
        }
    }
    parse(&response, start.elapsed())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn response_fixtures() {
        let latency = Duration::from_millis(18);
        assert_eq!(parse(b"HTTP/1.1 200 OK\r\n\r\n<HTML><HEAD><TITLE>Success</TITLE></HEAD><BODY>Success</BODY></HTML>\n",latency),ProbeStatus::Reachable { latency });
        assert_eq!(
            parse(
                b"HTTP/1.1 302 Found\r\nLocation: https://login.example/secret?token=abc\r\n\r\n",
                latency
            ),
            ProbeStatus::Captive {
                host: Some("login.example".into())
            }
        );
        assert_eq!(
            parse(b"HTTP/1.1 200 OK\r\n\r\nPlease sign in", latency),
            ProbeStatus::Captive { host: None }
        );
        assert_eq!(
            parse(
                b"HTTP/1.1 200 OK\r\nContent-Length: 20\r\n\r\nshort",
                latency
            ),
            ProbeStatus::ReadFailure
        );
        assert_eq!(
            parse(b"HTTP/1.1 503 Unavailable\r\n\r\n", latency),
            ProbeStatus::UnexpectedResponse
        );
        assert_eq!(
            parse(
                b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n",
                latency
            ),
            ProbeStatus::UnexpectedResponse
        );
    }
}
