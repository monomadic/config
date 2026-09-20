//! The job loop, as a library and a binary.
//!
//! One implementation of the contract, deliberately: the bash runner this
//! replaced was a second one, and every change had to be ported to it — the
//! ports kept lagging, and a missing heartbeat in one of them is what put a
//! false "runner not responding" in the menu bar.
//!
//! `job-monitor` deliberately does not depend on this crate. A binary with no
//! job loop linked into it cannot claim a job however it is launched, which is
//! what makes watching a shared folder from another machine safe — and it
//! loses nothing by it, since every command is a folder move.

pub mod runner;
pub mod watch;
