//! Sigil modes: a reserved first character switches the panel from app
//! search to a special-purpose resolver. `#` evaluates math, `!` expands
//! web-search shortcuts. Sigils and shortcuts live in the config
//! (`[modes]`, `[modes.web]`); everything here is pure string → rows, so
//! the UI layer stays the only place that touches AppKit.

use crate::config::WebShortcut;

/// One synthesized result row. `detail` renders dim and right-aligned;
/// `tag` renders as an inline pill after the name (web: the prefix).
pub struct ModeRow {
    pub name: String,
    pub detail: Option<String>,
    pub tag: Option<String>,
    pub action: ModeAction,
}

/// What Enter does on a mode row.
#[derive(Clone, PartialEq, Debug)]
pub enum ModeAction {
    /// Copy the string to the clipboard.
    Copy(String),
    /// Open the URL in the default browser.
    OpenUrl(String),
}

// ---- math (`#`) ----

/// Rows for a math query: the result first ("= 4"), then, for the
/// "P% of N" phrasing, the two neighboring readings (N + P%, N − P%). An
/// empty query shows a resting `= 0`; unparseable input shows nothing.
pub fn math_rows(input: &str) -> Vec<ModeRow> {
    let input = input.trim();
    if input.is_empty() {
        return vec![result_row(0.0)];
    }
    let Some(value) = eval(input) else {
        return Vec::new();
    };
    let mut rows = vec![result_row(value)];
    if let Some((pct, base)) = parse_pct_of(input) {
        for (op, v) in [
            ("+", base * (1.0 + pct / 100.0)),
            ("−", base * (1.0 - pct / 100.0)),
        ] {
            let f = format_num(v);
            rows.push(ModeRow {
                name: format!("{} {op} {}%", format_num(base), format_num(pct)),
                detail: Some(f.clone()),
                tag: None,
                action: ModeAction::Copy(f),
            });
        }
    }
    rows
}

fn result_row(value: f64) -> ModeRow {
    let f = format_num(value);
    ModeRow {
        name: format!("= {f}"),
        detail: None,
        tag: None,
        action: ModeAction::Copy(f),
    }
}

/// Exactly "P% of N" (the phrasing that gets alternate rows).
fn parse_pct_of(s: &str) -> Option<(f64, f64)> {
    let (lhs, rhs) = s.split_once("of")?;
    let lhs = lhs.trim();
    let pct = lhs.strip_suffix('%')?.trim();
    let pct = parse_number(pct)?;
    let base = parse_number(rhs.trim())?;
    Some((pct, base))
}

/// A bare number, commas allowed. Rejects anything with operators.
fn parse_number(s: &str) -> Option<f64> {
    if s.is_empty() || !s.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',') {
        return None;
    }
    s.replace(',', "").parse().ok()
}

/// Intermediate value: percents stay symbolic until they combine, because
/// `100 + 4%` means 104 while `4% of 100` means 4.
#[derive(Clone, Copy)]
enum Val {
    Num(f64),
    Pct(f64),
}

impl Val {
    fn as_f64(self) -> f64 {
        match self {
            Val::Num(n) => n,
            Val::Pct(p) => p / 100.0,
        }
    }
}

struct Parser<'a> {
    toks: Vec<Tok<'a>>,
    pos: usize,
}

#[derive(Clone, Copy, PartialEq)]
enum Tok<'a> {
    Num(f64),
    Word(&'a str),
    Op(char),
}

fn tokenize(s: &str) -> Option<Vec<Tok<'_>>> {
    let mut toks = Vec::new();
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() || c == '.' {
            let start = i;
            while i < b.len()
                && ((b[i] as char).is_ascii_digit() || b[i] == b'.' || b[i] == b',')
            {
                i += 1;
            }
            let n: f64 = s[start..i].replace(',', "").parse().ok()?;
            toks.push(Tok::Num(n));
        } else if c.is_ascii_alphabetic() {
            let start = i;
            while i < b.len() && (b[i] as char).is_ascii_alphabetic() {
                i += 1;
            }
            toks.push(Tok::Word(&s[start..i]));
        } else if "+-*/^()%×÷−".contains(c) {
            // Multibyte glyphs: normalize to their ASCII operator.
            let op = match c {
                '×' => '*',
                '÷' => '/',
                '−' => '-',
                other => other,
            };
            toks.push(Tok::Op(op));
            i += c.len_utf8();
        } else {
            return None;
        }
    }
    Some(toks)
}

