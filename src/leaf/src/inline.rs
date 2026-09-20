use anyhow::{bail, Result};
use ratatui::{
    style::{Color, Modifier},
    text::Line,
};
use std::io::Write;

const DEFAULT_WIDTH: usize = 80;
const MIN_WIDTH: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InlineSpec {
    pub(crate) format: InlineFormat,
    pub(crate) width: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InlineFormat {
    Auto,
    Ansi,
    Plain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResolvedFormat {
    Ansi,
    Plain,
}

pub(crate) fn is_inline_spec(value: &str) -> bool {
    if value.starts_with('-') {
        return false;
    }
    let lower = value.to_ascii_lowercase();
    if lower == "ansi" || lower == "plain" {
        return true;
    }
    if value.bytes().all(|b| b.is_ascii_digit()) && !value.is_empty() {
        return true;
    }
    matches!(lower.split_once(':'), Some((fmt, _)) if fmt == "ansi" || fmt == "plain")
}

pub(crate) fn parse_inline_spec(value: &str) -> Result<InlineSpec> {
    let value = value.trim();
    if value.is_empty() {
        bail!("Empty inline spec");
    }

    if value.bytes().all(|b| b.is_ascii_digit()) {
        let width = parse_width(value)?;
        return Ok(InlineSpec {
            format: InlineFormat::Auto,
            width: Some(width),
        });
    }

    if let Some((fmt, w)) = value.split_once(':') {
        let format = parse_format_name(fmt)?;
        let width = parse_width(w)?;
        return Ok(InlineSpec {
            format,
            width: Some(width),
        });
    }

    let format = parse_format_name(value)?;
    Ok(InlineSpec {
        format,
        width: None,
    })
}

fn parse_format_name(name: &str) -> Result<InlineFormat> {
    match name.to_ascii_lowercase().as_str() {
        "ansi" => Ok(InlineFormat::Ansi),
        "plain" => Ok(InlineFormat::Plain),
        _ => bail!("Unknown inline format: {name} (expected 'ansi' or 'plain')"),
    }
}

fn parse_width(s: &str) -> Result<usize> {
    let w: usize = s
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid width: {s}"))?;
    if w == 0 {
        bail!("Width must be a positive integer");
    }
    Ok(w.max(MIN_WIDTH))
}

pub(crate) fn render_width(spec: &InlineSpec, is_stdout_terminal: bool) -> usize {
    if let Some(w) = spec.width {
        return w.max(MIN_WIDTH);
    }
    if is_stdout_terminal {
        crossterm::terminal::size()
            .map(|(cols, _)| (cols as usize).max(MIN_WIDTH))
            .unwrap_or(DEFAULT_WIDTH)
    } else {
        DEFAULT_WIDTH
    }
}

pub(crate) fn resolve_format(spec: &InlineSpec, is_stdout_terminal: bool) -> ResolvedFormat {
    match spec.format {
        InlineFormat::Ansi => ResolvedFormat::Ansi,
        InlineFormat::Plain => ResolvedFormat::Plain,
        InlineFormat::Auto if is_stdout_terminal => ResolvedFormat::Ansi,
        InlineFormat::Auto => ResolvedFormat::Plain,
    }
}

pub(crate) fn write_lines<W: Write>(
    lines: &[Line<'_>],
    format: ResolvedFormat,
    max_width: usize,
    writer: &mut W,
) -> Result<()> {
    for line in lines {
        match format {
            ResolvedFormat::Ansi => write_line_ansi(line, max_width, writer)?,
            ResolvedFormat::Plain => write_line_plain(line, max_width, writer)?,
        }
    }
    Ok(())
}

fn write_line_ansi<W: Write>(line: &Line<'_>, max_width: usize, writer: &mut W) -> Result<()> {
    let mut col = 0usize;
    for span in &line.spans {
        let style = &span.style;
        let mods = style.add_modifier;
        let fg = style.fg.filter(|c| !matches!(c, Color::Reset));
        let bg = style.bg.filter(|c| !matches!(c, Color::Reset));
        let has_style = fg.is_some() || bg.is_some() || !mods.is_empty();

        if has_style {
            write_ansi_style(writer, fg, bg, mods)?;
        }

        for ch in span.content.chars() {
            let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            if col + ch_width > max_width && col > 0 {
                if has_style {
                    write_bytes(writer, b"\x1b[0m")?;
                }
                write_bytes(writer, b"\n")?;
                col = 0;
                if has_style {
                    write_ansi_style(writer, fg, bg, mods)?;
                }
            }
            let mut buf = [0u8; 4];
            write_bytes(writer, ch.encode_utf8(&mut buf).as_bytes())?;
            col += ch_width;
        }

        if has_style {
            write_bytes(writer, b"\x1b[0m")?;
        }
    }
    write_bytes(writer, b"\x1b[0m\n")?;
    Ok(())
}

fn write_line_plain<W: Write>(line: &Line<'_>, max_width: usize, writer: &mut W) -> Result<()> {
    let mut col = 0usize;
    for span in &line.spans {
        for ch in span.content.chars() {
            let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            if col + ch_width > max_width && col > 0 {
                write_bytes(writer, b"\n")?;
                col = 0;
            }
            let mut buf = [0u8; 4];
            write_bytes(writer, ch.encode_utf8(&mut buf).as_bytes())?;
            col += ch_width;
        }
    }
    write_bytes(writer, b"\n")?;
    Ok(())
}

fn write_ansi_style<W: Write>(
    writer: &mut W,
    fg: Option<Color>,
    bg: Option<Color>,
    mods: Modifier,
) -> Result<()> {
    write_bytes(writer, b"\x1b[")?;
    let mut need_sep = false;

    if let Some(c) = fg {
        if let Some(code) = color_ansi_code(c, false) {
            write_bytes(writer, code)?;
        } else {
            write_extended_color(writer, c, false)?;
        }
        need_sep = true;
    }
    if let Some(c) = bg {
        if need_sep {
            write_bytes(writer, b";")?;
        }
        if let Some(code) = color_ansi_code(c, true) {
            write_bytes(writer, code)?;
        } else {
            write_extended_color(writer, c, true)?;
        }
        need_sep = true;
    }

    for (flag, code) in [
        (Modifier::BOLD, &b"1"[..]),
        (Modifier::ITALIC, b"3"),
        (Modifier::UNDERLINED, b"4"),
        (Modifier::CROSSED_OUT, b"9"),
    ] {
        if mods.contains(flag) {
            if need_sep {
                write_bytes(writer, b";")?;
            }
            write_bytes(writer, code)?;
            need_sep = true;
        }
    }

    write_bytes(writer, b"m")?;
    Ok(())
}

fn write_bytes<W: Write>(writer: &mut W, bytes: &[u8]) -> Result<()> {
    match writer.write_all(bytes) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
        Err(err) => Err(err.into()),
    }
}

fn color_ansi_code(color: Color, bg: bool) -> Option<&'static [u8]> {
    #[rustfmt::skip]
    static TABLE: [[&[u8]; 2]; 16] = [
        [b"30",  b"40"],   // Black
        [b"31",  b"41"],   // Red
        [b"32",  b"42"],   // Green
        [b"33",  b"43"],   // Yellow
        [b"34",  b"44"],   // Blue
        [b"35",  b"45"],   // Magenta
        [b"36",  b"46"],   // Cyan
        [b"37",  b"47"],   // Gray
        [b"90",  b"100"],  // DarkGray
        [b"91",  b"101"],  // LightRed
        [b"92",  b"102"],  // LightGreen
        [b"93",  b"103"],  // LightYellow
        [b"94",  b"104"],  // LightBlue
        [b"95",  b"105"],  // LightMagenta
        [b"96",  b"106"],  // LightCyan
        [b"97",  b"107"],  // White
    ];
    let idx = bg as usize;
    match color {
        Color::Reset => None,
        Color::Black => Some(TABLE[0][idx]),
        Color::Red => Some(TABLE[1][idx]),
        Color::Green => Some(TABLE[2][idx]),
        Color::Yellow => Some(TABLE[3][idx]),
        Color::Blue => Some(TABLE[4][idx]),
        Color::Magenta => Some(TABLE[5][idx]),
        Color::Cyan => Some(TABLE[6][idx]),
        Color::Gray => Some(TABLE[7][idx]),
        Color::DarkGray => Some(TABLE[8][idx]),
        Color::LightRed => Some(TABLE[9][idx]),
        Color::LightGreen => Some(TABLE[10][idx]),
        Color::LightYellow => Some(TABLE[11][idx]),
        Color::LightBlue => Some(TABLE[12][idx]),
        Color::LightMagenta => Some(TABLE[13][idx]),
        Color::LightCyan => Some(TABLE[14][idx]),
        Color::White => Some(TABLE[15][idx]),
        Color::Indexed(_) | Color::Rgb(_, _, _) => None,
    }
}

fn write_extended_color<W: Write>(writer: &mut W, color: Color, bg: bool) -> Result<()> {
    use std::io::Cursor;
    let base: u8 = if bg { 48 } else { 38 };
    let mut buf = [0u8; 20];
    let mut cur = Cursor::new(&mut buf[..]);
    match color {
        Color::Indexed(n) => {
            let _ = write!(cur, "{base};5;{n}");
        }
        Color::Rgb(r, g, b) => {
            let _ = write!(cur, "{base};2;{r};{g};{b}");
        }
        _ => return Ok(()),
    }
    let len = cur.position() as usize;
    write_bytes(writer, &buf[..len])
}
