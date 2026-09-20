use super::file_picker::FilePickerEntry;

const IGNORED_FUZZY_PICKER_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    ".venv",
    "venv",
    "vendor",
    "var",
    "dist",
    "build",
    ".next",
    ".cache",
];

pub(super) fn is_ignored_fuzzy_picker_dir_name(name: &str) -> bool {
    IGNORED_FUZZY_PICKER_DIRS.contains(&name)
}

pub(super) fn fuzzy_directory_sort_key(
    root: &std::path::Path,
    path: &std::path::Path,
) -> (bool, String) {
    let label = path
        .strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string();
    (
        !label
            .split(std::path::MAIN_SEPARATOR)
            .next()
            .unwrap_or(&label)
            .starts_with('.'),
        label.to_lowercase(),
    )
}

pub(super) fn fuzzy_entry_sort_key(entry: &FilePickerEntry) -> (bool, &str) {
    let first_component = entry
        .label()
        .split(std::path::MAIN_SEPARATOR)
        .next()
        .unwrap_or(entry.label());
    (!first_component.starts_with('.'), entry.label_lower())
}

pub(super) fn fuzzy_component_match(candidate: &str, query: &str) -> Option<(usize, Vec<usize>)> {
    if let Some(start) = candidate.find(query) {
        let start_chars = candidate[..start].chars().count();
        let query_len = query.chars().count();
        let len_diff = candidate.chars().count().saturating_sub(query_len);
        let prefix_bonus = usize::from(start_chars == 0).saturating_mul(80);
        let boundary_bonus =
            usize::from(is_match_boundary(candidate, start_chars)).saturating_mul(40);
        let score = start_chars
            .saturating_mul(10)
            .saturating_add(len_diff)
            .saturating_sub(prefix_bonus)
            .saturating_sub(boundary_bonus);
        let positions = (start_chars..start_chars + query_len).collect::<Vec<_>>();
        return Some((score, positions));
    }

    let mut search_from = 0usize;
    let mut positions = Vec::with_capacity(query.len());

    for needle in query.chars() {
        let found = candidate[search_from..]
            .char_indices()
            .find(|(_, ch)| *ch == needle)
            .map(|(idx, _)| search_from + idx)?;
        let char_pos = candidate[..found].chars().count();
        positions.push(char_pos);
        search_from = found + needle.len_utf8();
    }

    let first = *positions.first()?;
    let last = *positions.last()?;
    let span = last.saturating_sub(first);
    let gaps = positions
        .windows(2)
        .map(|window| window[1].saturating_sub(window[0]).saturating_sub(1))
        .sum::<usize>();
    let len_diff = candidate
        .chars()
        .count()
        .saturating_sub(query.chars().count());
    let prefix_bonus = usize::from(first == 0).saturating_mul(80);
    let boundary_bonus = usize::from(is_match_boundary(candidate, first)).saturating_mul(40);
    let score = 1_000usize
        .saturating_add(gaps.saturating_mul(120))
        .saturating_add(first.saturating_mul(10))
        .saturating_add(span)
        .saturating_add(len_diff)
        .saturating_sub(prefix_bonus)
        .saturating_sub(boundary_bonus);
    Some((score, positions))
}

pub(super) fn is_match_boundary(candidate: &str, char_pos: usize) -> bool {
    if char_pos == 0 {
        return true;
    }

    candidate
        .chars()
        .nth(char_pos.saturating_sub(1))
        .is_some_and(|ch| matches!(ch, '-' | '_' | '.' | ' '))
}

pub(super) fn fuzzy_match(entry: &FilePickerEntry, query: &str) -> Option<(usize, Vec<usize>)> {
    if query.is_empty() {
        return Some((0, Vec::new()));
    }

    let (score, positions) = fuzzy_component_match(entry.file_name_lower(), query)?;
    Some((
        score,
        positions
            .into_iter()
            .map(|position| entry.file_name_offset() + position)
            .collect(),
    ))
}