/// Evaluate a calculator expression. `%` is a postfix percent; `of` and
/// `x` read as multiplication ("4% of 100", "3 x 5").
pub fn eval(s: &str) -> Option<f64> {
    let toks = tokenize(s)?;
    if toks.is_empty() {
        return None;
    }
    let mut p = Parser { toks, pos: 0 };
    let v = p.expr()?;
    (p.pos == p.toks.len()).then(|| v.as_f64())
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<Tok<'a>> {
        self.toks.get(self.pos).copied()
    }

    fn expr(&mut self) -> Option<Val> {
        let mut lhs = self.term()?;
        while let Some(Tok::Op(op @ ('+' | '-'))) = self.peek() {
            self.pos += 1;
            let rhs = self.term()?;
            lhs = match (lhs, rhs, op) {
                // `N + P%` reads as N grown/shrunk by P percent.
                (Val::Num(n), Val::Pct(p), '+') => Val::Num(n * (1.0 + p / 100.0)),
                (Val::Num(n), Val::Pct(p), '-') => Val::Num(n * (1.0 - p / 100.0)),
                (a, b, '+') => Val::Num(a.as_f64() + b.as_f64()),
                (a, b, _) => Val::Num(a.as_f64() - b.as_f64()),
            };
        }
        Some(lhs)
    }

    fn term(&mut self) -> Option<Val> {
        let mut lhs = self.factor()?;
        loop {
            let mul = match self.peek() {
                Some(Tok::Op(op @ ('*' | '/'))) => op == '*',
                Some(Tok::Word(w)) if w.eq_ignore_ascii_case("of") => true,
                Some(Tok::Word(w)) if w.eq_ignore_ascii_case("x") => true,
                _ => break,
            };
            self.pos += 1;
            let rhs = self.factor()?;
            // `as_f64` scales percents to fractions, so `4% of 100` and
            // `200 * 15%` fall out of plain multiplication.
            let v = if mul {
                lhs.as_f64() * rhs.as_f64()
            } else {
                lhs.as_f64() / rhs.as_f64()
            };
            lhs = Val::Num(v);
        }
        Some(lhs)
    }

    fn factor(&mut self) -> Option<Val> {
        let base = self.unary()?;
        if let Some(Tok::Op('^')) = self.peek() {
            self.pos += 1;
            let exp = self.factor()?; // right-associative
            return Some(Val::Num(base.as_f64().powf(exp.as_f64())));
        }
        Some(base)
    }

    fn unary(&mut self) -> Option<Val> {
        if let Some(Tok::Op('-')) = self.peek() {
            self.pos += 1;
            let v = self.unary()?;
            return Some(Val::Num(-v.as_f64()));
        }
        self.primary()
    }

    fn primary(&mut self) -> Option<Val> {
        match self.peek()? {
            Tok::Num(n) => {
                self.pos += 1;
                if let Some(Tok::Op('%')) = self.peek() {
                    self.pos += 1;
                    Some(Val::Pct(n))
                } else {
                    Some(Val::Num(n))
                }
            }
            Tok::Op('(') => {
                self.pos += 1;
                let v = self.expr()?;
                match self.peek() {
                    Some(Tok::Op(')')) => {
                        self.pos += 1;
                        Some(v)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

/// Human formatting: thousands separators, up to 6 decimals, no trailing
/// zeros ("8542.75" → "8,542.75", "4" → "4").
pub fn format_num(v: f64) -> String {
    if !v.is_finite() {
        return "∞".into();
    }
    let rounded = (v * 1e6).round() / 1e6;
    let neg = rounded < 0.0;
    let abs = rounded.abs();
    let int = abs.trunc() as u128;
    let frac = format!("{:.6}", abs.fract());
    let frac = frac[2..].trim_end_matches('0');

    let mut out = group_int(&int.to_string());
    if !frac.is_empty() {
        out.push('.');
        out.push_str(frac);
    }
    if neg && (int > 0 || !frac.is_empty()) {
        out.insert(0, '-');
    }
    out
}

/// Group an integer's digits with thousands commas ("1234567" → "1,234,567").
fn group_int(digits: &str) -> String {
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

// ---- web shortcuts (`!`) ----

/// Rows for a web query: `yt cat videos` puts the matching shortcut first
/// with "cat videos" as the search terms; the other shortcuts follow with
/// the same terms. Each row's title is the site name and its pill is the
/// prefix. An unrecognized first token is just part of the query.
pub fn web_rows(input: &str, shortcuts: &[WebShortcut]) -> Vec<ModeRow> {
    let input = input.trim();
    let find = |name: &str| shortcuts.iter().position(|w| w.prefix.eq_ignore_ascii_case(name));
    let (hit, terms) = match input.split_once(char::is_whitespace) {
        Some((first, rest)) => match find(first) {
            Some(i) => (Some(i), rest.trim()),
            None => (None, input),
        },
        None => match find(input) {
            Some(i) => (Some(i), ""),
            None => (None, input),
        },
    };

    let mut order: Vec<usize> = (0..shortcuts.len()).collect();
    if let Some(i) = hit {
        order.retain(|&j| j != i);
        order.insert(0, i);
    }
    order
        .into_iter()
        .map(|i| {
            let w = &shortcuts[i];
            let url = w.template.replace("{q}", &url_encode(terms));
            ModeRow {
                name: w.name.clone(),
                detail: None,
                tag: Some(w.prefix.clone()),
                action: ModeAction::OpenUrl(url),
            }
        })
        .collect()
}

// ---- currency (`$`) ----

/// A non-actionable informational row (e.g. "Fetching exchange rates…").
pub fn info_row(text: &str) -> ModeRow {
    ModeRow {
        name: text.to_string(),
        detail: None,
        tag: None,
        action: ModeAction::Copy(String::new()),
    }
}

/// Rows converting a typed amount into each target currency. `rates` maps a
/// currency code to units-per-USD (the Coinbase `exchange-rates` shape);
/// conversions pivot through USD. `age` (if any) rides the first row as its
/// dim right-hand label. Unparseable input or an unknown source currency
/// yields no rows.
pub fn currency_rows(
    input: &str,
    rates: &std::collections::HashMap<String, f64>,
    targets: &[String],
    age: Option<String>,
) -> Vec<ModeRow> {
    let Some((amount, from)) = parse_amount(input) else {
        return Vec::new();
    };
    let Some(from_rate) = rates.get(&from) else {
        return Vec::new();
    };
    let usd = amount / from_rate;

    let mut rows = Vec::new();
    for code in targets.iter().map(|t| t.to_uppercase()) {
        if code == from {
            continue;
        }
        let Some(rate) = rates.get(&code) else {
            continue;
        };
        let value = usd * rate;
        let sym = currency_symbol(&code).unwrap_or("");
        rows.push(ModeRow {
            name: format!("{sym}{}", format_money(value)),
            detail: None,
            tag: Some(code),
            action: ModeAction::Copy(plain_amount(value)),
        });
    }
    if let (Some(first), Some(age)) = (rows.first_mut(), age) {
        first.detail = Some(age);
    }
    rows
}

/// "500,000 php", "3k usd", "1.4btc" → (amount, uppercase code). A trailing
/// `k`/`m`/`b` right before a word boundary multiplies by 1e3/1e6/1e9 (so
/// "1.4btc" stays 1.4 BTC, not 1.4 billion). No code given defaults to USD.
fn parse_amount(input: &str) -> Option<(f64, String)> {
    let cs: Vec<char> = input.trim().chars().collect();
    let mut i = 0;
    let mut num = String::new();
    while i < cs.len() && (cs[i].is_ascii_digit() || cs[i] == '.' || cs[i] == ',') {
        if cs[i] != ',' {
            num.push(cs[i]);
        }
        i += 1;
    }
    let mut value: f64 = num.parse().ok()?;
    if i < cs.len() {
        let c = cs[i].to_ascii_lowercase();
        let boundary = i + 1 == cs.len() || cs[i + 1].is_whitespace();
        if boundary && matches!(c, 'k' | 'm' | 'b') {
            value *= match c {
                'k' => 1e3,
                'm' => 1e6,
                _ => 1e9,
            };
            i += 1;
        }
    }
    let code: String = cs[i..]
        .iter()
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c.is_ascii_alphabetic())
        .collect::<String>()
        .to_uppercase();
    Some((value, if code.is_empty() { "USD".into() } else { code }))
}

fn currency_symbol(code: &str) -> Option<&'static str> {
    Some(match code {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "JPY" | "CNY" => "¥",
        "PHP" => "₱",
        "AUD" => "A$",
        "CAD" => "C$",
        "BTC" => "₿",
        "ETH" => "Ξ",
        _ => return None,
    })
}

/// Money for display: 2 grouped decimals for ordinary amounts, up to 8
/// (trailing zeros trimmed) for sub-unit values like fractional BTC.
fn format_money(v: f64) -> String {
    let decimals: usize = if v != 0.0 && v.abs() < 1.0 { 8 } else { 2 };
    let factor = 10f64.powi(decimals as i32);
    let r = (v * factor).round() / factor;
    let neg = r < 0.0;
    let abs = r.abs();
    let int = abs.trunc() as u128;
    let frac = format!("{:.*}", decimals, abs.fract());
    let mut frac = frac[2..].to_string();
    if decimals > 2 {
        frac = frac.trim_end_matches('0').to_string();
    }
    let mut out = group_int(&int.to_string());
    if !frac.is_empty() {
        out.push('.');
        out.push_str(&frac);
    }
    if neg && (int > 0 || !frac.is_empty()) {
        out.insert(0, '-');
    }
    out
}

/// The value as a bare decimal for the clipboard (no symbol, no grouping).
fn plain_amount(v: f64) -> String {
    let r = if v != 0.0 && v.abs() < 1.0 {
        (v * 1e8).round() / 1e8
    } else {
        (v * 100.0).round() / 100.0
    };
    format!("{r}")
}

/// Extract `{ "CODE": "1.23", … }` from the Coinbase `exchange-rates`
/// payload into code → units-per-USD. Values may be quoted or bare; only
/// positive finite numbers are kept. Hand-rolled to stay dependency-free.
pub fn parse_rates(json: &str) -> std::collections::HashMap<String, f64> {
    let mut map = std::collections::HashMap::new();
    let Some(ri) = json.find("\"rates\"") else {
        return map;
    };
    let Some(open) = json[ri..].find('{') else {
        return map;
    };
    let body = &json[ri + open + 1..];
    let b = body.as_bytes();
    let mut i = 0;
    loop {
        while i < b.len() && b[i] != b'"' && b[i] != b'}' {
            i += 1;
        }
        if i >= b.len() || b[i] == b'}' {
            break;
        }
        i += 1;
        let ks = i;
        while i < b.len() && b[i] != b'"' {
            i += 1;
        }
        if i >= b.len() {
            break;
        }
        let key = &body[ks..i];
        i += 1;
        while i < b.len() && b[i] != b':' {
            i += 1;
        }
        if i >= b.len() {
            break;
        }
        i += 1;
        while i < b.len() && (b[i] as char).is_whitespace() {
            i += 1;
        }
        let val = if i < b.len() && b[i] == b'"' {
            i += 1;
            let vs = i;
            while i < b.len() && b[i] != b'"' {
                i += 1;
            }
            let v = &body[vs..i.min(b.len())];
            i += 1;
            v
        } else {
            let vs = i;
            while i < b.len()
                && matches!(b[i], b'0'..=b'9' | b'.' | b'-' | b'+' | b'e' | b'E')
            {
                i += 1;
            }
            &body[vs..i]
        };
        if let Ok(f) = val.parse::<f64>() {
            if f.is_finite() && f > 0.0 {
                map.insert(key.to_uppercase(), f);
            }
        }
    }
    map
}

/// Query-string percent-encoding: unreserved chars pass, space becomes
/// `+`, everything else is %XX per UTF-8 byte.
fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_basics() {
        assert_eq!(eval("2 + 3 * 4"), Some(14.0));
        assert_eq!(eval("(2 + 3) * 4"), Some(20.0));
        assert_eq!(eval("2 ^ 10"), Some(1024.0));
        assert_eq!(eval("-5 + 3"), Some(-2.0));
        assert_eq!(eval("10 / 4"), Some(2.5));
        assert_eq!(eval("3 x 5"), Some(15.0));
        assert_eq!(eval("1,000 + 24"), Some(1024.0));
        assert_eq!(eval("2 +"), None);
        assert_eq!(eval("what"), None);
        assert_eq!(eval(""), None);
    }

    #[test]
    fn eval_percent() {
        assert_eq!(eval("4% of 100"), Some(4.0));
        assert_eq!(eval("100 + 4%"), Some(104.0));
        assert_eq!(eval("100 - 4%"), Some(96.0));
        assert_eq!(eval("50%"), Some(0.5));
        assert_eq!(eval("200 * 15%"), Some(30.0));
    }

    #[test]
    fn math_rows_shape() {
        let rows = math_rows("4% of 100");
        assert_eq!(rows[0].name, "= 4");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[1].detail.as_deref(), Some("104"));
        assert_eq!(rows[2].detail.as_deref(), Some("96"));
        assert!(math_rows("garbage in").is_empty());
        assert_eq!(math_rows("2+2").len(), 1);
        // Empty query rests at "= 0".
        let empty = math_rows("");
        assert_eq!(empty.len(), 1);
        assert_eq!(empty[0].name, "= 0");
        assert_eq!(empty[0].action, ModeAction::Copy("0".into()));
    }

    #[test]
    fn formats_numbers() {
        assert_eq!(format_num(4.0), "4");
        assert_eq!(format_num(8542.75), "8,542.75");
        assert_eq!(format_num(1234567.0), "1,234,567");
        assert_eq!(format_num(-0.5), "-0.5");
        assert_eq!(format_num(0.1 + 0.2), "0.3");
    }

    #[test]
    fn web_rows_prefix() {
        let sc = vec![
            WebShortcut {
                prefix: "g".into(),
                name: "Google".into(),
                template: "https://g.example/s?q={q}".into(),
            },
            WebShortcut {
                prefix: "yt".into(),
                name: "YouTube".into(),
                template: "https://yt.example/r?q={q}".into(),
            },
        ];
        let rows = web_rows("yt cat videos", &sc);
        assert_eq!(rows.len(), 2);
        // Title is the site name; the prefix is the pill; no URL detail.
        assert_eq!(rows[0].name, "YouTube");
        assert_eq!(rows[0].tag.as_deref(), Some("yt"));
        assert_eq!(rows[0].detail, None);
        assert_eq!(
            rows[0].action,
            ModeAction::OpenUrl("https://yt.example/r?q=cat+videos".into())
        );
        // Second row carries the same terms through the other shortcut.
        assert_eq!(
            rows[1].action,
            ModeAction::OpenUrl("https://g.example/s?q=cat+videos".into())
        );

        // No prefix hit: the whole input is the query, config order kept.
        let rows = web_rows("cat videos", &sc);
        assert_eq!(rows[0].name, "Google");

        // Bare matching token: shortcut with empty terms.
        let rows = web_rows("yt", &sc);
        assert_eq!(rows[0].action, ModeAction::OpenUrl("https://yt.example/r?q=".into()));
    }

    #[test]
    fn parses_amounts() {
        assert_eq!(parse_amount("500,000 php"), Some((500000.0, "PHP".into())));
        assert_eq!(parse_amount("3k usd"), Some((3000.0, "USD".into())));
        assert_eq!(parse_amount("1.4btc"), Some((1.4, "BTC".into())));
        assert_eq!(parse_amount("2.5m eur"), Some((2_500_000.0, "EUR".into())));
        assert_eq!(parse_amount("100"), Some((100.0, "USD".into())));
        assert_eq!(parse_amount("abc"), None);
    }

    #[test]
    fn parses_coinbase_rates() {
        let json = r#"{"data":{"currency":"USD","rates":{"EUR":"0.92","PHP":"58.5","BTC":"0.0000152"}}}"#;
        let r = parse_rates(json);
        assert_eq!(r.get("EUR"), Some(&0.92));
        assert_eq!(r.get("PHP"), Some(&58.5));
        assert_eq!(r.get("BTC"), Some(&0.0000152));
        assert!(parse_rates("not json").is_empty());
    }

    #[test]
    fn converts_currency() {
        let mut rates = std::collections::HashMap::new();
        rates.insert("USD".to_string(), 1.0);
        rates.insert("PHP".to_string(), 58.0);
        rates.insert("EUR".to_string(), 0.5);
        let targets = vec!["USD".to_string(), "EUR".to_string(), "PHP".to_string()];
        // 580 PHP = 10 USD = 5 EUR; source PHP is skipped.
        let rows = currency_rows("580 php", &rates, &targets, Some("2h ago".into()));
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].tag.as_deref(), Some("USD"));
        assert_eq!(rows[0].name, "$10.00");
        assert_eq!(rows[0].detail.as_deref(), Some("2h ago")); // age on first row
        assert_eq!(rows[0].action, ModeAction::Copy("10".into()));
        assert_eq!(rows[1].tag.as_deref(), Some("EUR"));
        assert_eq!(rows[1].name, "€5.00");
        // Unknown source currency → no rows.
        assert!(currency_rows("5 xyz", &rates, &targets, None).is_empty());
    }

    #[test]
    fn formats_money_precision() {
        assert_eq!(format_money(8542.75), "8,542.75");
        assert_eq!(format_money(10.0), "10.00");
        assert_eq!(format_money(0.072153), "0.072153");
    }

    #[test]
    fn encodes_urls() {
        assert_eq!(url_encode("cat videos"), "cat+videos");
        assert_eq!(url_encode("a&b=c"), "a%26b%3Dc");
        assert_eq!(url_encode("naïve"), "na%C3%AFve");
    }
}
