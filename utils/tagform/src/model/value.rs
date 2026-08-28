//! Values and the multi-file aggregation model (SPEC §4.3).

use serde::Serialize;

/// A tag value as it exists in a container. Everything is a string or a list of
/// strings at this layer; typed interpretation (stars, enums, dates) belongs to
/// the control layer, so the reader never has to guess.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum Value {
    Text(String),
    List(Vec<String>),
}

impl Value {
    pub fn text(s: impl Into<String>) -> Self {
        Value::Text(s.into())
    }

    /// The form a field is displayed and edited in.
    pub fn as_display(&self) -> String {
        match self {
            Value::Text(s) => s.clone(),
            Value::List(v) => v.join(", "),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Value::Text(s) => s.trim().is_empty(),
            Value::List(v) => v.is_empty(),
        }
    }
}

/// How a field's value stands across the whole selection.
///
/// `Mixed` is preserved on write: a field left alone keeps each file's own
/// value. Only a user assignment (`Set`) or clear (`Unset`) touches disk.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum Agg {
    /// Present in no file.
    Absent,
    /// Present and identical in every file.
    Same { value: Value },
    /// Differs between files, or present in only some of them.
    Mixed { values: Vec<Option<Value>> },
}

impl Agg {
    /// Fold one value per file into an aggregate.
    ///
    /// A field absent everywhere is `Absent`, not `Mixed` — "nobody has this"
    /// and "these disagree" are different states and the UI renders them
    /// differently.
    pub fn fold(per_file: Vec<Option<Value>>) -> Self {
        let mut present = per_file.iter().flatten();
        let Some(first) = present.next() else {
            return Agg::Absent;
        };
        let all_present = per_file.iter().all(|v| v.is_some());
        if all_present && present.all(|v| v == first) {
            Agg::Same { value: first.clone() }
        } else {
            Agg::Mixed { values: per_file }
        }
    }

    /// The single value, when there is one.
    pub fn value(&self) -> Option<&Value> {
        match self {
            Agg::Same { value } => Some(value),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> Option<Value> {
        Some(Value::text(s))
    }

    #[test]
    fn absent_everywhere_is_absent_not_mixed() {
        assert!(matches!(Agg::fold(vec![None, None]), Agg::Absent));
    }

    #[test]
    fn identical_in_all_is_same() {
        let a = Agg::fold(vec![t("x"), t("x"), t("x")]);
        assert_eq!(a.value(), Some(&Value::text("x")));
    }

    #[test]
    fn differing_is_mixed() {
        assert!(matches!(Agg::fold(vec![t("x"), t("y")]), Agg::Mixed { .. }));
    }

    /// Present in only some files is Mixed even though the present values
    /// agree — writing the shared value would create it where it was absent.
    #[test]
    fn partial_presence_is_mixed() {
        assert!(matches!(Agg::fold(vec![t("x"), None]), Agg::Mixed { .. }));
    }

    #[test]
    fn lists_compare_by_content() {
        let l = |v: &[&str]| Some(Value::List(v.iter().map(|s| s.to_string()).collect()));
        assert!(matches!(Agg::fold(vec![l(&["a", "b"]), l(&["a", "b"])]), Agg::Same { .. }));
        assert!(matches!(Agg::fold(vec![l(&["a", "b"]), l(&["b", "a"])]), Agg::Mixed { .. }));
    }
}
