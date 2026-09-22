use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    Poor,
    Fair,
    Good,
    Excellent,
}
impl Tier {
    pub fn segments(self) -> u8 {
        self as u8 + 1
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signal {
    Snr(i32),
    Rssi(i32),
}
impl Signal {
    pub fn from_readings(rssi: i32, noise: Option<i32>) -> Self {
        match noise {
            Some(noise) => Self::Snr(rssi - noise),
            None => Self::Rssi(rssi),
        }
    }
    fn value(self) -> i32 {
        match self {
            Self::Snr(v) | Self::Rssi(v) => v,
        }
    }
    fn edges(self) -> [i32; 3] {
        match self {
            Self::Snr(_) => [15, 25, 40],
            Self::Rssi(_) => [-80, -70, -60],
        }
    }
    pub fn tier(self) -> Tier {
        let [a, b, c] = self.edges();
        match self.value() {
            v if v < a => Tier::Poor,
            v if v < b => Tier::Fair,
            v if v < c => Tier::Good,
            _ => Tier::Excellent,
        }
    }
    pub fn unit(self) -> &'static str {
        match self {
            Self::Snr(_) => "dB",
            Self::Rssi(_) => "dBm",
        }
    }
}
/// Bar-only hysteresis; card measurements remain immediate. Degradation waits
/// ten seconds; recovery waits fifteen. A metric change resets the baseline.
#[derive(Default)]
pub struct Hysteresis {
    current: Option<(Tier, bool)>,
    pending: Option<(Tier, Instant)>,
}
impl Hysteresis {
    pub fn update(&mut self, signal: Option<Signal>, now: Instant) -> Option<Tier> {
        let Some(signal) = signal else {
            self.current = None;
            self.pending = None;
            return None;
        };
        let is_snr = matches!(signal, Signal::Snr(_));
        let target = signal.tier();
        let Some((current, metric)) = self.current else {
            self.current = Some((target, is_snr));
            return Some(target);
        };
        if metric != is_snr {
            self.current = Some((target, is_snr));
            self.pending = None;
            return Some(target);
        }
        let edges = signal.edges();
        let crossed = if target > current {
            signal.value() >= edges[current as usize] + 3
        } else if target < current {
            signal.value() < edges[current as usize - 1] - 3
        } else {
            false
        };
        if !crossed {
            self.pending = None;
            return Some(current);
        }
        let since = match self.pending {
            Some((t, since)) if t == target => since,
            _ => {
                self.pending = Some((target, now));
                now
            }
        };
        let hold = Duration::from_secs(if target < current { 10 } else { 15 });
        if now.saturating_duration_since(since) >= hold {
            self.current = Some((target, is_snr));
            self.pending = None;
            return Some(target);
        }
        Some(current)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tier_edges_and_units() {
        for (v, t) in [
            (14, Tier::Poor),
            (15, Tier::Fair),
            (24, Tier::Fair),
            (25, Tier::Good),
            (39, Tier::Good),
            (40, Tier::Excellent),
        ] {
            assert_eq!(Signal::Snr(v).tier(), t);
        }
        let signal = Signal::from_readings(-52, Some(-93));
        assert_eq!(signal, Signal::Snr(41));
        let fallback = Signal::from_readings(-81, None);
        assert_eq!(fallback.tier(), Tier::Poor);
        assert_eq!(fallback.unit(), "dBm");
    }
    #[test]
    fn margins_hold_recovery_and_missing_values() {
        let now = Instant::now();
        let mut h = Hysteresis::default();
        assert_eq!(h.update(Some(Signal::Snr(30)), now), Some(Tier::Good));
        assert_eq!(h.update(Some(Signal::Snr(24)), now), Some(Tier::Good));
        h.update(Some(Signal::Snr(21)), now);
        assert_eq!(
            h.update(Some(Signal::Snr(21)), now + Duration::from_secs(9)),
            Some(Tier::Good)
        );
        assert_eq!(
            h.update(Some(Signal::Snr(21)), now + Duration::from_secs(10)),
            Some(Tier::Fair)
        );
        h.update(Some(Signal::Snr(28)), now + Duration::from_secs(11));
        assert_eq!(
            h.update(Some(Signal::Snr(28)), now + Duration::from_secs(25)),
            Some(Tier::Fair)
        );
        assert_eq!(
            h.update(Some(Signal::Snr(28)), now + Duration::from_secs(26)),
            Some(Tier::Good)
        );
        assert_eq!(h.update(None, now + Duration::from_secs(27)), None);
    }
}
