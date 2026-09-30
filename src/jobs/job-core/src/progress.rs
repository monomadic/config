//! Progress, as read out of whatever a job prints.

/// A percentage anywhere in a line, as a 0..1 fraction — `45%`, `at 45.5%`,
/// `[ 45% ]`. The last one wins, since a line that carries several is most
/// likely counting up to the one nearest its end.
pub fn parse_progress(line: &str) -> Option<f64> {
    let bytes = line.as_bytes();
    let mut found = None;
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b'%' {
            continue;
        }
        let mut start = index;
        let mut seen_digit = false;
        let mut seen_dot = false;
        while start > 0 {
            let previous = bytes[start - 1];
            if previous.is_ascii_digit() {
                seen_digit = true;
            } else if previous == b'.' && !seen_dot && seen_digit {
                seen_dot = true;
            } else {
                break;
            }
            start -= 1;
        }
        if seen_digit && let Ok(value) = line[start..index].parse::<f64>() {
            found = Some((value / 100.0).clamp(0.0, 1.0));
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_last_percentage_in_a_line_wins() {
        assert_eq!(parse_progress("encoding 45% eta 1:00"), Some(0.45));
        assert_eq!(parse_progress("[ 12.5% ]"), Some(0.125));
        assert_eq!(parse_progress("pass 1 100% pass 2 30%"), Some(0.3));
        assert_eq!(parse_progress("no numbers here"), None);
        assert_eq!(parse_progress("odd %"), None);
    }
}
