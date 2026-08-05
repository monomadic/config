//! Shared guts of the job folder tools.
//!
//! The jobs folder itself is the API: `job-server` runs jobs and shows them,
//! `job-server-cli` runs them from launchd, and `job-monitor` watches a folder
//! mounted from another machine. All three agree on the same on-disk layout,
//! so everything that reads that layout — the naming rules, the observer that
//! turns a folder into a [`Snapshot`], the menu bar icon — lives here rather
//! than in any one of them.
//!
//! Nothing in this crate ever claims, moves or runs a job. Reading a jobs
//! folder is always safe, from any machine, however many readers there are.

pub mod clock;
pub mod icon;
pub mod observe;

pub use observe::{Observer, Outcome, Root, Run, Snapshot};
