//! tagform — a form-based metadata tagger for MP4/MOV.
//!
//! Milestone 3: probe -> model -> aggregate -> editable form with validation
//! and undo. Edits are staged in memory; nothing is written to disk yet.
//! See SPEC.md for the design and docs/CONTAINER.md for the measured container
//! behaviour the design rests on.

mod config;
mod model;
mod tags;
mod thumb;
mod ui;

use anyhow::{bail, Result};
use std::collections::BTreeMap;
use std::path::PathBuf;

use model::schema::{claimed_atom_keys, claimed_xmp_tags, FIELDS};
use model::value::{Agg, Value};
use tags::probe::{probe, FileTags};

#[derive(serde::Serialize)]
struct Report {
    files: Vec<String>,
    fields: BTreeMap<String, FieldReport>,
    /// Keys found on disk that no field claims. Never dropped — losing an
    /// unrecognised tag by failing to recognise it is the bug this guards.
    custom: BTreeMap<String, Agg>,
    /// Fields these files carry that have no iTunes atom at all, so they are
    /// exactly what `--compat ilst` would silently drop. Measured, not guessed:
    /// the default `.mp4` path keeps 11 of 20 keys (docs/CONTAINER.md §1.1).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    ilst_lossy: Vec<&'static str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    disputes: Vec<Dispute>,
}

#[derive(serde::Serialize)]
struct FieldReport {
    label: &'static str,
    control: model::schema::Control,
    #[serde(flatten)]
    agg: Agg,
}

#[derive(serde::Serialize)]
struct Dispute {
    file: String,
    field: &'static str,
    xmp: Value,
    atom: Value,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("tagform: {e:#}");
        std::process::exit(2);
    }
}

fn run() -> Result<()> {
    let mut paths: Vec<PathBuf> = Vec::new();
    let mut print_json = false;
    let mut no_thumbnail = false;
    let mut theme: Option<String> = None;

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--print-json" => print_json = true,
            "--no-thumbnail" => no_thumbnail = true,
            a if a.starts_with("--theme=") => theme = Some(a[8..].to_string()),
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(());
            }
            a if a.starts_with('-') => bail!("unknown option: {a}"),
            a => paths.push(PathBuf::from(a)),
        }
    }

    if let Some(name) = &theme {
        if !ui::theme::set_by_name(name) {
            bail!("unknown theme {name:?}; try one of: {}", ui::theme::names().join(", "));
        }
    }

    if paths.is_empty() {
        bail!("no files given\n\n{USAGE}");
    }
    let files: Vec<FileTags> = paths.iter().map(|p| probe(p)).collect::<Result<_>>()?;

    if print_json {
        let report = build_report(&files);
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    let custom = custom_keys(&files);
    ui::app::run(files, custom, no_thumbnail)
}

fn build_report(files: &[FileTags]) -> Report {
    let mut fields = BTreeMap::new();
    let mut disputes = Vec::new();
    let mut ilst_lossy = Vec::new();

    for f in FIELDS {
        let per_file: Vec<Option<Value>> = files.iter().map(|t| t.lookup(f)).collect();
        let agg = Agg::fold(per_file);
        if matches!(agg, Agg::Absent) && f.footage_only {
            continue; // footage fields stay hidden until they hold something
        }
        if !matches!(agg, Agg::Absent) && f.ilst.is_none() {
            ilst_lossy.push(f.id);
        }
        fields.insert(
            f.id.to_string(),
            FieldReport { label: f.label, control: f.control, agg },
        );
        for t in files {
            if let Some((xmp, atom)) = t.disputes(f) {
                disputes.push(Dispute {
                    file: t.path.display().to_string(),
                    field: f.id,
                    xmp,
                    atom,
                });
            }
        }
    }

    let custom = custom_keys(files);

    Report {
        files: files.iter().map(|t| t.path.display().to_string()).collect(),
        fields,
        custom,
        ilst_lossy,
        disputes,
    }
}

/// Keys on disk that no field claims. Aggregated and carried through rather
/// than dropped -- losing an unrecognised tag by failing to recognise it is
/// exactly the bug this guards against.
fn custom_keys(files: &[FileTags]) -> BTreeMap<String, Agg> {
    let atoms = claimed_atom_keys();
    let xmp = claimed_xmp_tags();
    let mut keys: Vec<String> = Vec::new();
    for t in files {
        keys.extend(
            t.atoms
                .keys()
                .filter(|k| !atoms.contains(&k.as_str()))
                .map(|k| format!("custom:{k}")),
        );
        // XMP too. rename-footage keeps growing the set it writes -- the IPTC
        // location block gained a province, a country and coordinates -- and a
        // tag no field claims used to be invisible here: preserved on write,
        // but nothing on screen said it existed.
        keys.extend(
            t.xmp
                .keys()
                .filter(|k| !xmp.contains(&k.as_str()))
                .map(|k| format!("xmp:{k}")),
        );
    }
    keys.sort();
    keys.dedup();
    keys.into_iter()
        .map(|k| {
            let per_file = files
                .iter()
                .map(|t| match k.split_once(':') {
                    Some(("xmp", tag)) => t.xmp.get(tag).cloned(),
                    Some((_, key)) => t.atoms.get(key).cloned(),
                    None => None,
                })
                .collect();
            (k, Agg::fold(per_file))
        })
        .collect()
}

const USAGE: &str = "\
Usage: tagform [OPTIONS] FILE...

  --print-json     dump the aggregated tag model and exit
  --no-thumbnail   do not render a thumbnail
  --theme=NAME     colour scheme; `c` cycles them at runtime
  -h, --help       show this message

Keys — the form is modal.

  SELECT (default)
    j / k, arrows     move between fields          g / G   first / last
    enter             edit the focused field
    w                 write staged edits (shows a plan to confirm first)
    m                 merge a list field across every file in the selection
    p                 inspector: per-file values for the focused field
    ] / [             next / previous file         a       all files
    u / ctrl-r        undo / redo                  r       revert every staged edit
    c                 cycle the colour scheme
    f                 toggle MOV faststart on the write   [on]
    q / esc           quit (asks if edits are staged)

  EDIT
    (type)            edit the field               left/right  adjust a rating
    j / k, enter      choose from a fixed set (Kind), then commit it
    enter             save and stop editing
    tab / shift-tab   save and move to the next / previous field
    esc               cancel this field's edit
    ctrl-c            quit from anywhere

Edits are staged until `w`; the original is only replaced by a
result that has been read back and verified.";
