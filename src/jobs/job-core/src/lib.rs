//! The pieces of job-folder's menu that are drawing rather than queueing: the
//! menu bar icon, the job rows, local-time stamps, and reading progress out of
//! what a job prints.

pub mod clock;
pub mod icon;
pub mod progress;
pub mod row;

pub use row::{JobRow, Kind, Progress, RowSpec};
