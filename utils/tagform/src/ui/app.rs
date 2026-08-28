//! Application state and the event loop (SPEC §6.3).
//!
//! Milestone 2 is read-only: the form displays and navigates, nothing is
//! edited and nothing is written. The focus ring, the aggregate/single-file
//! split and the inspector are all here because they are what the editing
//! milestones plug into.

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use crate::config::{Enums, KINDS};
use crate::model::schema::{Control, FieldDef, FIELDS};
use crate::ui::edit::{Editor, Opt, Validation};
use crate::model::value::{Agg, Value};
use crate::tags::plan::{self, FilePlan};
use crate::tags::probe::FileTags;
use crate::tags::write;
use crate::thumb::{self, MediaInfo};

/// One line in the form. A schema field, or -- below them -- a key found on
/// disk that no field claims. Custom keys get rows of their own so that an
/// unrecognised tag is visibly present rather than quietly missing.
pub struct Row {
    /// Stable across view changes and row rebuilds, so a staged edit stays
    /// attached to its field when the selection is re-aggregated.
    pub key: String,
    pub label: String,
    pub control: Control,
    pub def: Option<&'static FieldDef>,
    pub agg: Agg,
}

impl Row {
    pub fn is_mixed(&self) -> bool {
        matches!(self.agg, Agg::Mixed { .. })
    }

    /// What the field held before editing. None for a field the selection does
    /// not agree on -- there is nothing to compare an edit against, so any
    /// assignment counts as a change.
    pub fn original(&self) -> Option<&Value> {
        self.agg.value()
    }

    pub fn editable(&self) -> bool {
        self.control != Control::ReadOnly
    }
}

/// Select mode moves and commands; Edit mode types. Keeping them apart is what
/// frees the single-letter keys -- `w` can mean write because in Select mode
/// nothing is listening for the letter w.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Select,
    Edit,
}

pub struct WriteResults {
    pub ok: Vec<PathBuf>,
    pub failed: Vec<(PathBuf, String)>,
}

pub enum Msg {
    Thumb(usize, Box<image::DynamicImage>),
    Media(usize, MediaInfo),
}

pub struct App {
    pub files: Vec<FileTags>,
    pub media: Vec<MediaInfo>,
    pub rows: Vec<Row>,
    /// How many trailing rows are unrecognised keys rather than schema fields.
    pub n_custom: usize,
    /// Keys no field claims, kept as names so their aggregate can be recomputed
    /// for whichever files are in scope -- otherwise a custom key would still
    /// read ‹multiple› while looking at a single file.
    custom_keys: Vec<String>,
    pub focus: usize,
    /// None = aggregate view over every file; Some(i) = that one file.
    pub view: Option<usize>,
    pub inspector: bool,
    pub status: String,
    pub enums: Enums,
    /// Ride the faststart flag along on any remux we are already doing. On by
    /// default, per the brief.
    pub faststart: bool,
    /// The write plan, awaiting confirmation. Nothing reaches disk until this
    /// has been shown and accepted.
    pub pending: Option<Vec<FilePlan>>,
    /// The outcome of the last write, held until dismissed.
    pub results: Option<WriteResults>,
    pub writing: bool,
    /// The live control for the focused row. Recreated whenever focus moves, so
    /// there is no separate "edit mode": the focused field is always editable
    /// and typing goes straight into it, the way a GUI form behaves.
    pub editor: Option<Editor>,
    pub mode: Mode,
    /// Edits not yet written. Nothing reaches disk in this milestone; this is
    /// the staging model mp4-tui-tagger got right, kept.
    pub staged: BTreeMap<String, Value>,
    undo: Vec<BTreeMap<String, Value>>,
    redo: Vec<BTreeMap<String, Value>>,
    pub quit: bool,
    /// Esc with staged edits asks once before discarding them.
    confirm_quit: bool,
    pub thumb_image: Option<image::DynamicImage>,
    pub thumb_for: Option<usize>,
    /// width/height of the current thumbnail, so the band can be shaped to the
    /// picture rather than the picture squeezed into a fixed band.
    pub thumb_aspect: Option<f32>,
    rx: Receiver<Msg>,
    tx: mpsc::Sender<Msg>,
}

