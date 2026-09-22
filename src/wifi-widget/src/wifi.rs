//! Read-only CoreWLAN adapter. No joins or power changes.
use objc2_core_wlan::{CWChannelBand, CWChannelWidth, CWPHYMode, CWSecurity, CWWiFiClient};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Band {
    Ghz2,
    Ghz5,
    Ghz6,
}
impl Band {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ghz2 => "2.4GHz",
            Self::Ghz5 => "5GHz",
            Self::Ghz6 => "6GHz",
        }
    }
}
pub fn band(value: CWChannelBand) -> Option<Band> {
    match value {
        CWChannelBand::Band2GHz => Some(Band::Ghz2),
        CWChannelBand::Band5GHz => Some(Band::Ghz5),
        CWChannelBand::Band6GHz => Some(Band::Ghz6),
        _ => None,
    }
}
pub fn width(value: CWChannelWidth) -> Option<u16> {
    match value {
        CWChannelWidth::Width20MHz => Some(20),
        CWChannelWidth::Width40MHz => Some(40),
        CWChannelWidth::Width80MHz => Some(80),
        CWChannelWidth::Width160MHz => Some(160),
        _ => None,
    }
}
pub fn phy(value: CWPHYMode, band: Option<Band>) -> &'static str {
    match value {
        CWPHYMode::Mode11n => "Wi-Fi 4",
        CWPHYMode::Mode11ac => "Wi-Fi 5",
        CWPHYMode::Mode11ax if band == Some(Band::Ghz6) => "Wi-Fi 6E",
        CWPHYMode::Mode11ax => "Wi-Fi 6",
        _ => "Wi-Fi",
    }
}
#[derive(Clone, Debug, Default)]
pub struct LinkReading {
    pub interface: Option<String>,
    pub power: bool,
    pub associated: bool,
    pub ssid: Option<String>,
    pub bssid: Option<String>,
    pub mac: Option<String>,
    pub rssi: Option<i32>,
    pub noise: Option<i32>,
    pub rate_mbps: Option<f64>,
    pub channel: Option<u16>,
    pub band: Option<Band>,
    pub width_mhz: Option<u16>,
    pub phy: Option<&'static str>,
}
fn dbm(value: isize) -> Option<i32> {
    // CoreWLAN reports 0 for unavailable/dropped measurements.
    (-127..0).contains(&value).then_some(value as i32)
}
pub fn read() -> Option<LinkReading> {
    objc2::rc::autoreleasepool(|_| {
        // SAFETY: framework getters on retained objects; all values copied before
        // the pool drains. No Objective-C object crosses a thread boundary.
        unsafe {
            let client = CWWiFiClient::sharedWiFiClient();
            let interface = client.interface()?;
            let channel = interface.wlanChannel();
            let mode = interface.activePHYMode();
            let band = channel.as_ref().and_then(|c| band(c.channelBand()));
            let power = interface.powerOn();
            let ssid = interface.ssid().map(|s| s.to_string());
            let rssi = dbm(interface.rssiValue());
            // A dropped RSSI must not turn an otherwise active link into a
            // disconnection. PHY is available even when Location hides identity.
            let associated =
                power && (mode != CWPHYMode::ModeNone || ssid.is_some() || rssi.is_some());
            let rate = interface.transmitRate();
            Some(LinkReading {
                interface: interface.interfaceName().map(|s| s.to_string()),
                power,
                associated,
                ssid,
                bssid: interface.bssid().map(|s| s.to_string()),
                mac: interface.hardwareAddress().map(|s| s.to_string()),
                rssi,
                noise: dbm(interface.noiseMeasurement()),
                rate_mbps: (associated && rate.is_finite() && rate > 0.0).then_some(rate),
                channel: channel
                    .as_ref()
                    .and_then(|c| u16::try_from(c.channelNumber()).ok()),
                band,
                width_mhz: channel.as_ref().and_then(|c| width(c.channelWidth())),
                phy: associated.then(|| phy(mode, band)),
            })
        }
    })
}
/// Saved names do not require Location access. This is not a scan and provides
/// no evidence about availability, signal, band or whether a network is open.
pub fn known_networks() -> Vec<String> {
    objc2::rc::autoreleasepool(|_| unsafe {
        let Some(interface) = CWWiFiClient::sharedWiFiClient().interface() else {
            return Vec::new();
        };
        let Some(configuration) = interface.configuration() else {
            return Vec::new();
        };
        let profiles = configuration.networkProfiles();
        (0..profiles.count())
            .filter_map(|index| {
                profiles
                    .objectAtIndex(index)
                    .ssid()
                    .map(|name| name.to_string())
            })
            .collect()
    })
}

