//! The editable controls (SPEC §5).
//!
//! Every control answers the same four questions: what does a key do, what is
//! the value now, is that value sound, and how should it be drawn. `Reaction`
//! is what makes navigation work -- a control that does not consume a key hands
//! it back, so Tab always moves focus even while a field is being typed into.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_input::{Input, InputRequest};

use crate::model::schema::Control;
use crate::model::value::Value;

#[derive(Debug, PartialEq, Eq)]
pub enum Reaction {
    /// The control used the key.
    Consumed,
    /// The control did not want it; the app should treat it as a command.
    Pass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Validation {
    Ok,
    /// Something worth saying, but never a reason to refuse a write. A tagger
    /// that will not save because it dislikes your description is worse than
    /// one that saves it.
    Warn(String),
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Plain,
    Url,
    Date,
    /// Multi-line in principle; edited here as one line, with ⌃E reserved for
    /// handing the whole thing to $EDITOR.
    Long,
}

/// An enum option: what is stored, and what is shown. They differ for Kind,
/// where the container holds the `stik` integer but nobody wants to read "9".
#[derive(Debug, Clone)]
pub struct Opt {
    pub code: String,
    pub label: String,
}

pub struct EnumEdit {
    pub input: Input,
    pub options: Vec<Opt>,
    /// A closed enum refuses free text: `stik` has no meaning outside its set.
    pub closed: bool,
}

/// A list rendered as chips but edited as one line. Per-chip selection and
/// reordering are deliberately not here yet: editing the joined text is more
/// flexible and far less code, and the chip look is what carries the meaning.
pub struct Chips {
    pub input: Input,
    pub hash: bool,
}

pub enum Editor {
    Line { input: Input, kind: LineKind },
    Chips(Chips),
    Stars(u8),
    Enum(EnumEdit),
    ReadOnly(String),
}

impl Editor {
    pub fn new(control: Control, value: Option<&Value>, options: Vec<Opt>) -> Self {
        let text = value.map(render_value).unwrap_or_default();
        match control {
            Control::Stars => Editor::Stars(parse_stars(&text)),
            Control::Enum => {
                // Show the label for a stored code, so an existing `9` reads
                // as "Movie" the moment the form opens.
                let shown = options
                    .iter()
                    .find(|o| o.code == text)
                    .map(|o| o.label.clone())
                    .unwrap_or(text);
                let closed = options.iter().any(|o| o.code != o.label);
                Editor::Enum(EnumEdit {
                    input: Input::new(shown),
                    options,
                    closed,
                })
            }
            Control::List => Editor::Chips(Chips { input: Input::new(text), hash: false }),
            Control::HashTags => Editor::Chips(Chips { input: Input::new(text), hash: true }),
            Control::Url => Editor::Line { input: Input::new(text), kind: LineKind::Url },
            Control::Date => Editor::Line { input: Input::new(text), kind: LineKind::Date },
            Control::TextArea => Editor::Line { input: Input::new(text), kind: LineKind::Long },
            Control::ReadOnly => Editor::ReadOnly(text),
            Control::Text => Editor::Line { input: Input::new(text), kind: LineKind::Plain },
        }
    }

    pub fn handle(&mut self, key: KeyEvent) -> Reaction {
        match self {
            Editor::ReadOnly(_) => Reaction::Pass,
            Editor::Line { input, .. } => line_key(input, key),
            Editor::Chips(c) => line_key(&mut c.input, key),
            Editor::Stars(n) => stars_key(n, key),
            Editor::Enum(e) => enum_key(e, key),
        }
    }

    pub fn value(&self) -> Value {
        match self {
            Editor::ReadOnly(s) => Value::Text(s.clone()),
            Editor::Line { input, .. } => Value::Text(input.value().trim().to_string()),
            Editor::Chips(c) => Value::List(if c.hash {
                split_tags(c.input.value())
            } else {
                split_list(c.input.value())
            }),
            Editor::Stars(n) => Value::Text(n.to_string()),
            Editor::Enum(e) => {
                let shown = e.input.value().trim();
                let code = e
                    .options
                    .iter()
                    .find(|o| o.label.eq_ignore_ascii_case(shown))
                    .map(|o| o.code.clone())
                    .unwrap_or_else(|| shown.to_string());
                Value::Text(code)
            }
        }
    }

