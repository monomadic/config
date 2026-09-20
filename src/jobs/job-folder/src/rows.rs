//! The queue as menu rows.
//!
//! Same rows as the folder-watching apps draw — `job_core::row` owns the
//! drawing, so a queue looks the same whichever app is showing it. What differs
//! is behind the buttons: every one of these is an [`Act::Call`] carrying a job
//! id and a verb back into this process, where a `SIGSTOP` or a `Vec` splice
//! answers it before the row redraws.

use job_core::row::{Act, Action, Glyph, Kind, Progress, RowSpec, ago_phrase, duration, short_duration};

use crate::queue::{Job, Phase, Queue, Verb, token};

/// What has to change before the menu is worth rebuilding rather than
/// redrawing. Everything else — the state symbol, the percentage, the clock,
/// the last line the job printed — is handed to the row already on screen.
///
/// Deliberately *not* keyed on the job's state. Pausing changes what a row
/// draws and nothing about the shape of the menu, and a pause that tore down
/// the menu to say so would take the pointer's place in it with it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Key {
    id: u64,
    buttons: usize,
    has_log: bool,
}

pub struct Row {
    pub key: Key,
    pub spec: RowSpec,
}

/// The queue, in the order it is worth reading: what is running, what is
/// waiting, then the last few outcomes.
pub fn rows(queue: &Queue, max_recent: usize) -> Vec<Row> {
    let mut rows: Vec<RowSpec> = queue
        .jobs
        .iter()
        .filter(|job| job.phase.active())
        .map(active_row)
        .collect();

    for (position, job) in queue
        .jobs
        .iter()
        .filter(|job| job.phase.waiting())
        .enumerate()
    {
        rows.push(waiting_row(job, position));
    }

    let mut finished: Vec<&Job> = queue.jobs.iter().filter(|job| job.phase.finished()).collect();
    // Newest first, whatever order they were queued in.
    finished.sort_by_key(|job| std::cmp::Reverse(job.finished));
    for job in finished.into_iter().take(max_recent) {
        rows.push(finished_row(job));
    }

    rows.into_iter()
        .map(|spec| Row {
            key: key_of(&spec),
            spec,
        })
        .collect()
}

fn key_of(spec: &RowSpec) -> Key {
    Key {
        // The id is packed into every one of a row's buttons; the first is
        // enough to identify the job the row is about.
        id: spec
            .actions
            .first()
            .and_then(|action| match action.act {
                Act::Call(token) => crate::queue::untoken(token).map(|(id, _)| id),
                _ => None,
            })
            .unwrap_or(0),
        buttons: spec.actions.len(),
        has_log: spec.log.is_some(),
    }
}

/// A job with a process behind it.
fn active_row(job: &Job) -> RowSpec {
    let paused = job.phase == Phase::Paused;
    let stopping = job.note.as_deref() == Some("stopping");
    let elapsed = job.elapsed();

    // There is no "not running" here and no "no output" either. Both are
    // inferences a folder-watcher has to make about a process it cannot see;
    // this app is the process's parent, so a job that is in the list is a job
    // that is running, and one that isn't has already become an outcome.
    let value = if stopping {
        "stopping…".to_string()
    } else if paused {
        "paused".to_string()
    } else if let Some(progress) = job.progress {
        format!("{}%", (progress * 100.0).round() as i64)
    } else {
        elapsed.map(duration).unwrap_or_default()
    };

    RowSpec::new(if paused { Kind::Paused } else { Kind::Running }, job.name.clone())
        .caption(elapsed.map(short_duration).unwrap_or_default())
        .value(value)
        .progress(match job.progress {
            Some(fraction) => Progress::Fraction(fraction),
            // A suspended job's bar shouldn't animate: a flat track says
            // stopped where stripes would say working.
            None if paused => Progress::Track,
            None => Progress::Unknown,
        })
        .log(job.last_line.clone())
        .open(job.dir.clone())
        .actions(job.dir.clone(), active_actions(job, paused))
}

fn active_actions(job: &Job, paused: bool) -> Vec<Action> {
    let mut actions = vec![
        if paused {
            Action::call(Glyph::Resume, token(job.id, Verb::Resume))
        } else {
            Action::call(Glyph::Pause, token(job.id, Verb::Pause))
        },
        Action::call(Glyph::Stop, token(job.id, Verb::Stop)),
    ];
    if let Some(log) = job.log_path() {
        actions.push(Action {
            glyph: Glyph::Log,
            act: Act::Open(log),
            back: None,
        });
    }
    actions
}

/// A job waiting for a slot, held or not. Its position in the list is its
/// position in the queue, which is the one number worth showing.
fn waiting_row(job: &Job, position: usize) -> RowSpec {
    let held = job.phase == Phase::Held;
    RowSpec::new(if held { Kind::Paused } else { Kind::Queued }, job.name.clone())
        .caption(format!("{}", position + 1))
        .value(if held {
            "held".to_string()
        } else if position == 0 {
            "next".to_string()
        } else {
            String::new()
        })
        .progress(Progress::Track)
        .reveal(job.dir.clone())
        .actions(job.dir.clone(), vec![
            // First, because it is the one that changes the queue rather than
            // the job, and the queue is what you opened this list to arrange.
            Action::call(Glyph::Top, token(job.id, Verb::Top)),
            if held {
                Action::call(Glyph::Resume, token(job.id, Verb::Resume))
            } else {
                Action::call(Glyph::Pause, token(job.id, Verb::Pause))
            },
            Action::call(Glyph::Stop, token(job.id, Verb::Stop)),
        ])
}

fn finished_row(job: &Job) -> RowSpec {
    let ok = job.phase == Phase::Finished { ok: true };
    let value = match (ok, job.note.as_deref(), job.exit) {
        (true, _, _) => job.since_finished().map(ago_phrase).unwrap_or_default(),
        // Why it failed, where there is something to say beyond the number: a
        // job that was stopped by hand didn't fail on its own account.
        (false, Some(note), _) => note.to_string(),
        (false, None, Some(code)) => format!("exit {code}"),
        (false, None, None) => "failed".to_string(),
    };

    let mut actions = vec![Action::call(Glyph::Retry, token(job.id, Verb::Retry))];
    if let Some(log) = job.log_path() {
        actions.push(Action {
            glyph: Glyph::Log,
            act: Act::Open(log),
            back: None,
        });
    }

    RowSpec::new(if ok { Kind::Done } else { Kind::Failed }, job.name.clone())
        .caption(job.elapsed().map(short_duration).unwrap_or_default())
        .value(value)
        .alert(!ok)
        // A finished job's bar is full: it ran to its end, well or badly, and
        // the red fill says which.
        .progress(Progress::Fraction(1.0))
        .reveal(job.dir.clone())
        .actions(job.dir.clone(), actions)
}
