//! Shared guts of the job folder tools.
//!
//! The jobs folder itself is the API: `job-daemon` runs jobs, and `job-monitor`
//! shows them — the local queue, or one mounted from another machine. Both
//! agree on the same on-disk layout, so everything that reads it — the naming
//! rules, the observer that turns a folder into a [`Snapshot`], the rows and
//! the menu bar icon — lives here rather than in either of them.
//!
//! Nothing in this crate ever claims, moves or runs a job. Reading a jobs
//! folder is always safe, from any machine, however many readers there are.

pub mod clock;
pub mod icon;
pub mod observe;
pub mod row;

pub use observe::{Observer, Outcome, Root, Run, Snapshot, State};
pub use row::{JobRow, Kind, Progress, RowSpec, Section};