impl App {
    pub fn new(files: Vec<FileTags>, custom: BTreeMap<String, Agg>, thumbnails: bool) -> Self {
        let rows = build_rows(&files.iter().collect::<Vec<_>>(), &custom);
        let n_custom = custom.len();
        let (tx, rx) = mpsc::channel();
        let n = files.len();
        let mut app = Self {
            media: vec![MediaInfo::default(); n],
            files,
            rows,
            n_custom,
            custom_keys: custom.keys().cloned().collect(),
            focus: 0,
            view: None,
            inspector: false,
            status: String::new(),
            enums: Enums::load(),
            faststart: true,
            pending: None,
            results: None,
            writing: false,
            editor: None,
            mode: Mode::Select,
            staged: BTreeMap::new(),
            undo: Vec::new(),
            redo: Vec::new(),
            quit: false,
            confirm_quit: false,
            thumb_image: None,
            thumb_for: None,
            thumb_aspect: None,
            rx,
            tx,
        };
        for i in 0..n {
            app.spawn_media(i);
        }
        if thumbnails {
            app.request_thumb(0);
        }
        app.open_editor();
        app
    }

    /// The file the header describes: the focused one in single view, else the
    /// first, so the band always has something to show.
    pub fn current_file(&self) -> usize {
        self.view.unwrap_or(0)
    }

