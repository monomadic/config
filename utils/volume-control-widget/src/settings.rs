//! `key=value` lines in `~/.config/volume-control-widget/settings`, in the
//! family convention. Alongside the layout style, this file is the rule
//! store: `always_mute` and `never_mute` lines carry CoreAudio device UIDs,
//! and each ruled device also gets a `known` line remembering its name and
//! transport — that is what lets a disconnected DJM keep its row.
//!
//! Rows mutate rules directly (the lock button cycles a device's policy), so
//! everything loads from disk on every refresh rather than trusting a copy
//! in memory; the file is a few hundred bytes.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use crate::audio::Transport;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LayoutStyle {
    IconRoute,
    Icon,
    IconBar,
    IconBarRoute,
    IconPercent,
}

pub const ALL_STYLES: [LayoutStyle; 5] = [
    LayoutStyle::IconRoute,
    LayoutStyle::Icon,
    LayoutStyle::IconBar,
    LayoutStyle::IconBarRoute,
    LayoutStyle::IconPercent,
];

impl LayoutStyle {
    pub fn label(self) -> &'static str {
        match self {
            LayoutStyle::IconRoute => "Icon and Route",
            LayoutStyle::Icon => "Icon",
            LayoutStyle::IconBar => "Icon and Bar",
            LayoutStyle::IconBarRoute => "Icon, Bar and Route",
            LayoutStyle::IconPercent => "Icon and Percent",
        }
    }

    fn key(self) -> &'static str {
        match self {
            LayoutStyle::IconRoute => "icon_route",
            LayoutStyle::Icon => "icon",
            LayoutStyle::IconBar => "icon_bar",
            LayoutStyle::IconBarRoute => "icon_bar_route",
            LayoutStyle::IconPercent => "icon_percent",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        ALL_STYLES.iter().copied().find(|style| style.key() == key)
    }
}

/// What the lock button on a row currently holds.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    None,
    /// Held muted: enforcement re-mutes the instant anything unmutes it.
    AlwaysMute,
    /// Read-only: the app never writes mute or volume to this device, and
    /// its absence is worth a warning.
    NeverMute,
}

impl Policy {
    fn next(self) -> Self {
        match self {
            Policy::None => Policy::AlwaysMute,
            Policy::AlwaysMute => Policy::NeverMute,
            Policy::NeverMute => Policy::None,
        }
    }
}

/// A device we hold a rule for, remembered well enough to draw its row when
/// it is unplugged.
#[derive(Clone)]
pub struct KnownDevice {
    pub uid: String,
    pub name: String,
    pub transport: Transport,
}

#[derive(Clone)]
pub struct Settings {
    pub style: LayoutStyle,
    pub always_mute: HashSet<String>,
    pub never_mute: HashSet<String>,
    pub known: Vec<KnownDevice>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            style: LayoutStyle::IconRoute,
            always_mute: HashSet::new(),
            never_mute: HashSet::new(),
            known: Vec::new(),
        }
    }
}

impl Settings {
    fn path() -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        Some(PathBuf::from(home).join(".config/volume-control-widget/settings"))
    }

    pub fn load() -> Self {
        let mut settings = Settings::default();
        let Some(text) = Settings::path().and_then(|path| fs::read_to_string(path).ok()) else {
            return settings;
        };
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            match key.trim() {
                "style" => {
                    if let Some(style) = LayoutStyle::from_key(value) {
                        settings.style = style;
                    }
                }
                "always_mute" => {
                    settings.always_mute.insert(value.to_string());
                }
                "never_mute" => {
                    settings.never_mute.insert(value.to_string());
                }
                "known" => {
                    let mut parts = value.splitn(3, '\t');
                    if let (Some(uid), Some(transport), Some(name)) =
                        (parts.next(), parts.next(), parts.next())
                    {
                        settings.known.push(KnownDevice {
                            uid: uid.to_string(),
                            name: name.to_string(),
                            transport: Transport::from_key(transport),
                        });
                    }
                }
                _ => {}
            }
        }
        settings
    }

    pub fn save(&self) {
        let Some(path) = Settings::path() else { return };
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let mut body = format!("style={}\n", self.style.key());
        for uid in &self.always_mute {
            body.push_str(&format!("always_mute={uid}\n"));
        }
        for uid in &self.never_mute {
            body.push_str(&format!("never_mute={uid}\n"));
        }
        for device in &self.known {
            body.push_str(&format!(
                "known={}\t{}\t{}\n",
                device.uid,
                device.transport.key(),
                device.name
            ));
        }
        if let Err(err) = fs::write(&path, body) {
            eprintln!("error saving settings: {err}");
        }
    }

    pub fn policy(&self, uid: &str) -> Policy {
        if self.always_mute.contains(uid) {
            Policy::AlwaysMute
        } else if self.never_mute.contains(uid) {
            Policy::NeverMute
        } else {
            Policy::None
        }
    }

    fn set_policy(&mut self, uid: &str, policy: Policy) {
        self.always_mute.remove(uid);
        self.never_mute.remove(uid);
        match policy {
            Policy::AlwaysMute => {
                self.always_mute.insert(uid.to_string());
            }
            Policy::NeverMute => {
                self.never_mute.insert(uid.to_string());
            }
            Policy::None => {}
        }
        self.known
            .retain(|known| known.uid != uid || policy != Policy::None);
    }

    /// Keep the `known` records for ruled devices current while they are
    /// connected, so a later unplugged row shows the right name and pill.
    pub fn remember(&mut self, devices: &[crate::audio::Device]) {
        for device in devices {
            if self.policy(&device.uid) == Policy::None {
                continue;
            }
            match self.known.iter_mut().find(|known| known.uid == device.uid) {
                Some(known) => {
                    known.name = device.name.clone();
                    known.transport = device.transport;
                }
                None => self.known.push(KnownDevice {
                    uid: device.uid.clone(),
                    name: device.name.clone(),
                    transport: device.transport,
                }),
            }
        }
        let always = &self.always_mute;
        let never = &self.never_mute;
        self.known
            .retain(|known| always.contains(&known.uid) || never.contains(&known.uid));
    }
}

/// The lock button's action: none → always mute → never mute → none, saved
/// straight to disk. Returns the new policy for the row to redraw with.
pub fn cycle_policy(uid: &str, name: &str, transport: Transport) -> Policy {
    let mut settings = Settings::load();
    let next = settings.policy(uid).next();
    settings.set_policy(uid, next);
    if next != Policy::None && !settings.known.iter().any(|known| known.uid == uid) {
        settings.known.push(KnownDevice {
            uid: uid.to_string(),
            name: name.to_string(),
            transport,
        });
    }
    settings.save();
    next
}
