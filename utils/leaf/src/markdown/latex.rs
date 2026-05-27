pub(crate) fn to_unicode(text: &str) -> String {
    let preprocessed = strip_command_spaces(text);
    let converted = unicodeit::replace(&preprocessed);
    postprocess(&converted)
}

fn strip_command_spaces(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '\\' && i + 1 < len && chars[i + 1].is_ascii_alphabetic() {
            result.push('\\');
            i += 1;
            while i < len && chars[i].is_ascii_alphabetic() {
                result.push(chars[i]);
                i += 1;
            }
            if i < len && chars[i] == ' ' {
                let next = chars.get(i + 1).copied().unwrap_or(' ');
                if next.is_ascii_alphabetic() || next == '\\' || next == '{' {
                    i += 1;
                }
            }
            continue;
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}

fn postprocess(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut i = 0;

    while i < input.len() {
        if input[i..].starts_with("\\frac{") {
            if let Some((output, end)) = parse_frac(input, i) {
                result.push_str(&output);
                i = end;
                continue;
            }
            result.push_str("\\frac{");
            i += 6;
            continue;
        }

        if input[i..].starts_with("√{") {
            let brace_start = i + '√'.len_utf8() + 1;
            if let Some((group, end)) = read_brace_group(input, brace_start) {
                result.push('√');
                result.push('(');
                result.push_str(&postprocess(group));
                result.push(')');
                i = end;
                continue;
            }
        }

        if input[i..].starts_with("^{") {
            if let Some((output, end)) = convert_script(input, i + 2, to_superscript) {
                result.push_str(&output);
                i = end;
                continue;
            }
            result.push_str("^{");
            i += 2;
            continue;
        }

        if input[i..].starts_with("_{") {
            if let Some((output, end)) = convert_script(input, i + 2, to_subscript) {
                result.push_str(&output);
                i = end;
                continue;
            }
            result.push_str("_{");
            i += 2;
            continue;
        }

        if input[i..].starts_with('^') && i + 1 < input.len() {
            let next = input[i + 1..].chars().next().unwrap();
            if next != '{' {
                i += 1;
                continue;
            }
        }

        let ch = input[i..].chars().next().unwrap();
        result.push(ch);
        i += ch.len_utf8();
    }

    result
}

fn parse_frac(input: &str, start: usize) -> Option<(String, usize)> {
    let after_frac = start + 6;
    let (num, after_num) = read_brace_group(input, after_frac)?;
    if after_num >= input.len() || input.as_bytes()[after_num] != b'{' {
        return None;
    }
    let (den, after_den) = read_brace_group(input, after_num + 1)?;
    let num = postprocess(num);
    let den = postprocess(den);
    let mut out = String::new();
    wrap_if_multi(&mut out, &num);
    out.push('/');
    wrap_if_multi(&mut out, &den);
    Some((out, after_den))
}

fn wrap_if_multi(out: &mut String, s: &str) {
    if s.chars().count() > 1 {
        out.push('(');
        out.push_str(s);
        out.push(')');
    } else {
        out.push_str(s);
    }
}

fn convert_script(
    input: &str,
    brace_start: usize,
    mapper: fn(char) -> char,
) -> Option<(String, usize)> {
    let (group, end) = read_brace_group(input, brace_start)?;
    let group = postprocess(group);
    let mapped: String = group.chars().map(mapper).collect();
    let all_converted = mapped
        .chars()
        .zip(group.chars())
        .all(|(m, g)| m != g || g.is_ascii_digit());
    if all_converted {
        Some((mapped, end))
    } else {
        Some((format!("({group})"), end))
    }
}

fn read_brace_group(input: &str, start: usize) -> Option<(&str, usize)> {
    let bytes = input.as_bytes();
    let mut depth: u32 = 1;
    let mut i = start;
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            _ => {}
        }
        if depth > 0 {
            i += 1;
        }
    }
    if depth == 0 {
        Some((&input[start..i], i + 1))
    } else {
        None
    }
}

fn to_superscript(ch: char) -> char {
    match ch {
        '0' => '⁰',
        '1' => '¹',
        '2' => '²',
        '3' => '³',
        '4' => '⁴',
        '5' => '⁵',
        '6' => '⁶',
        '7' => '⁷',
        '8' => '⁸',
        '9' => '⁹',
        '+' => '⁺',
        '-' | '−' => '⁻',
        '=' => '⁼',
        '(' => '⁽',
        ')' => '⁾',
        'n' => 'ⁿ',
        'i' => 'ⁱ',
        _ => ch,
    }
}

fn to_subscript(ch: char) -> char {
    match ch {
        '0' => '₀',
        '1' => '₁',
        '2' => '₂',
        '3' => '₃',
        '4' => '₄',
        '5' => '₅',
        '6' => '₆',
        '7' => '₇',
        '8' => '₈',
        '9' => '₉',
        '+' => '₊',
        '-' | '−' => '₋',
        '=' => '₌',
        '(' => '₍',
        ')' => '₎',
        'a' => 'ₐ',
        'e' => 'ₑ',
        'i' => 'ᵢ',
        'j' => 'ⱼ',
        'k' => 'ₖ',
        'n' => 'ₙ',
        'o' => 'ₒ',
        'p' => 'ₚ',
        'r' => 'ᵣ',
        's' => 'ₛ',
        't' => 'ₜ',
        'u' => 'ᵤ',
        'x' => 'ₓ',
        _ => ch,
    }
}
