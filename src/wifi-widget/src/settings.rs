use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Style {
    #[default]
    Smart,
    Icon,
    /// Glyph on the left; band text stacked above the signal meter on the right.
    Stacked,
}
impl Style {
    pub fn token(self) -> &'static str {
        match self {
            Style::Smart => "smart",
            Style::Icon => "icon",
            Style::Stacked => "stacked",
        }
    }
    pub fn from_token(token: &str) -> Option<Self> {
        match token {
            "smart" => Some(Style::Smart),
            "icon" => Some(Style::Icon),
            "stacked" => Some(Style::Stacked),
            _ => None,
        }
    }
}
fn path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config/wifi-widget/settings"))
}
pub fn load() -> Style {
    path()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|contents| {
            contents
                .lines()
                .filter_map(|line| line.strip_prefix("style="))
                .find_map(Style::from_token)
        })
        .unwrap_or_default()
}
pub fn save(style: Style) -> io::Result<()> {
    let path = path().ok_or_else(|| io::Error::other("Home directory unavailable"))?;
    fs::create_dir_all(path.parent().unwrap())?;
    let temp = path.with_extension(format!("{}.tmp", std::process::id()));
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        writeln!(file, "style={}", style.token())?;
        file.sync_all()?;
        fs::rename(&temp, &path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}
