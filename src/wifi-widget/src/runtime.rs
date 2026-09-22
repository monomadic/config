//! Single-flight probe worker and association generation checks.
use crate::{
    model::Store,
    probe::{self, ProbeStatus},
};
use std::{
    sync::mpsc::{self, Receiver, TryRecvError},
    time::{Duration, Instant},
};
struct Result {
    generation: u64,
    status: ProbeStatus,
    at: Instant,
}
pub struct Probes {
    receiver: Option<Receiver<Result>>,
    due: Instant,
}
impl Probes {
    pub fn new(now: Instant) -> Self {
        Self {
            receiver: None,
            due: now,
        }
    }
    pub fn schedule(&mut self, now: Instant, delay: Duration) {
        self.due = now + delay;
    }
    pub fn busy(&self) -> bool {
        self.receiver.is_some()
    }
    pub fn tick(&mut self, store: &mut Store, associated: bool, now: Instant) {
        if let Some(receiver) = &self.receiver {
            match receiver.try_recv() {
                Ok(result) => {
                    apply(store, result);
                    self.receiver = None;
                }
                Err(TryRecvError::Disconnected) => {
                    self.receiver = None;
                }
                Err(TryRecvError::Empty) => {}
            }
        }
        if !associated || self.busy() || now < self.due {
            return;
        }
        self.due = now + Duration::from_secs(30);
        let generation = store.generation();
        let (sender, receiver) = mpsc::sync_channel(1);
        self.receiver = Some(receiver);
        std::thread::spawn(move || {
            let status = probe::run();
            let _ = sender.send(Result {
                generation,
                status,
                at: Instant::now(),
            });
        });
    }
}
fn apply(store: &mut Store, result: Result) {
    if result.generation == store.generation() {
        store.update_probe(result.status, result.at);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn old_probe_cannot_cross_an_association_or_wake() {
        let mut store = Store::default();
        let now = Instant::now();
        let old = store.generation();
        store.invalidate_link();
        apply(
            &mut store,
            Result {
                generation: old,
                status: ProbeStatus::DnsFailure,
                at: now,
            },
        );
        assert!(store.snapshot(now).probe.is_none());
        let current = store.generation();
        apply(
            &mut store,
            Result {
                generation: current,
                status: ProbeStatus::Reachable {
                    latency: Duration::ZERO,
                },
                at: now,
            },
        );
        assert!(store.snapshot(now).probe.is_some());
    }
}
