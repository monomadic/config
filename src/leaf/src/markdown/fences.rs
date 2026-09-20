use std::borrow::Cow;

pub(super) struct FenceInfo<'a> {
    pub(super) backtick_count: usize,
    pub(super) info: &'a str,
    pub(super) prefix: &'a str,
}

pub(super) fn normalize_code_fences(src: &str) -> Cow<'_, str> {
    if !needs_fence_normalization(src) {
        return Cow::Borrowed(src);
    }

    let raw_lines: Vec<&str> = src.lines().collect();
    let mut result: Vec<String> = raw_lines.iter().map(|l| l.to_string()).collect();
    let mut changed = false;
    let mut i = 0;

    while i < raw_lines.len() {
        let Some(fi) = parse_fence_line(raw_lines[i]) else {
            i += 1;
            continue;
        };
        if fi.info.is_empty() {
            i += 1;
            continue;
        }

        let open_line = i;
        let open_prefix = fi.prefix;
        let fence_len = fi.backtick_count;
        let mut max_inner_fence = 0usize;
        let mut close_line: Option<(usize, &str)> = None;
        let mut j = i + 1;
        let mut depth = 1usize;

        while j < raw_lines.len() && depth > 0 {
            if let Some(inner) = parse_fence_line(raw_lines[j]) {
                if !inner.info.is_empty() {
                    max_inner_fence = max_inner_fence.max(inner.backtick_count);
                    depth += 1;
                } else if depth > 1 {
                    depth -= 1;
                } else {
                    close_line = Some((j, inner.prefix));
                    depth = 0;
                }
            }
            j += 1;
        }

        if max_inner_fence >= fence_len {
            if let Some((cl, cl_prefix)) = close_line {
                let new_fence = "`".repeat(max_inner_fence + 1);
                result[open_line] = format!("{}{}{}", open_prefix, new_fence, fi.info);
                result[cl] = format!("{}{}", cl_prefix, new_fence);
                changed = true;
            }
        }

        i = j;
    }

    if changed {
        Cow::Owned(result.join("\n"))
    } else {
        Cow::Borrowed(src)
    }
}

pub(super) fn needs_fence_normalization(src: &str) -> bool {
    let mut in_fenced = false;
    let mut fence_len = 0usize;
    for line in src.lines() {
        if let Some(fi) = parse_fence_line(line) {
            if !in_fenced && !fi.info.is_empty() {
                in_fenced = true;
                fence_len = fi.backtick_count;
            } else if in_fenced {
                if !fi.info.is_empty() && fi.backtick_count >= fence_len {
                    return true;
                }
                if fi.info.is_empty() {
                    in_fenced = false;
                }
            }
        }
    }
    false
}

pub(super) fn parse_fence_line(line: &str) -> Option<FenceInfo<'_>> {
    let mut rest = line;
    let mut prefix_end = 0;

    loop {
        let trimmed = rest.trim_start();
        let spaces = rest.len() - trimmed.len();
        prefix_end += spaces;
        if let Some(after_gt) = trimmed.strip_prefix('>') {
            prefix_end += 1;
            rest = after_gt;
            if let Some(after_space) = rest.strip_prefix(' ') {
                prefix_end += 1;
                rest = after_space;
            }
        } else {
            break;
        }
    }

    let leading_spaces = rest.len() - rest.trim_start().len();
    if leading_spaces > 3 {
        return None;
    }
    let trimmed = rest.trim_start();
    let backtick_count = trimmed.chars().take_while(|c| *c == '`').count();
    if backtick_count >= 3 {
        let info = trimmed[backtick_count..].trim();
        Some(FenceInfo {
            backtick_count,
            info,
            prefix: &line[..prefix_end],
        })
    } else {
        None
    }
}
