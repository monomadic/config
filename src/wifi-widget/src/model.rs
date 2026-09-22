//! Sources supply plain values; the owning thread merges them. Passing `now`
//! explicitly keeps freshness and transition tests independent of wall time.
use crate::{
    probe::ProbeStatus,
    signal::{Signal, Tier},
    wifi::LinkReading,
};
use std::time::{Duration, Instant};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Freshness {
    Fresh,
    Stale,
    Unknown,
}
#[derive(Clone, Debug)]
pub struct Sample<T> {
    pub value: T,
    pub at: Instant,
}
impl<T> Sample<T> {
    pub fn age(&self, now: Instant) -> Duration {
        now.saturating_duration_since(self.at)
    }
    pub fn freshness(&self, now: Instant) -> Freshness {
        match self.age(now).as_secs() {
            0..10 => Freshness::Fresh,
            10..20 => Freshness::Stale,
            _ => Freshness::Unknown,
        }
    }
    pub fn fresh_value(&self, now: Instant) -> Option<&T> {
        (self.freshness(now) == Freshness::Fresh).then_some(&self.value)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkHealth {
    Unavailable,
    Off,
    Disconnected,
    Associated,
    Redacted,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct PathFlags {
    pub expensive: bool,
    pub constrained: bool,
}
#[derive(Clone, Debug, Default)]
pub struct BandStatus {
    pub faster_band_visible: bool,
}
#[derive(Clone, Debug, Default)]
pub struct GatewayEvidence {
    pub address: Option<String>,
    pub arp_resolved: Option<bool>,
    pub echo_rtt: Option<Duration>,
}
#[derive(Clone, Debug)]
pub enum Confidence {
    Weak,
    Strong,
}
#[derive(Clone, Debug)]
pub struct HomeGuess {
    pub ssid: String,
    pub confidence: Confidence,
}
#[derive(Clone, Debug)]
pub struct NetworkChange {
    pub from: String,
    pub to: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Unknown,
    WifiOff,
    Disconnected,
    NoInternet,
    LoginRequired,
    Weak,
    MeteredFallback,
    BandFallback,
    Healthy,
}
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub link: LinkHealth,
    pub reading: Option<Sample<LinkReading>>,
    pub rssi: Option<Sample<i32>>,
    pub noise: Option<Sample<i32>>,
    pub signal: Option<Signal>,
    pub probe: Option<Sample<ProbeStatus>>,
    pub failures: u8,
    pub path: Option<Sample<PathFlags>>,
    pub band: BandStatus,
    pub gateway: GatewayEvidence,
    pub home: Option<HomeGuess>,
    pub change: Option<NetworkChange>,
    pub headline: State,
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            link: LinkHealth::Unavailable,
            reading: None,
            rssi: None,
            noise: None,
            signal: None,
            probe: None,
            failures: 0,
            path: None,
            band: BandStatus::default(),
            gateway: GatewayEvidence::default(),
            home: None,
            change: None,
            headline: State::Unknown,
        }
    }
}
pub fn headline_state(s: &Snapshot, now: Instant) -> State {
    headline_state_for_tier(s, now, s.signal.map(|signal| signal.tier()))
}
/// The status chip uses the held tier, while the menu reports current facts.
pub fn headline_state_for_tier(s: &Snapshot, now: Instant, tier: Option<Tier>) -> State {
    match s.link {
        LinkHealth::Off => return State::WifiOff,
        LinkHealth::Disconnected => return State::Disconnected,
        LinkHealth::Unavailable => return State::Unknown,
        _ => {}
    }
    // Probe cadence is 30 s, so it has a longer lifetime than signal samples.
    let probe = s
        .probe
        .as_ref()
        .filter(|p| p.age(now) < Duration::from_secs(65))
        .map(|p| &p.value);
    if s.failures >= 2 && probe.is_some_and(ProbeStatus::is_failure) {
        return State::NoInternet;
    }
    if matches!(probe, Some(ProbeStatus::Captive { .. })) {
        return State::LoginRequired;
    }
    if tier == Some(Tier::Poor) {
        return State::Weak;
    }
    if s.path.as_ref().is_some_and(|p| p.value.expensive) {
        return State::MeteredFallback;
    }
    if s.band.faster_band_visible {
        return State::BandFallback;
    }
    if matches!(probe, Some(ProbeStatus::Reachable { .. }))
        && tier.is_some_and(|tier| tier >= Tier::Good)
    {
        State::Healthy
    } else {
        State::Unknown
    }
}
#[derive(Default)]
pub struct Store {
    state: Snapshot,
    generation: u64,
}
impl Store {
    pub fn generation(&self) -> u64 {
        self.generation
    }
    /// Invalidate even a redacted association on an event or wake. Results from
    /// workers holding an earlier generation must be discarded.
    pub fn invalidate_link(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        let home = self.state.home.take();
        self.state = Snapshot {
            home,
            ..Default::default()
        };
    }
    pub fn update_link(&mut self, reading: Option<LinkReading>, now: Instant) {
        let changed = match (&self.state.reading, &reading) {
            (Some(old), Some(new)) => {
                old.value.interface != new.interface
                    || old.value.power != new.power
                    || old.value.associated != new.associated
                    || old.value.ssid != new.ssid
                    || old.value.bssid != new.bssid
            }
            (None, None) => false,
            _ => true,
        };
        if changed {
            self.invalidate_link();
        }
        self.state.link = match &reading {
            None => LinkHealth::Unavailable,
            Some(r) if !r.power => LinkHealth::Off,
            Some(r) if !r.associated => LinkHealth::Disconnected,
            Some(r) if r.ssid.is_none() => LinkHealth::Redacted,
            _ => LinkHealth::Associated,
        };
        if let Some(r) = &reading {
            if r.power && r.associated {
                if let Some(value) = r.rssi.filter(|v| *v < 0) {
                    self.state.rssi = Some(Sample { value, at: now });
                }
                // Missing noise invalidates the pair; do not combine a new RSSI
                // with noise measured on a previous poll.
                self.state.noise = r
                    .noise
                    .filter(|v| *v < 0)
                    .map(|value| Sample { value, at: now });
            } else {
                self.state.rssi = None;
                self.state.noise = None;
            }
        }
        self.state.reading = reading.map(|value| Sample { value, at: now });
    }
    pub fn update_probe(&mut self, value: ProbeStatus, now: Instant) {
        if self
            .state
            .probe
            .as_ref()
            .is_some_and(|p| p.age(now) >= Duration::from_secs(65))
        {
            self.state.failures = 0;
        }
        self.state.failures = if value.is_failure() {
            self.state.failures.saturating_add(1)
        } else {
            0
        };
        self.state.probe = Some(Sample { value, at: now });
    }
    pub fn update_path(&mut self, value: PathFlags, now: Instant) {
        self.state.path = Some(Sample { value, at: now });
    }
    pub fn snapshot(&self, now: Instant) -> Snapshot {
        let mut s = self.state.clone();
        let rssi = s.rssi.as_ref().and_then(|r| r.fresh_value(now)).copied();
        let noise = s.noise.as_ref().and_then(|r| r.fresh_value(now)).copied();
        s.signal = rssi.map(|r| Signal::from_readings(r, noise));
        if s.reading
            .as_ref()
            .is_some_and(|r| r.freshness(now) == Freshness::Unknown)
        {
            s.link = LinkHealth::Unavailable;
        }
        s.headline = headline_state(&s, now);
        s
    }
}
impl Snapshot {
    pub fn dump(&self, now: Instant) -> String {
        // Debug-quoted strings escape newlines and '=' cannot become a new key.
        let mut out = format!(
            "link={:?}\nheadline={:?}\nsignal={:?}\nprobe_failures={}\n",
            self.link, self.headline, self.signal, self.failures
        );
        if let Some(r) = &self.reading {
            let v = &r.value;
            out.push_str(&format!("link.age_ms={}\ninterface={:?}\nssid={:?}\nbssid={:?}\nmac={:?}\nband={:?}\nchannel={:?}\nwidth_mhz={:?}\nphy={:?}\nrate_mbps={:?}\n", r.age(now).as_millis(),v.interface,v.ssid,v.bssid,v.mac,v.band.map(|b| b.label()),v.channel,v.width_mhz,v.phy,v.rate_mbps));
        }
        for (name, sample) in [("rssi_dbm", &self.rssi), ("noise_dbm", &self.noise)] {
            if let Some(s) = sample {
                out.push_str(&format!(
                    "{name}={}\n{name}.age_ms={}\n{name}.freshness={:?}\n",
                    s.value,
                    s.age(now).as_millis(),
                    s.freshness(now)
                ));
            } else {
                out.push_str(&format!("{name}=unknown\n"));
            }
        }
        if let Some(p) = &self.probe {
            out.push_str(&format!(
                "probe={:?}\nprobe.age_ms={}\n",
                p.value,
                p.age(now).as_millis()
            ));
        }
        if let Some(p) = &self.path {
            out.push_str(&format!(
                "path.expensive={}\npath.constrained={}\npath.age_ms={}\n",
                p.value.expensive,
                p.value.constrained,
                p.age(now).as_millis()
            ));
        } else {
            out.push_str("path=unknown\n");
        }
        out.push_str(&format!(
            "band_fallback={}\ngateway={:?}\nhome={:?}\nchange={:?}\n",
            self.band.faster_band_visible, self.gateway, self.home, self.change
        ));
        out
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn link() -> LinkReading {
        LinkReading {
            power: true,
            associated: true,
            rssi: Some(-52),
            noise: Some(-93),
            ..Default::default()
        }
    }
    #[test]
    fn all_headlines_and_stale_probe() {
        let now = Instant::now();
        for (expected, health, signal, probe, failures, expensive, fallback) in [
            (
                State::WifiOff,
                LinkHealth::Off,
                None,
                ProbeStatus::Unknown,
                0,
                false,
                false,
            ),
            (
                State::Disconnected,
                LinkHealth::Disconnected,
                None,
                ProbeStatus::Unknown,
                0,
                false,
                false,
            ),
            (
                State::Unknown,
                LinkHealth::Unavailable,
                None,
                ProbeStatus::Unknown,
                0,
                false,
                false,
            ),
            (
                State::NoInternet,
                LinkHealth::Associated,
                Some(Signal::Snr(41)),
                ProbeStatus::DnsFailure,
                2,
                false,
                false,
            ),
            (
                State::LoginRequired,
                LinkHealth::Associated,
                Some(Signal::Snr(41)),
                ProbeStatus::Captive { host: None },
                0,
                false,
                false,
            ),
            (
                State::Weak,
                LinkHealth::Associated,
                Some(Signal::Snr(10)),
                ProbeStatus::Unknown,
                0,
                false,
                false,
            ),
            (
                State::MeteredFallback,
                LinkHealth::Associated,
                Some(Signal::Snr(41)),
                ProbeStatus::Unknown,
                0,
                true,
                false,
            ),
            (
                State::BandFallback,
                LinkHealth::Associated,
                Some(Signal::Snr(41)),
                ProbeStatus::Unknown,
                0,
                false,
                true,
            ),
            (
                State::Healthy,
                LinkHealth::Redacted,
                Some(Signal::Snr(41)),
                ProbeStatus::Reachable {
                    latency: Duration::ZERO,
                },
                0,
                false,
                false,
            ),
            (
                State::Unknown,
                LinkHealth::Associated,
                None,
                ProbeStatus::Reachable {
                    latency: Duration::ZERO,
                },
                0,
                false,
                false,
            ),
        ] {
            let snapshot = Snapshot {
                link: health,
                signal,
                probe: Some(Sample {
                    value: probe,
                    at: now,
                }),
                failures,
                path: Some(Sample {
                    value: PathFlags {
                        expensive,
                        constrained: true,
                    },
                    at: now,
                }),
                band: BandStatus {
                    faster_band_visible: fallback,
                },
                ..Default::default()
            };
            assert_eq!(headline_state(&snapshot, now), expected);
        }
        let mut store = Store::default();
        store.update_link(Some(link()), now);
        store.update_probe(ProbeStatus::DnsFailure, now);
        store.update_probe(ProbeStatus::DnsFailure, now + Duration::from_secs(66));
        assert_eq!(store.snapshot(now + Duration::from_secs(66)).failures, 1);
    }
    #[test]
    fn diagnostic_strings_cannot_inject_rows() {
        let now = Instant::now();
        let mut store = Store::default();
        let mut reading = link();
        reading.ssid = Some("café=home\nheadline=Healthy".into());
        store.update_link(Some(reading), now);
        let dump = store.snapshot(now).dump(now);
        assert_eq!(
            dump.lines()
                .filter(|line| line.starts_with("headline="))
                .count(),
            1
        );
        assert!(dump.contains("café=home\\nheadline=Healthy"));
    }
    #[test]
    fn dropped_samples_age_out_and_disconnect_clears() {
        let now = Instant::now();
        let mut s = Store::default();
        s.update_link(Some(link()), now);
        assert_eq!(s.snapshot(now).signal, Some(Signal::Snr(41)));
        let mut missing = link();
        missing.rssi = None;
        s.update_link(Some(missing), now + Duration::from_secs(5));
        assert_eq!(
            s.snapshot(now + Duration::from_secs(9)).signal,
            Some(Signal::Snr(41))
        );
        assert_eq!(s.snapshot(now + Duration::from_secs(10)).signal, None);
        assert_eq!(
            s.snapshot(now + Duration::from_secs(20))
                .rssi
                .unwrap()
                .freshness(now + Duration::from_secs(20)),
            Freshness::Unknown
        );
        s.update_link(Some(LinkReading::default()), now + Duration::from_secs(21));
        assert!(s.snapshot(now + Duration::from_secs(21)).rssi.is_none());
    }
    #[test]
    fn failures_require_confirmation_and_reset_on_association() {
        let now = Instant::now();
        let mut s = Store::default();
        s.update_link(Some(link()), now);
        s.update_probe(ProbeStatus::DnsFailure, now);
        assert_eq!(s.snapshot(now).headline, State::Unknown);
        s.update_probe(ProbeStatus::ReadFailure, now);
        assert_eq!(s.snapshot(now).headline, State::NoInternet);
        let mut new = link();
        new.ssid = Some("new".into());
        s.update_link(Some(new), now);
        assert_eq!(s.snapshot(now).failures, 0);
        assert!(s.snapshot(now).probe.is_none());
    }
    #[test]
    fn headline_precedence_preserves_facts() {
        let now = Instant::now();
        let mut s = Store::default();
        let mut r = link();
        r.rssi = Some(-85);
        r.noise = Some(-93);
        s.update_link(Some(r), now);
        s.update_path(
            PathFlags {
                expensive: true,
                constrained: true,
            },
            now,
        );
        s.state.band.faster_band_visible = true;
        s.update_probe(ProbeStatus::ConnectFailure, now);
        s.update_probe(ProbeStatus::ConnectFailure, now);
        let snap = s.snapshot(now);
        assert_eq!(snap.headline, State::NoInternet);
        assert!(snap.path.unwrap().value.constrained);
        assert!(snap.band.faster_band_visible);
        s.update_probe(ProbeStatus::Captive { host: None }, now);
        assert_eq!(s.snapshot(now).headline, State::LoginRequired);
        s.update_probe(
            ProbeStatus::Reachable {
                latency: Duration::ZERO,
            },
            now,
        );
        assert_eq!(s.snapshot(now).headline, State::Weak);
    }
    #[test]
    fn no_noise_is_rssi_and_unknown_is_not_healthy() {
        let now = Instant::now();
        let mut s = Store::default();
        let mut r = link();
        r.noise = None;
        s.update_link(Some(r), now);
        assert_eq!(s.snapshot(now).signal, Some(Signal::Rssi(-52)));
        assert_eq!(s.snapshot(now).headline, State::Unknown);
    }
}