    fn spawn_media(&self, idx: usize) {
        let path = self.files[idx].path.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            if let Ok(info) = thumb::probe_media(&path) {
                let _ = tx.send(Msg::Media(idx, info));
            }
        });
    }

    /// Extraction shells out to ffmpeg and can seek through a multi-gigabyte
    /// file, so it never runs on the UI thread.
    fn request_thumb(&mut self, idx: usize) {
        if self.thumb_for == Some(idx) || idx >= self.files.len() {
            return;
        }
        self.thumb_for = Some(idx);
        self.thumb_image = None;
        self.thumb_aspect = None;
        let path = self.files[idx].path.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            if let Ok(jpg) = thumb::extract(&path, 720, 720) {
                if let Ok(img) = image::ImageReader::open(&jpg).and_then(|r| Ok(r.decode())) {
                    if let Ok(img) = img {
                        let _ = tx.send(Msg::Thumb(idx, Box::new(img)));
                    }
                }
            }
        });
    }

    pub fn drain(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                Msg::Thumb(i, img) => {
                    if self.thumb_for == Some(i) {
                        use image::GenericImageView;
                        let (w, h) = img.dimensions();
                        self.thumb_aspect =
                            (w > 0 && h > 0).then(|| w as f32 / h as f32);
                        self.thumb_image = Some(*img);
                    }
                }
                Msg::Media(i, info) => {
                    if i < self.media.len() {
                        self.media[i] = info;
                    }
                }
            }
        }
    }

    /// Union the focused list field across every file in scope.
    ///
    /// Setting a ‹multiple› list field otherwise means picking one file's
    /// values and destroying the rest, which is rarely what you want when
    /// tagging a batch -- you want everyone's actors, or every tag that appears
    /// anywhere. Order is first-seen; duplicates are folded case-insensitively
    /// so "Alice" and "alice" do not both survive.
    fn merge_focused(&mut self) {
        let Some(row) = self.rows.get(self.focus) else { return };
        if !matches!(row.control, Control::List | Control::HashTags) {
            self.status = "merge applies to list fields".into();
            return;
        }
        let Agg::Mixed { values } = &row.agg else {
            self.status = "nothing to merge: the files already agree".into();
            return;
        };
        let merged = merge_values(values);
        if merged.is_empty() {
            self.status = "nothing to merge".into();
            return;
        }
        let key = row.key.clone();
        let n = merged.len();
        let before = self.staged.clone();
        self.staged.insert(key, Value::List(merged));
        self.undo.push(before);
        self.redo.clear();
        self.open_editor();
        self.status = format!("merged {n} value{} across the selection", if n == 1 { "" } else { "s" });
    }

    /// How many distinct values a staged edit is about to replace. Zero when
    /// the files already agreed; that is the difference between changing a
    /// value and flattening several.
    pub fn overwrites(&self, row: &Row) -> usize {
        if !self.staged.contains_key(&row.key) {
            return 0;
        }
        let Agg::Mixed { values } = &row.agg else { return 0 };
        let mut seen: Vec<&Value> = Vec::new();
        for v in values.iter().flatten() {
            if !seen.contains(&v) {
                seen.push(v);
            }
        }
        seen.len()
    }

    /// Build the plan for the files in scope and hold it for confirmation.
    fn prepare_write(&mut self) {
        self.commit_editor();
        if self.staged.is_empty() {
            self.status = "nothing to write".into();
            return;
        }
        let scope: Vec<usize> = match self.view {
            Some(i) => vec![i],
            None => (0..self.files.len()).collect(),
        };
        let plans: Vec<FilePlan> = scope
            .iter()
            .map(|i| plan::build(&self.files[*i], &self.staged, self.faststart))
            .filter(|p| !p.is_empty())
            .collect();
        if plans.is_empty() {
            self.status = "nothing to write".into();
            return;
        }
        self.pending = Some(plans);
    }

    /// Run the confirmed plan, then re-read from disk so the form shows what is
    /// actually on the files rather than what was hoped for.
    fn apply(&mut self) {
        let Some(plans) = self.pending.take() else { return };
        self.writing = true;
        let mut written: Vec<PathBuf> = Vec::new();
        let mut failed: Vec<(PathBuf, String)> = Vec::new();
        for p in &plans {
            let snapshot = self
                .files
                .iter()
                .find(|f| f.path == p.path)
                .map(|f| f.xmp.clone())
                .unwrap_or_default();
            match write::execute(p, &snapshot) {
                Ok(()) => written.push(p.path.clone()),
                Err(e) => failed.push((p.path.clone(), e.to_string())),
            }
        }
        // A bad file in a batch must not cost the others their write.
        for f in self.files.iter_mut() {
            if let Ok(fresh) = crate::tags::probe::probe(&f.path) {
                *f = fresh;
            }
        }
        self.staged.clear();
        self.undo.clear();
        self.redo.clear();
        self.rebuild_rows();
        self.writing = false;
        self.status = format!(
            "wrote {} of {}",
            written.len(),
            written.len() + failed.len()
        );
        self.results = Some(WriteResults { ok: written, failed });
    }

    /// Route by mode. Select moves and commands; Edit types.
    pub fn on_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }
        if self.results.is_some() {
            self.results = None;
            return;
        }
        // A dialog owns every key while it is up: a stray character must not
        // leak into a form field behind a prompt asking to write.
        if self.pending.is_some() {
            match key.code {
                KeyCode::Enter => self.apply(),
                KeyCode::Esc | KeyCode::Char('n') => {
                    self.pending = None;
                    self.status = "write cancelled".into();
                }
                _ => {}
            }
            return;
        }
        match self.mode {
            Mode::Edit => self.edit_key(key),
            Mode::Select => self.select_key(key),
        }
    }

    fn edit_key(&mut self, key: KeyEvent) {
        match key.code {
            // Commit and stop editing.
            KeyCode::Enter => {
                self.commit_editor();
                self.mode = Mode::Select;
                self.status.clear();
            }
            // Commit and carry straight on to the next field, which is what
            // tab means in every form.
            KeyCode::Tab => {
                self.move_focus(1);
            }
            KeyCode::BackTab => {
                self.move_focus(-1);
            }
            // Abandon this field's edit. Reseeding restores whatever the row
            // showed before -- the staged value if there was one, else disk.
            KeyCode::Esc => {
                self.open_editor();
                self.mode = Mode::Select;
                self.status = "edit cancelled".into();
            }
            _ => {
                if let Some(ed) = &mut self.editor {
                    ed.handle(key);
                }
                self.status.clear();
            }
        }
    }

    fn select_key(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if key.code != KeyCode::Esc && key.code != KeyCode::Char('q') {
            self.confirm_quit = false;
        }
        match (key.code, ctrl) {
            (KeyCode::Char('j'), false) | (KeyCode::Down, _) | (KeyCode::Tab, _) => {
                self.move_focus(1)
            }
            (KeyCode::Char('k'), false) | (KeyCode::Up, _) | (KeyCode::BackTab, _) => {
                self.move_focus(-1)
            }
            (KeyCode::Char('g'), false) => self.jump(0),
            (KeyCode::Char('G'), false) => self.jump(self.rows.len().saturating_sub(1)),
            (KeyCode::Enter, _) => self.begin_edit(),
            (KeyCode::Char('p'), false) => {
                self.inspector = !self.inspector;
                self.status.clear();
            }
            (KeyCode::Char(']'), false) => self.cycle_file(1),
            (KeyCode::Char('['), false) => self.cycle_file(-1),
            (KeyCode::Char('a'), false) => {
                self.view = None;
                self.rebuild_rows();
                self.status = "aggregate view".into();
            }
            (KeyCode::Char('m'), false) => self.merge_focused(),
            (KeyCode::Char('u'), false) => self.undo(),
            (KeyCode::Char('r'), true) => self.redo(),
            (KeyCode::Char('r'), false) => self.revert_all(),
            (KeyCode::Char('w'), false) => self.prepare_write(),
            (KeyCode::Char('f'), false) => {
                self.faststart = !self.faststart;
                self.status = format!("faststart {}", if self.faststart { "on" } else { "off" });
            }
            (KeyCode::Char('q'), false) | (KeyCode::Esc, _) => self.escape(),
            _ => {}
        }
    }

    fn jump(&mut self, to: usize) {
        self.focus = to.min(self.rows.len().saturating_sub(1));
        self.open_editor();
    }

    fn begin_edit(&mut self) {
        match self.rows.get(self.focus) {
            Some(row) if !row.editable() => {
                self.status = format!("{} is read-only", row.label);
            }
            Some(_) => {
                self.open_editor();
                self.mode = Mode::Edit;
                self.status.clear();
            }
            None => {}
        }
    }

    /// In Select mode there is no half-typed field to back out of -- Esc in
    /// Edit mode already handled that -- so here Esc and q mean quit. Staged
    /// edits are never discarded silently.
    fn escape(&mut self) {
        if !self.staged.is_empty() && !self.confirm_quit {
            self.confirm_quit = true;
            self.status = format!(
                "{} staged edit{} · press again to discard and quit, or w to write",
                self.staged.len(),
                if self.staged.len() == 1 { "" } else { "s" }
            );
        } else {
            self.quit = true;
        }
    }

    /// Human label for a stored enum code, so an unfocused Kind row reads
    /// "Movie" rather than the `stik` integer 9 that is actually stored.
    pub fn enum_label(&self, row: &Row, code: &str) -> Option<String> {
        let opts = self.options_for(row);
        if opts.is_empty() {
            return None;
        }
        Some(
            opts.iter()
                .find(|o| o.code == code)
                .map(|o| o.label.clone())
                .unwrap_or_else(|| code.to_string()),
        )
    }

    /// Options for the focused row's enum, if it has one.
    pub fn options_for(&self, row: &Row) -> Vec<Opt> {
        let same = |v: &Vec<String>| {
            v.iter().map(|s| Opt { code: s.clone(), label: s.clone() }).collect::<Vec<_>>()
        };
        match row.key.as_str() {
            "genre" => same(&self.enums.genre),
            "type" => same(&self.enums.type_),
            "kind" => KINDS
                .iter()
                .map(|(c, l)| Opt { code: (*c).into(), label: (*l).into() })
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Seed a control from the staged edit if there is one, else from disk.
    fn open_editor(&mut self) {
        let Some(row) = self.rows.get(self.focus) else {
            self.editor = None;
            return;
        };
        let staged = self.staged.get(&row.key).cloned();
        let value = staged.as_ref().or_else(|| row.original());
        let opts = self.options_for(row);
        self.editor = Some(Editor::new(row.control, value, opts));
    }

    /// Fold the focused control's value into the staging map.
    ///
    /// "Unchanged" means the control produces the same value the *disk state*
    /// produces through that same control -- not that the raw stored value
    /// matches. The difference matters: an absent Rating opens as ☆☆☆☆☆, whose
    /// value is "0", so comparing against the stored `None` would stage a 0 on
    /// every file merely tabbed past. Round-tripping the original through the
    /// control puts both sides in the same terms.
    fn commit_editor(&mut self) {
        let (Some(ed), Some(row)) = (&self.editor, self.rows.get(self.focus)) else { return };
        if !row.editable() {
            return;
        }
        let new = ed.value();
        let baseline =
            Editor::new(row.control, row.original(), self.options_for(row)).value();
        let key = row.key.clone();
        let before = self.staged.clone();
        if new == baseline {
            self.staged.remove(&key);
        } else {
            self.staged.insert(key, new);
        }
        if before != self.staged {
            self.undo.push(before);
            self.redo.clear();
        }
    }

    pub fn validation(&self) -> Validation {
        self.editor.as_ref().map(|e| e.validate()).unwrap_or(Validation::Ok)
    }

    pub fn is_staged(&self, key: &str) -> bool {
        self.staged.contains_key(key)
    }

    /// The value a row should display: the staged edit if any, else what is on
    /// disk.
    pub fn shown_value(&self, row: &Row) -> Option<Value> {
        self.staged.get(&row.key).cloned().or_else(|| row.original().cloned())
    }

    fn undo(&mut self) {
        if let Some(prev) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.staged, prev));
            self.open_editor();
            self.status = format!("undo · {} staged", self.staged.len());
        } else {
            self.status = "nothing to undo".into();
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.staged, next));
            self.open_editor();
            self.status = format!("redo · {} staged", self.staged.len());
        } else {
            self.status = "nothing to redo".into();
        }
    }

    fn revert_all(&mut self) {
        if self.staged.is_empty() {
            self.status = "no staged edits".into();
            return;
        }
        let n = self.staged.len();
        self.undo.push(self.staged.clone());
        self.redo.clear();
        self.staged.clear();
        self.open_editor();
        self.status = format!("reverted {n} staged edit{}", if n == 1 { "" } else { "s" });
    }

    /// In single-file view the rows describe that file alone, so a value shows
    /// as itself rather than as ‹multiple› -- the aggregate is only meaningful
    /// when more than one file is in scope.
    fn rebuild_rows(&mut self) {
        let subset: Vec<&FileTags> = match self.view {
            Some(i) => vec![&self.files[i]],
            None => self.files.iter().collect(),
        };
        let custom = custom_aggs(&subset, &self.custom_keys);
        self.rows = build_rows(&subset, &custom);
        if self.focus >= self.rows.len() {
            self.focus = self.rows.len().saturating_sub(1);
        }
        self.open_editor();
    }

    fn move_focus(&mut self, delta: isize) {
        if self.rows.is_empty() {
            return;
        }
        self.commit_editor();
        let n = self.rows.len() as isize;
        self.focus = (((self.focus as isize + delta) % n + n) % n) as usize;
        self.open_editor();
    }

    /// Stepping past either end returns to the aggregate view rather than
    /// wrapping, so there is always a way back to "all files" by walking.
    fn cycle_file(&mut self, delta: isize) {
        if self.files.len() < 2 {
            return;
        }
        self.commit_editor();
        let n = self.files.len() as isize;
        self.view = match self.view {
            None if delta > 0 => Some(0),
            None => Some((n - 1) as usize),
            Some(i) => {
                let next = i as isize + delta;
                if next < 0 || next >= n { None } else { Some(next as usize) }
            }
        };
        if let Some(i) = self.view {
            self.request_thumb(i);
        }
        self.rebuild_rows();
        self.status = match self.view {
            Some(i) => format!("file {} of {}", i + 1, self.files.len()),
            None => "aggregate view".into(),
        };
    }
}

