//! Frames on screen: decode, crop to the zoom region, and hand to the terminal
//! image protocol (kitty graphics, or half-blocks elsewhere), centred in its
//! area. Zoom is what makes a 4K render judgeable in a terminal cell grid: at
//! 2× and 4× the crop is scaled up with nearest-neighbour, so each rendered
//! pixel stays a crisp block rather than a smear.

use image::DynamicImage;
use ratatui::layout::Rect;
use ratatui::Frame;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::{FilterType, Resize, StatefulImage};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const ZOOM_LEVELS: [u32; 4] = [1, 2, 4, 8];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Zoom {
    pub level: u32,
    /// Centre of the view, as a fraction of the image's width and height.
    pub cx: f64,
    pub cy: f64,
}

impl Default for Zoom {
    fn default() -> Self {
        Zoom { level: 1, cx: 0.5, cy: 0.5 }
    }
}

impl Zoom {
    pub fn cycle(&mut self) {
        let i = ZOOM_LEVELS.iter().position(|l| *l == self.level).unwrap_or(0);
        self.level = ZOOM_LEVELS[(i + 1) % ZOOM_LEVELS.len()];
        if self.level == 1 {
            self.cx = 0.5;
            self.cy = 0.5;
        }
    }

    /// Move by half the visible width / height.
    pub fn pan(&mut self, dx: f64, dy: f64) {
        if self.level == 1 {
            return;
        }
        let half = 0.5 / self.level as f64;
        let step = 0.5 / self.level as f64;
        self.cx = (self.cx + dx * step).clamp(half, 1.0 - half);
        self.cy = (self.cy + dy * step).clamp(half, 1.0 - half);
    }

    fn key(&self) -> (u32, i32, i32) {
        (self.level, (self.cx * 1000.0).round() as i32, (self.cy * 1000.0).round() as i32)
    }
}

type Key = (PathBuf, (u32, i32, i32));

/// Decoded frames and protocol states are both costly (a 4K frame is ~25 MB
/// decoded), so each cache is small and evicts everything not on screen.
const DECODED_CAP: usize = 4;
const PROTO_CAP: usize = 16;
/// Largest edge handed to the protocol; the terminal never shows more.
const MAX_EDGE: u32 = 2560;

pub struct Images {
    picker: Picker,
    decoded: HashMap<PathBuf, DynamicImage>,
    protos: HashMap<Key, (StatefulProtocol, u32, u32)>,
    /// Keys drawn this frame: never evicted mid-frame (split view shows two).
    pinned: Vec<Key>,
}

impl Images {
    pub fn new(picker: Picker) -> Self {
        Images { picker, decoded: HashMap::new(), protos: HashMap::new(), pinned: Vec::new() }
    }

    pub fn begin_frame(&mut self) {
        self.pinned.clear();
    }

    /// Drop everything held for `path` — for live frames, which are replaced
    /// and deleted every few seconds through a long encode.
    pub fn forget(&mut self, path: &Path) {
        self.decoded.remove(path);
        self.protos.retain(|(p, _), _| p != path);
    }

    pub fn render(&mut self, f: &mut Frame, path: &Path, zoom: Zoom, area: Rect) -> Result<(), String> {
        if area.width == 0 || area.height == 0 {
            return Ok(());
        }
        let key: Key = (path.to_path_buf(), zoom.key());
        if !self.protos.contains_key(&key) {
            let img = self.crop(path, zoom)?;
            let (w, h) = (img.width(), img.height());
            if self.protos.len() >= PROTO_CAP {
                let pinned = &self.pinned;
                self.protos.retain(|k, _| pinned.contains(k));
            }
            let proto = self.picker.new_resize_protocol(img);
            self.protos.insert(key.clone(), (proto, w, h));
        }
        self.pinned.push(key.clone());
        let font = self.picker.font_size();
        let (proto, w, h) = self.protos.get_mut(&key).unwrap();
        let upscale = zoom.level > 1;
        let rect = centred(area, *w, *h, (font.width, font.height), upscale);
        let resize = if upscale {
            Resize::Scale(Some(FilterType::Nearest))
        } else {
            Resize::Fit(Some(FilterType::Triangle))
        };
        f.render_stateful_widget(StatefulImage::default().resize(resize), rect, proto);
        Ok(())
    }

    fn crop(&mut self, path: &Path, zoom: Zoom) -> Result<DynamicImage, String> {
        if !self.decoded.contains_key(path) {
            let img = image::ImageReader::open(path)
                .map_err(|e| format!("cannot open {}: {e}", path.display()))?
                .decode()
                .map_err(|e| format!("cannot decode {}: {e}", path.display()))?;
            if self.decoded.len() >= DECODED_CAP {
                self.decoded.clear();
            }
            self.decoded.insert(path.to_path_buf(), img);
        }
        let img = &self.decoded[path];
        let (iw, ih) = (img.width(), img.height());
        let out = if zoom.level > 1 {
            let cw = (iw / zoom.level).max(1);
            let ch = (ih / zoom.level).max(1);
            let x = ((zoom.cx * iw as f64) as i64 - cw as i64 / 2).clamp(0, (iw - cw) as i64) as u32;
            let y = ((zoom.cy * ih as f64) as i64 - ch as i64 / 2).clamp(0, (ih - ch) as i64) as u32;
            img.crop_imm(x, y, cw, ch)
        } else {
            img.clone()
        };
        Ok(if out.width().max(out.height()) > MAX_EDGE {
            out.resize(MAX_EDGE, MAX_EDGE, FilterType::Triangle)
        } else {
            out
        })
    }
}

/// The part of `area` an image of `w`×`h` pixels fills once fitted (and, when
/// zoomed, scaled up), centred — so it does not hug the top-left corner.
fn centred(area: Rect, w: u32, h: u32, font: (u16, u16), upscale: bool) -> Rect {
    let (fw, fh) = (font.0.max(1) as f64, font.1.max(1) as f64);
    let (aw, ah) = (area.width as f64 * fw, area.height as f64 * fh);
    let mut s = (aw / w.max(1) as f64).min(ah / h.max(1) as f64);
    if !upscale {
        s = s.min(1.0);
    }
    let cols = ((w as f64 * s / fw).ceil() as u16).clamp(1, area.width);
    let rows = ((h as f64 * s / fh).ceil() as u16).clamp(1, area.height);
    Rect {
        x: area.x + (area.width - cols) / 2,
        y: area.y + (area.height - rows) / 2,
        width: cols,
        height: rows,
    }
}