    /// The text drawn in the value column, and where the cursor sits in it.
    pub fn display(&self) -> (String, Option<usize>) {
        match self {
            Editor::ReadOnly(s) => (s.clone(), None),
            Editor::Line { input, .. } => (input.value().to_string(), Some(input.visual_cursor())),
            Editor::Chips(c) => (c.input.value().to_string(), Some(c.input.visual_cursor())),
            Editor::Stars(n) => (stars_glyphs(*n), None),
            Editor::Enum(e) => (e.input.value().to_string(), Some(e.input.visual_cursor())),
        }
    }

    pub fn validate(&self) -> Validation {
        match self {
            Editor::Line { input, kind } => validate_line(input.value().trim(), *kind),
            Editor::Enum(e) => {
                let v = e.input.value().trim();
                if v.is_empty() || e.options.iter().any(|o| o.label.eq_ignore_ascii_case(v)) {
                    Validation::Ok
                } else if e.closed {
                    Validation::Error(format!(
                        "not one of: {}",
                        e.options.iter().map(|o| o.label.as_str()).collect::<Vec<_>>().join(", ")
                    ))
                } else {
                    Validation::Warn(format!(
                        "unknown value; known: {}",
                        e.options.iter().map(|o| o.label.as_str()).collect::<Vec<_>>().join(", ")
                    ))
                }
            }
            Editor::Chips(c) => {
                let items = if c.hash { split_tags(c.input.value()) } else { split_list(c.input.value()) };
                match items.iter().find(|t| t.contains(['/', '\\', ':']) || t.starts_with('.')) {
                    Some(bad) => Validation::Warn(format!("‘{bad}’ is awkward in a filename")),
                    None => Validation::Ok,
                }
            }
            _ => Validation::Ok,
        }
    }
}

fn validate_line(v: &str, kind: LineKind) -> Validation {
    if v.is_empty() {
        return Validation::Ok;
    }
    match kind {
        LineKind::Plain => Validation::Ok,
        LineKind::Long => {
            // Some readers truncate `desc` at 255; the overflow belongs in the
            // long-description field rather than being silently lost.
            if v.len() > 255 {
                Validation::Warn(format!("{} bytes; over 255 some readers truncate", v.len()))
            } else {
                Validation::Ok
            }
        }
        LineKind::Url => match url::Url::parse(v) {
            Ok(u) if matches!(u.scheme(), "http" | "https") => Validation::Ok,
            Ok(u) => Validation::Warn(format!("unusual scheme ‘{}’", u.scheme())),
            Err(_) if v.contains('.') && !v.contains(' ') => {
                Validation::Warn("no scheme; did you mean https://…".into())
            }
            Err(e) => Validation::Error(format!("not a URL: {e}")),
        },
        LineKind::Date => {
            if is_iso_date(v) {
                Validation::Ok
            } else if v.len() == 8 && v.chars().all(|c| c.is_ascii_digit()) {
                Validation::Warn("YYYYMMDD; will be stored as YYYY-MM-DD".into())
            } else {
                Validation::Warn("expected YYYY-MM-DD".into())
            }
        }
    }
}

fn is_iso_date(v: &str) -> bool {
    let b = v.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

/// Only the editing chords are claimed. Everything else passes back so the app
/// can use it as a command -- otherwise a control would swallow ⌃P and the
/// inspector would stop working whenever a text field had focus.
fn line_key(input: &mut Input, key: KeyEvent) -> Reaction {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let req = match (key.code, ctrl) {
        (KeyCode::Char(c), false) => InputRequest::InsertChar(c),
        (KeyCode::Backspace, false) => InputRequest::DeletePrevChar,
        (KeyCode::Delete, false) => InputRequest::DeleteNextChar,
        (KeyCode::Left, false) => InputRequest::GoToPrevChar,
        (KeyCode::Right, false) => InputRequest::GoToNextChar,
        (KeyCode::Home, _) => InputRequest::GoToStart,
        (KeyCode::End, _) => InputRequest::GoToEnd,
        (KeyCode::Left, true) => InputRequest::GoToPrevWord,
        (KeyCode::Right, true) => InputRequest::GoToNextWord,
        (KeyCode::Char('w'), true) => InputRequest::DeletePrevWord,
        (KeyCode::Char('k'), true) => InputRequest::DeleteTillEnd,
        _ => return Reaction::Pass,
    };
    input.handle(req);
    Reaction::Consumed
}

fn stars_key(n: &mut u8, key: KeyEvent) -> Reaction {
    match key.code {
        KeyCode::Char(c @ '0'..='5') => *n = c as u8 - b'0',
        KeyCode::Left | KeyCode::Char('h') => *n = n.saturating_sub(1),
        KeyCode::Right | KeyCode::Char('l') => *n = (*n + 1).min(5),
        _ => return Reaction::Pass,
    }
    Reaction::Consumed
}

fn enum_key(e: &mut EnumEdit, key: KeyEvent) -> Reaction {
    match key.code {
        KeyCode::Left | KeyCode::Right => {
            if e.options.is_empty() {
                return Reaction::Pass;
            }
            let cur = e
                .options
                .iter()
                .position(|o| o.label.eq_ignore_ascii_case(e.input.value().trim()));
            let n = e.options.len();
            let next = match (cur, key.code) {
                (Some(i), KeyCode::Right) => (i + 1) % n,
                (Some(i), _) => (i + n - 1) % n,
                (None, KeyCode::Right) => 0,
                (None, _) => n - 1,
            };
            e.input = Input::new(e.options[next].label.clone());
            Reaction::Consumed
        }
        // A closed enum has no free-text mode; typing into it would only ever
        // produce a value nothing can read.
        KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Delete if e.closed => Reaction::Pass,
        _ => line_key(&mut e.input, key),
    }
}

pub fn render_value(v: &Value) -> String {
    match v {
        Value::Text(s) => s.replace('\n', " "),
        Value::List(l) => l.join(", "),
    }
}

pub fn stars_glyphs(n: u8) -> String {
    (1..=5).map(|i| if i <= n { '★' } else { '☆' }).collect()
}

fn parse_stars(s: &str) -> u8 {
    s.trim().parse::<u8>().unwrap_or_else(|_| s.chars().filter(|c| *c == '★').count() as u8).min(5)
}

pub fn split_list(s: &str) -> Vec<String> {
    s.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect()
}

pub fn split_tags(s: &str) -> Vec<String> {
    s.split([',', ' '])
        .map(|p| p.trim().trim_start_matches('#').to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
    }
    fn code(k: KeyCode) -> KeyEvent {
        KeyEvent::new(k, KeyModifiers::NONE)
    }
    fn opts(v: &[(&str, &str)]) -> Vec<Opt> {
        v.iter().map(|(c, l)| Opt { code: c.to_string(), label: l.to_string() }).collect()
    }

    #[test]
    fn typing_edits_but_tab_passes_through() {
        let mut e = Editor::new(Control::Text, None, vec![]);
        assert_eq!(e.handle(key('h')), Reaction::Consumed);
        assert_eq!(e.handle(key('i')), Reaction::Consumed);
        assert_eq!(e.value(), Value::Text("hi".into()));
        // Focus movement must survive a focused text field.
        assert_eq!(e.handle(code(KeyCode::Tab)), Reaction::Pass);
    }

    #[test]
    fn stars_take_digits_and_arrows() {
        let mut e = Editor::new(Control::Stars, Some(&Value::Text("2".into())), vec![]);
        assert_eq!(e.display().0, "★★☆☆☆");
        e.handle(key('4'));
        assert_eq!(e.value(), Value::Text("4".into()));
        e.handle(code(KeyCode::Right));
        assert_eq!(e.value(), Value::Text("5".into()));
        e.handle(code(KeyCode::Right)); // clamps
        assert_eq!(e.value(), Value::Text("5".into()));
    }

    #[test]
    fn hashtags_round_trip_without_storing_the_hash() {
        let mut e = Editor::new(Control::HashTags, Some(&Value::List(vec!["pov".into()])), vec![]);
        for c in " #hd".chars() {
            e.handle(key(c));
        }
        assert_eq!(e.value(), Value::List(vec!["pov".into(), "hd".into()]));
    }

    /// Kind stores the stik integer but shows a word.
    #[test]
    fn closed_enum_shows_label_and_stores_code() {
        let o = opts(&[("9", "Movie"), ("10", "TV Show")]);
        let mut e = Editor::new(Control::Enum, Some(&Value::Text("9".into())), o);
        assert_eq!(e.display().0, "Movie");
        assert_eq!(e.value(), Value::Text("9".into()));
        e.handle(code(KeyCode::Right));
        assert_eq!(e.display().0, "TV Show");
        assert_eq!(e.value(), Value::Text("10".into()));
    }

    #[test]
    fn closed_enum_refuses_free_text() {
        let o = opts(&[("9", "Movie")]);
        let mut e = Editor::new(Control::Enum, Some(&Value::Text("9".into())), o);
        assert_eq!(e.handle(key('x')), Reaction::Pass);
        assert_eq!(e.value(), Value::Text("9".into()));
    }

    /// Genre is open: an unknown value is a warning, never a refusal, because
    /// the known set comes from a config file that changes.
    #[test]
    fn open_enum_warns_but_accepts() {
        let o = opts(&[("Media", "Media"), ("Footage", "Footage")]);
        let mut e = Editor::new(Control::Enum, None, o);
        for c in "Concert".chars() {
            assert_eq!(e.handle(key(c)), Reaction::Consumed);
        }
        assert!(matches!(e.validate(), Validation::Warn(_)));
        assert_eq!(e.value(), Value::Text("Concert".into()));
    }

    #[test]
    fn url_validation() {
        let v = |s: &str| validate_line(s, LineKind::Url);
        assert_eq!(v(""), Validation::Ok);
        assert_eq!(v("https://example.com/a"), Validation::Ok);
        assert!(matches!(v("ftp://example.com"), Validation::Warn(_)));
        assert!(matches!(v("example.com/a"), Validation::Warn(_)));
        assert!(matches!(v("not a url"), Validation::Error(_)));
    }

    #[test]
    fn date_validation() {
        assert_eq!(validate_line("2026-08-29", LineKind::Date), Validation::Ok);
        assert!(matches!(validate_line("20260829", LineKind::Date), Validation::Warn(_)));
        assert!(matches!(validate_line("29/08/26", LineKind::Date), Validation::Warn(_)));
    }

    #[test]
    fn long_text_warns_past_the_desc_limit() {
        assert_eq!(validate_line(&"x".repeat(255), LineKind::Long), Validation::Ok);
        assert!(matches!(validate_line(&"x".repeat(256), LineKind::Long), Validation::Warn(_)));
    }

    /// Seeding a control from a value and reading it straight back must be a
    /// fixed point, for present *and* absent values. The commit path compares
    /// an edit against the original round-tripped through the same control, so
    /// if this drifted, merely tabbing past a field would stage a change --
    /// which is exactly what an empty Rating used to do, staging a 0 on every
    /// file the cursor passed over.
    #[test]
    fn seeding_a_control_from_its_own_value_is_a_fixed_point() {
        let kinds = opts(&[("9", "Movie"), ("10", "TV Show")]);
        let genres = opts(&[("Media", "Media"), ("Footage", "Footage")]);
        let cases: Vec<(Control, Option<Value>, Vec<Opt>)> = vec![
            (Control::Text, None, vec![]),
            (Control::Text, Some(Value::Text("hi".into())), vec![]),
            (Control::Stars, None, vec![]),
            (Control::Stars, Some(Value::Text("3".into())), vec![]),
            (Control::List, None, vec![]),
            (Control::List, Some(Value::List(vec!["A".into(), "B".into()])), vec![]),
            (Control::HashTags, None, vec![]),
            (Control::HashTags, Some(Value::List(vec!["pov".into()])), vec![]),
            (Control::Url, None, vec![]),
            (Control::Date, None, vec![]),
            (Control::TextArea, None, vec![]),
            (Control::Enum, None, genres.clone()),
            (Control::Enum, Some(Value::Text("Media".into())), genres),
            (Control::Enum, None, kinds.clone()),
            (Control::Enum, Some(Value::Text("9".into())), kinds),
        ];
        for (control, value, options) in cases {
            let once = Editor::new(control, value.as_ref(), options.clone()).value();
            let twice = Editor::new(control, Some(&once), options).value();
            assert_eq!(once, twice, "{control:?} with {value:?} is not a fixed point");
        }
    }

    #[test]
    fn tag_with_a_slash_warns() {
        let e = Editor::new(Control::HashTags, Some(&Value::List(vec!["a/b".into()])), vec![]);
        assert!(matches!(e.validate(), Validation::Warn(_)));
    }
}
