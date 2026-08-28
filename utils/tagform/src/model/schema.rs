//! The field schema (SPEC §3): what the user sees, and where it lands.
//!
//! A *field* is one label and one control. A *key* is what is stored in the
//! container. The relation is one-to-many — the URL field writes five keys —
//! and that fan-out is the reason this tool exists.
//!
//! The `read` list is deliberately wider than `mdta`: this library has files
//! tagged by several generations of these scripts, so a URL might be present as
//! `comment` (old media-write-tags), `purl` (yt-dlp), `source_url`/`webpage_url`
//! (media-embed) or `original_url` (media-audit). Read accepts any alias; write
//! emits the canonical set. That asymmetry is what makes tagform idempotent.

/// The control a field is edited with (SPEC §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Control {
    Text,
    TextArea,
    List,
    HashTags,
    Url,
    Stars,
    Enum,
    Date,
    /// Displayed, never edited — probed or write-once data.
    ReadOnly,
}

pub struct FieldDef {
    pub id: &'static str,
    pub label: &'static str,
    pub control: Control,
    /// Canonical keys written in mdta mode, in order.
    pub mdta: &'static [&'static str],
    /// Atom keys accepted on read, first match wins.
    pub read: &'static [&'static str],
    /// XMP tags, first match wins. Authoritative over atoms when present,
    /// because that is where rename-footage puts authored data (SPEC §3.6).
    pub xmp: &'static [&'static str],
    /// The iTunes atom, where one exists at all. Measured, not assumed —
    /// see docs/CONTAINER.md.
    pub ilst: Option<&'static str>,
    /// Shown only when Genre is Footage.
    pub footage_only: bool,
}

macro_rules! field {
    ($id:literal, $label:literal, $control:expr, mdta: [$($m:literal),*],
     read: [$($r:literal),*], xmp: [$($x:literal),*], ilst: $ilst:expr) => {
        FieldDef {
            id: $id, label: $label, control: $control,
            mdta: &[$($m),*], read: &[$($r),*], xmp: &[$($x),*],
            ilst: $ilst, footage_only: false,
        }
    };
}

pub static FIELDS: &[FieldDef] = &[
    field!("title", "Title", Control::Text,
        mdta: ["title"], read: ["title"], xmp: ["XMP-dc:Title"], ilst: Some("\u{a9}nam")),

    // yt-dlp writes %(cast,uploader)l to both actors and artist; rename-footage
    // writes the same people to XMP as a true list.
    field!("actors", "Actors", Control::List,
        mdta: ["actors", "artist"], read: ["actors", "cast", "artist"],
        xmp: ["XMP-iptcExt:PersonInImage"], ilst: Some("\u{a9}ART")),

    field!("artist", "Artist", Control::Text,
        mdta: ["artist"], read: ["artist"], xmp: [], ilst: Some("\u{a9}ART")),

    // Stars, 0-5. Not rtng, not iTunEXTC (SPEC §3.3). XMP-xmp:Rating is a real
    // standard 0-5 field and is authoritative wherever it is present.
    field!("rating", "Rating", Control::Stars,
        mdta: ["rating"], read: ["rating"], xmp: ["XMP-xmp:Rating"], ilst: None),

    field!("description", "Description", Control::TextArea,
        mdta: ["description"], read: ["description"],
        xmp: ["XMP-dc:Description"], ilst: Some("desc")),

    // One field, five keys.
    field!("url", "URL", Control::Url,
        mdta: ["webpage_url", "source_url", "purl", "comment", "original_url"],
        read: ["webpage_url", "source_url", "purl", "original_url", "comment"],
        xmp: [], ilst: Some("purl")),

    field!("channel", "Channel", Control::Text,
        mdta: ["channel", "album_artist", "album"],
        read: ["channel", "album_artist", "album"],
        xmp: ["XMP-xmpDM:Album"], ilst: Some("aART")),

    field!("tags", "Tags", Control::HashTags,
        mdta: ["keywords"], read: ["keywords", "keyw"],
        xmp: ["XMP-dc:Subject"], ilst: Some("keyw")),

    field!("genre", "Genre", Control::Enum,
        mdta: ["genre"], read: ["genre"], xmp: [], ilst: Some("\u{a9}gen")),

    // The user's own axis (Clip/Master/Original) — no ilst atom exists.
    field!("type", "Type", Control::Enum,
        mdta: ["type"], read: ["type"], xmp: [], ilst: None),

    // The iTunes media kind (stik).
    field!("kind", "Kind", Control::Enum,
        mdta: ["media_type"], read: ["media_type"], xmp: [], ilst: Some("stik")),

    // Deliberately does NOT read `creation_time`: that is muxer-generated
    // bookkeeping (JUNK_KEYS), not an authored date, and treating it as one
    // would show every file carrying a date nobody set. Resolving a date for a
    // footage filename from exif/ctime is a separate concern (rename-footage's
    // resolve_date), not a field value.
    field!("date", "Date", Control::Date,
        mdta: ["date"], read: ["date"],
        xmp: ["XMP-xmp:CreateDate"], ilst: Some("\u{a9}day")),

    field!("synopsis", "Synopsis", Control::TextArea,
        mdta: ["synopsis"], read: ["synopsis"], xmp: [], ilst: Some("ldes")),

    field!("origin", "Origin", Control::Text,
        mdta: ["origin"], read: ["origin"], xmp: [], ilst: None),

    FieldDef {
        id: "location", label: "Location", control: Control::Text,
        mdta: &["location"], read: &["location"],
        xmp: &["XMP-iptcExt:LocationCreatedCity"], ilst: None, footage_only: true,
    },
    // Write-once: the only surviving copy of a camera's own IMG_4855.MOV.
    FieldDef {
        id: "preserved_name", label: "Original name", control: Control::ReadOnly,
        mdta: &[], read: &[], xmp: &["XMP-xmpMM:PreservedFileName"],
        ilst: None, footage_only: true,
    },
];

/// Muxer bookkeeping. Hidden from the form, and actively cleared on write —
/// with `-map_metadata 0` plus `use_metadata_tags`, ffmpeg promotes these to
/// real readable tags that then accumulate on every rewrite (docs/CONTAINER.md).
pub static JUNK_KEYS: &[&str] = &[
    "major_brand", "minor_version", "compatible_brands", "encoder",
    "handler_name", "vendor_id", "creation_time",
];

/// Every atom key any field claims, for splitting known from custom.
pub fn field_by_id(id: &str) -> Option<&'static FieldDef> {
    FIELDS.iter().find(|f| f.id == id)
}

pub fn claimed_atom_keys() -> Vec<&'static str> {
    let mut v: Vec<&'static str> =
        FIELDS.iter().flat_map(|f| f.read.iter().chain(f.mdta.iter()).copied()).collect();
    v.sort_unstable();
    v.dedup();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique() {
        let mut ids: Vec<_> = FIELDS.iter().map(|f| f.id).collect();
        let n = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), n, "duplicate field id");
    }

    /// Every canonical write key must also be readable, or a value tagform
    /// wrote would not be seen on the next open.
    #[test]
    fn write_keys_round_trip_through_read() {
        for f in FIELDS {
            for k in f.mdta {
                assert!(f.read.contains(k), "{}: writes {k} but cannot read it", f.id);
            }
        }
    }

    #[test]
    fn junk_keys_are_not_claimed_by_any_field() {
        let claimed = claimed_atom_keys();
        for j in JUNK_KEYS {
            assert!(!claimed.contains(j), "{j} is both junk and a field key");
        }
    }
}