fn security_label(supports: impl Fn(CWSecurity) -> bool) -> &'static str {
    if supports(CWSecurity::WPA3Transition)
        || (supports(CWSecurity::WPA2Personal) && supports(CWSecurity::WPA3Personal))
    {
        return "WPA2/3";
    }
    if supports(CWSecurity::WPA2Enterprise) && supports(CWSecurity::WPA3Enterprise) {
        return "WPA2/3 Ent";
    }
    for (kind, label) in [
        (CWSecurity::WPA3Enterprise, "WPA3 Ent"),
        (CWSecurity::WPA3Personal, "WPA3"),
        (CWSecurity::OWETransition, "OWE/Open"),
        (CWSecurity::OWE, "OWE"),
        (CWSecurity::WPAEnterpriseMixed, "WPA/2 Ent"),
        (CWSecurity::WPAPersonalMixed, "WPA/2"),
        (CWSecurity::WPA2Enterprise, "WPA2 Ent"),
        (CWSecurity::WPA2Personal, "WPA2"),
        (CWSecurity::WPAEnterprise, "WPA Ent"),
        (CWSecurity::WPAPersonal, "WPA"),
        (CWSecurity::DynamicWEP, "WEP Ent"),
        (CWSecurity::WEP, "WEP"),
        (CWSecurity::None, "Open"),
    ] {
        if supports(kind) {
            return label;
        }
    }
    "Unknown"
}
type ScannedNetwork = (String, Band, i32, &'static str);
/// Single-flight, user-triggered scan; no AppKit objects cross threads.
#[derive(Default)]
pub struct Scan {
    pending: Option<std::sync::mpsc::Receiver<(Vec<ScannedNetwork>, bool)>>,
    results: Vec<ScannedNetwork>,
    updated: Option<std::time::Instant>,
    cached: bool,
}
impl Scan {
    pub fn start_if_stale(&mut self) {
        if self
            .updated
            .is_none_or(|t| t.elapsed() > std::time::Duration::from_secs(180))
        {
            self.start();
        }
    }
    pub fn start(&mut self) {
        if self.pending.is_some() {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        self.pending = Some(rx);
        std::thread::spawn(move || {
            let results = objc2::rc::autoreleasepool(|_| unsafe {
                let Some(interface) = CWWiFiClient::new().interface() else {
                    return Vec::new();
                };
                if let Some(networks) = interface.cachedScanResults() {
                    let cached = networks
                        .iter()
                        .filter_map(|network| {
                            Some((
                                network.ssid()?.to_string(),
                                band(network.wlanChannel()?.channelBand())?,
                                dbm(network.rssiValue())?,
                                security_label(|kind| network.supportsSecurity(kind)),
                            ))
                        })
                        .collect();
                    let _ = tx.send((cached, false));
                }
                let Ok(networks) = interface.scanForNetworksWithName_error(None) else {
                    return Vec::new();
                };
                networks
                    .iter()
                    .filter_map(|network| {
                        Some((
                            network.ssid()?.to_string(),
                            band(network.wlanChannel()?.channelBand())?,
                            dbm(network.rssiValue())?,
                            security_label(|kind| network.supportsSecurity(kind)),
                        ))
                    })
                    .collect()
            });
            let _ = tx.send((results, true));
        });
    }
    pub fn tick(&mut self) {
        if let Some(rx) = &self.pending {
            match rx.try_recv() {
                Ok((results, finished)) => {
                    if finished || self.results.is_empty() {
                        self.results = results;
                        self.updated = Some(std::time::Instant::now());
                        self.cached = !finished;
                    }
                    if finished {
                        self.pending = None;
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.results.clear();
                    self.pending = None;
                }
                _ => {}
            }
        }
    }
    pub fn visible_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self
            .results
            .iter()
            .filter(|r| !r.0.is_empty() && self.reading(&r.0).is_some())
            .map(|r| r.0.clone())
            .collect();
        names.sort();
        names.dedup();
        names.sort_by_key(|name| std::cmp::Reverse(self.reading(name).map(|r| r.1)));
        names
    }
    pub fn is_cached(&self) -> bool {
        self.cached
    }
    pub fn busy(&self) -> bool {
        self.pending.is_some()
    }
    pub fn security(&self, name: &str) -> Option<&'static str> {
        self.reading(name)?;
        self.results
            .iter()
            .filter(|r| r.0 == name)
            .max_by_key(|r| r.2)
            .map(|r| r.3)
    }
    pub fn reading(&self, name: &str) -> Option<(Band, i32)> {
        if self.updated?.elapsed() > std::time::Duration::from_secs(180) {
            return None;
        }
        self.results
            .iter()
            .filter(|r| r.0 == name)
            .max_by_key(|r| r.2)
            .map(|r| (r.1, r.2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cached_results_arrive_before_scan_completion() {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut scan = Scan {
            pending: Some(rx),
            ..Scan::default()
        };
        tx.send((vec![("Cached".into(), Band::Ghz5, -60, "WPA2")], false))
            .unwrap();
        scan.tick();
        assert!(scan.busy());
        assert!(scan.is_cached());
        assert_eq!(scan.visible_names(), vec!["Cached"]);
        tx.send((vec![("Fresh".into(), Band::Ghz6, -50, "WPA3")], true))
            .unwrap();
        scan.tick();
        assert!(!scan.busy());
        assert!(!scan.is_cached());
        assert_eq!(scan.visible_names(), vec!["Fresh"]);
    }
    #[test]
    fn scan_uses_strongest_matching_access_point_and_expires() {
        let mut scan = Scan {
            pending: None,
            cached: false,
            results: vec![
                ("Home".into(), Band::Ghz5, -70, "WPA2"),
                ("Home".into(), Band::Ghz6, -50, "WPA3"),
            ],
            updated: Some(std::time::Instant::now()),
        };
        assert_eq!(scan.reading("Home"), Some((Band::Ghz6, -50)));
        assert_eq!(scan.reading("Other"), None);
        assert_eq!(scan.visible_names(), vec!["Home"]);
        assert_eq!(scan.security("Home"), Some("WPA3"));
        scan.updated = Some(std::time::Instant::now() - std::time::Duration::from_secs(120));
        assert_eq!(scan.visible_names(), vec!["Home"]);
        scan.start_if_stale();
        assert!(!scan.busy());
        scan.updated = Some(std::time::Instant::now() - std::time::Duration::from_secs(181));
        assert_eq!(scan.reading("Home"), None);
        assert!(scan.visible_names().is_empty());
        assert_eq!(scan.security("Home"), None);
    }
    #[test]
    fn security_labels_preserve_mixed_and_unknown_modes() {
        assert_eq!(
            security_label(|s| s == CWSecurity::WPA3Transition),
            "WPA2/3"
        );
        assert_eq!(security_label(|s| s == CWSecurity::None), "Open");
        assert_eq!(security_label(|_| false), "Unknown");
        assert_eq!(
            security_label(|s| s == CWSecurity::WPA2Enterprise),
            "WPA2 Ent"
        );
    }
    #[test]
    fn conversions_do_not_guess_future_standards() {
        assert_eq!(band(CWChannelBand::Band2GHz).unwrap().label(), "2.4GHz");
        assert_eq!(band(CWChannelBand::Band5GHz), Some(Band::Ghz5));
        assert_eq!(band(CWChannelBand::Band6GHz), Some(Band::Ghz6));
        assert_eq!(band(CWChannelBand(99)), None);
        for (v, n) in [(1, 20), (2, 40), (3, 80), (4, 160)] {
            assert_eq!(width(CWChannelWidth(v)), Some(n));
        }
        assert_eq!(width(CWChannelWidth(99)), None);
        assert_eq!(phy(CWPHYMode::Mode11ax, Some(Band::Ghz6)), "Wi-Fi 6E");
        assert_eq!(phy(CWPHYMode::Mode11ax, Some(Band::Ghz5)), "Wi-Fi 6");
        assert_eq!(phy(CWPHYMode(99), Some(Band::Ghz6)), "Wi-Fi");
        assert_eq!(dbm(0), None);
    }
}
