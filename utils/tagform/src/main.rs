//! tagform — a form-based metadata tagger for MP4/MOV.
//!
//! Milestone 1: probe -> model -> aggregate -> --print-json. No UI, no writes.
//! See SPEC.md for the design and docs/CONTAINER.md for the measured container
//! behaviour the design rests on.

mod model;
mod tags;

use anyhow::{bail, Result};
use std::collections::BTreeMap;
use std::path::PathBuf;

use model::schema::{claimed_atom_keys, FIELDS};
use model::value::{Agg, Value};
use tags::probe::{probe, FileTags};

#[derive(serde::Serialize)]
struct Report {
    files: Vec<String>,
    fields: BTreeMap<String, FieldReport>,
    /// Keys found on disk that no field claims. Never dropped — losing an
    /// unrecognised tag by failing to recognise it is the bug this guards.
    custom: BTreeMap<String, Agg>,
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

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--print-json" => print_json = true,
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(());
            }
            a if a.starts_with('-') => bail!("unknown option: {a}"),
            a => paths.push(PathBuf::from(a)),
        }
    }

    if paths.is_empty() {
        bail!("no files given\n\n{USAGE}");
    }
    if !print_json {
        // The TUI is milestone 2. Say so plainly rather than doing something
        // surprising with the argument.
        bail!("the interactive form is not built yet (milestone 2); use --print-json");
    }

    let files: Vec<FileTags> = paths.iter().map(|p| probe(p)).collect::<Result<_>>()?;
    let report = build_report(&files);
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn build_report(files: &[FileTags]) -> Report {
    let mut fields = BTreeMap::new();
    let mut disputes = Vec::new();

    for f in FIELDS {
        let per_file: Vec<Option<Value>> = files.iter().map(|t| t.lookup(f)).collect();
        let agg = Agg::fold(per_file);
        if matches!(agg, Agg::Absent) && f.footage_only {
            continue; // footage fields stay hidden until they hold something
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

    let claimed = claimed_atom_keys();
    let mut custom_keys: Vec<String> = files
        .iter()
        .flat_map(|t| t.atoms.keys())
        .filter(|k| !claimed.contains(&k.as_str()))
        .cloned()
        .collect();
    custom_keys.sort_unstable();
    custom_keys.dedup();

    let custom = custom_keys
        .into_iter()
        .map(|k| {
            let per_file = files.iter().map(|t| t.atoms.get(&k).cloned()).collect();
            (k, Agg::fold(per_file))
        })
        .collect();

    Report {
        files: files.iter().map(|t| t.path.display().to_string()).collect(),
        fields,
        custom,
        disputes,
    }
}

const USAGE: &str = "\
Usage: tagform [OPTIONS] FILE...

  --print-json   dump the aggregated tag model and exit
  -h, --help     show this message

Milestone 1: read-only. The interactive form is milestone 2.";