/// Visible rows: every primary field, plus footage fields only once they hold
/// something. An absent primary field still gets a row -- seeing that Title is
/// empty is the point of a form.
/// Aggregate the unclaimed keys over just the files in scope.
fn custom_aggs(files: &[&FileTags], keys: &[String]) -> BTreeMap<String, Agg> {
    keys.iter()
        .map(|k| {
            let per_file = files.iter().map(|t| t.atoms.get(k).cloned()).collect();
            (k.clone(), Agg::fold(per_file))
        })
        .collect()
}

fn build_rows(files: &[&FileTags], custom: &BTreeMap<String, Agg>) -> Vec<Row> {
    let mut rows: Vec<Row> = FIELDS
        .iter()
        .filter_map(|def| {
            let agg = Agg::fold(files.iter().map(|t| t.lookup(def)).collect());
            if def.footage_only && matches!(agg, Agg::Absent) {
                return None;
            }
            Some(Row {
                key: def.id.to_string(),
                label: def.label.to_string(),
                control: def.control,
                def: Some(def),
                agg,
            })
        })
        .collect();
    rows.extend(custom.iter().map(|(k, agg)| Row {
        key: format!("custom:{k}"),
        label: k.clone(),
        control: Control::Text,
        def: None,
        agg: agg.clone(),
    }));
    rows
}

