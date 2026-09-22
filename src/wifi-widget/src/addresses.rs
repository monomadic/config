//! Wi-Fi service IPv4 facts from SystemConfiguration, read off the UI thread.
use std::{
    io::Write,
    process::{Command, Stdio},
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};
#[derive(Default)]
struct Cache {
    key: String,
    values: (Option<String>, Option<String>),
    updated: Option<Instant>,
    busy: bool,
}
static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();
fn query(script: &str) -> String {
    let Ok(mut child) = Command::new("/usr/sbin/scutil")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return String::new();
    };
    if let Some(mut input) = child.stdin.take() {
        let _ = input.write_all(script.as_bytes());
    }
    child
        .wait_with_output()
        .ok()
        .filter(|r| r.status.success())
        .map(|r| String::from_utf8_lossy(&r.stdout).into_owned())
        .unwrap_or_default()
}
fn parse(text: &str, interface: &str) -> (Option<String>, Option<String>) {
    let mut matches = false;
    let mut address = None;
    let mut router = None;
    let mut addresses = false;
    for line in text.lines().map(str::trim) {
        if let Some(value) = line.strip_prefix("InterfaceName : ") {
            matches = value == interface;
        }
        if line.starts_with("Addresses :") {
            addresses = true;
        }
        if addresses && let Some(value) = line.strip_prefix("0 : ") {
            address = value
                .parse::<std::net::Ipv4Addr>()
                .ok()
                .map(|v| v.to_string());
        }
        if line == "}" {
            addresses = false;
        }
        if let Some(value) = line.strip_prefix("Router : ") {
            router = value
                .parse::<std::net::Ipv4Addr>()
                .ok()
                .map(|v| v.to_string());
        }
    }
    if matches {
        (address, router)
    } else {
        (None, None)
    }
}
pub fn read(interface: Option<&str>, association: &str) -> (Option<String>, Option<String>) {
    let Some(interface) = interface else {
        return (None, None);
    };
    let key = format!("{interface}/{association}");
    let mut cache = CACHE
        .get_or_init(|| Mutex::new(Cache::default()))
        .lock()
        .unwrap();
    if cache.key != key {
        cache.key = key.clone();
        cache.values = (None, None);
        cache.updated = None;
    }
    if !cache.busy
        && cache
            .updated
            .is_none_or(|t| t.elapsed() > Duration::from_secs(5))
    {
        cache.busy = true;
        let interface = interface.to_owned();
        std::thread::spawn(move || {
            let list = query("list State:/Network/Service/.*/IPv4\nquit\n");
            let mut values = (None, None);
            for line in list.lines() {
                if let Some((_, path)) = line.split_once(" = ") {
                    let found = parse(&query(&format!("show {}\nquit\n", path.trim())), &interface);
                    if found.0.is_some() {
                        values = found;
                        break;
                    }
                }
            }
            let mut cache = CACHE.get().unwrap().lock().unwrap();
            if cache.key == key {
                cache.values = values;
                cache.updated = Some(Instant::now());
            }
            cache.busy = false;
        });
    }
    cache.values.clone()
}
#[cfg(test)]
mod tests {
    #[test]
    fn service_is_scoped_to_wifi() {
        let s =
            "Addresses : <array> {\n0 : 192.168.1.12\n}\nInterfaceName : en0\nRouter : 192.168.1.1";
        assert_eq!(
            super::parse(s, "en0"),
            (Some("192.168.1.12".into()), Some("192.168.1.1".into()))
        );
        assert_eq!(super::parse(s, "utun0"), (None, None));
    }
}