/// Pick an image backend, querying the terminal only where a reply is plausible.
///
/// `Picker::from_query_stdio()` spawns a thread that blocks reading stdin for a
/// capability response. On a terminal that never answers, the call times out
/// after 2 s -- but that thread stays parked on the read, and then competes with
/// the event loop for keypresses and silently eats them. Driving the app through
/// a plain pty lost roughly half of them that way, which looks like a broken
/// keymap rather than a stuck probe.
///
/// So the query is only issued to terminals that plausibly implement a graphics
/// protocol. Everything else goes straight to halfblocks, which needs no query,
/// spawns no thread, and still draws a picture.
fn make_picker(no_thumbnail: bool) -> ratatui_image::picker::Picker {
    use ratatui_image::picker::Picker;
    if no_thumbnail {
        return Picker::halfblocks();
    }
    let env = |k: &str| std::env::var(k).unwrap_or_default();
    let term = env("TERM").to_ascii_lowercase();
    let program = env("TERM_PROGRAM").to_ascii_lowercase();
    let graphical = !env("KITTY_WINDOW_ID").is_empty()
        || !env("WEZTERM_EXECUTABLE").is_empty()
        || term.contains("kitty")
        || term.contains("ghostty")
        || matches!(program.as_str(), "iterm.app" | "wezterm" | "ghostty" | "kitty");
    if graphical {
        Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks())
    } else {
        Picker::halfblocks()
    }
}

pub fn run(files: Vec<FileTags>, custom: BTreeMap<String, Agg>, no_thumbnail: bool) -> Result<()> {
    let mut terminal = ratatui::init();
    let picker = make_picker(no_thumbnail);

    let mut app = App::new(files, custom, !no_thumbnail);
    let mut proto: Option<ratatui_image::protocol::StatefulProtocol> = None;
    let mut proto_for: Option<usize> = None;

    let res = (|| -> Result<()> {
        loop {
            app.drain();
            // Rebuild the image protocol only when the thumbnail actually
            // changed; doing it per frame would re-encode on every redraw.
            if let (Some(img), Some(idx)) = (&app.thumb_image, app.thumb_for) {
                if proto_for != Some(idx) {
                    proto = Some(picker.new_resize_protocol(img.clone()));
                    proto_for = Some(idx);
                }
            }
            terminal.draw(|f| render::draw(f, &app, proto.as_mut()))?;

            if event::poll(Duration::from_millis(250))? {
                if let Event::Key(key) = event::read()? {
                    app.on_key(key);
                }
            }
            if app.quit {
                return Ok(());
            }
        }
    })();

    ratatui::restore();
    res
}

use crate::ui::render;

/// Union of every file's values, first-seen order, folded case-insensitively.
///
/// Case folding matters because the same person arrives spelled differently
/// from different sources -- yt-dlp's `%(cast)l`, a hand-typed filename, an XMP
/// list -- and a merge that kept "Alice" and "alice" would make the batch worse
/// rather than better.
pub fn merge_values(per_file: &[Option<Value>]) -> Vec<String> {
    let mut merged: Vec<String> = Vec::new();
    for v in per_file.iter().flatten() {
        let items = match v {
            Value::List(l) => l.clone(),
            Value::Text(t) => vec![t.clone()],
        };
        for item in items {
            let item = item.trim().to_string();
            if item.is_empty() {
                continue;
            }
            if !merged.iter().any(|m| m.eq_ignore_ascii_case(&item)) {
                merged.push(item);
            }
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    fn l(v: &[&str]) -> Option<Value> {
        Some(Value::List(v.iter().map(|s| s.to_string()).collect()))
    }

    #[test]
    fn merge_is_a_union_in_first_seen_order() {
        let got = merge_values(&[l(&["Alice", "Bob"]), l(&["Carol"]), l(&["Dave"])]);
        assert_eq!(got, vec!["Alice", "Bob", "Carol", "Dave"]);
    }

    #[test]
    fn merge_folds_case_and_keeps_the_first_spelling() {
        let got = merge_values(&[l(&["Alice", "Bob"]), l(&["bob", "Carol"])]);
        assert_eq!(got, vec!["Alice", "Bob", "Carol"]);
    }

    #[test]
    fn merge_skips_absent_files_and_blank_entries() {
        let got = merge_values(&[l(&["Alice", "  ", ""]), None, l(&["Bob"])]);
        assert_eq!(got, vec!["Alice", "Bob"]);
    }

    /// An mdta atom holds a list as one comma-joined string on some files and a
    /// real list on others; the merge has to cope with both shapes.
    #[test]
    fn merge_accepts_a_scalar_alongside_lists() {
        let got = merge_values(&[Some(Value::text("Solo")), l(&["Duo"])]);
        assert_eq!(got, vec!["Solo", "Duo"]);
    }

    #[test]
    fn merging_nothing_yields_nothing() {
        assert!(merge_values(&[None, None]).is_empty());
    }
}
