//! Render deterministic native cards without touching live network or permissions.
use objc2_app_kit::NSAppearanceCustomization;

use crate::card::{Card, Content};
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSAppearance, NSAppearanceNameAqua, NSAppearanceNameDarkAqua, NSApplication,
    NSApplicationActivationPolicy, NSBackingStoreType, NSBitmapImageFileType, NSWindow,
    NSWindowStyleMask,
};
use objc2_foundation::{NSDictionary, NSPoint, NSRect, NSSize, NSString};
use std::{
    path::Path,
    time::{Duration, Instant},
};
use wifi_widget::{
    model::{PathFlags, Store},
    probe::ProbeStatus,
    wifi::{Band, LinkReading},
};
pub fn render(directory: &Path) -> Result<(), String> {
    let mtm = MainThreadMarker::new().ok_or("Main thread required")?;
    NSApplication::sharedApplication(mtm)
        .setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    for name in [
        "healthy",
        "healthy-light",
        "weak",
        "offline",
        "portal",
        "metered",
        "redacted",
        "missing-signal",
        "off",
        "disconnected",
        "long-name",
    ] {
        let now = Instant::now();
        let mut link = LinkReading {
            interface: Some("en0".into()),
            power: true,
            associated: true,
            ssid: Some("Studio".into()),
            bssid: Some("3c:22:fb:9e:41:a0".into()),
            mac: Some("f6:1d:83:5a:c0:27".into()),
            rssi: Some(-52),
            noise: Some(-93),
            rate_mbps: Some(1201.),
            channel: Some(37),
            band: Some(Band::Ghz6),
            width_mhz: Some(160),
            phy: Some("Wi-Fi 6E"),
        };
        match name {
            "weak" => {
                link.rssi = Some(-81);
                link.rate_mbps = Some(54.);
            }
            "redacted" => {
                link.ssid = None;
                link.bssid = None;
            }
            "missing-signal" => {
                link.rssi = None;
            }
            "off" => {
                link.power = false;
                link.associated = false;
            }
            "disconnected" => {
                link.associated = false;
            }
            "long-name" => {
                link.ssid = Some("A very long café network name that must truncate safely".into());
            }
            _ => {}
        }
        let mut store = Store::default();
        store.update_link(Some(link), now);
        store.update_probe(
            ProbeStatus::Reachable {
                latency: Duration::from_millis(18),
            },
            now,
        );
        match name {
            "offline" => {
                store.update_probe(ProbeStatus::ConnectFailure, now);
                store.update_probe(ProbeStatus::ConnectFailure, now);
            }
            "portal" => store.update_probe(
                ProbeStatus::Captive {
                    host: Some("login.example".into()),
                },
                now,
            ),
            "metered" => store.update_path(
                PathFlags {
                    expensive: true,
                    constrained: false,
                },
                now,
            ),
            _ => {}
        }
        let mut content = Content::from_snapshot(&store.snapshot(now), now);
        content.ip = Some("192.168.1.24".into());
        content.gateway = Some("192.168.1.1".into());
        let card = Card::new(content, mtm);
        let window = unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                NSWindow::alloc(mtm),
                NSRect {
                    origin: NSPoint {
                        x: -2000.,
                        y: -2000.,
                    },
                    size: NSSize {
                        width: 320.,
                        height: card.frame().size.height,
                    },
                },
                NSWindowStyleMask::Borderless,
                NSBackingStoreType::Buffered,
                false,
            )
        };
        window.setAppearance(
            NSAppearance::appearanceNamed(unsafe {
                if name == "healthy-light" {
                    NSAppearanceNameAqua
                } else {
                    NSAppearanceNameDarkAqua
                }
            })
            .as_deref(),
        );
        window.setContentView(Some(&card));
        let bitmap = card
            .bitmapImageRepForCachingDisplayInRect(card.bounds())
            .ok_or("Could not allocate preview bitmap")?;
        card.cacheDisplayInRect_toBitmapImageRep(card.bounds(), &bitmap);
        let data = unsafe {
            bitmap.representationUsingType_properties(
                NSBitmapImageFileType::PNG,
                &NSDictionary::new(),
            )
        }
        .ok_or("Could not encode PNG")?;
        let path = directory.join(format!("{name}.png"));
        if !data.writeToFile_atomically(&NSString::from_str(&path.to_string_lossy()), true) {
            return Err(format!("Could not write {}", path.display()));
        }
        println!("{}", path.display());
        if name == "healthy" {
            render_panel(
                Content::from_snapshot(&store.snapshot(now), now),
                directory,
                mtm,
            )?;
        }
    }
    Ok(())
}

fn render_panel(content: Content, directory: &Path, mtm: MainThreadMarker) -> Result<(), String> {
    use objc2_app_kit::*;
    let r = |x, y, width, height| NSRect {
        origin: NSPoint { x, y },
        size: NSSize { width, height },
    };
    let panel = NSView::new(mtm);
    panel.setFrame(r(0., 0., 320., 592.));
    let background = NSBox::new(mtm);
    background.setFrame(panel.bounds());
    background.setBoxType(NSBoxType::Custom);
    background.setBorderWidth(0.);
    background.setFillColor(&NSColor::windowBackgroundColor());
    panel.addSubview(&background);
    let card = Card::new(content, mtm);
    card.setFrameOrigin(NSPoint { x: 0., y: 272. });
    panel.addSubview(&card);
    let (header, _, _) = crate::row::section(None, None, mtm);
    header.setFrameOrigin(NSPoint { x: 0., y: 238. });
    panel.addSubview(&header);
    for (i, name) in ["Studio-IoT", "iPhone", "Grind Coffee", "Hotel"]
        .iter()
        .enumerate()
    {
        let row = crate::row::KnownRow::new(mtm);
        row.set_name(name);
        row.set_security(Some(if i == 0 { "WPA2/3" } else { "WPA2" }));
        row.set_reading(Some((wifi_widget::wifi::Band::Ghz5, -52 - i as i32 * 8)));
        row.set_saved(i < 2);
        row.setFrameOrigin(NSPoint {
            x: 0.,
            y: 208. - i as f64 * 30.,
        });
        panel.addSubview(&row);
    }
    for (i, title) in [
        "Style                                         ›",
        "Open at Login",
        "Wi-Fi Settings…",
        "Quit WiFi Widget",
    ]
    .iter()
    .enumerate()
    {
        let label = NSTextField::labelWithString(&NSString::from_str(title), mtm);
        label.setFont(Some(&NSFont::systemFontOfSize(13.)));
        label.setFrame(r(16., 85. - i as f64 * 24., 310., 21.));
        panel.addSubview(&label);
    }
    let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
            NSWindow::alloc(mtm),
            r(-2000., -2000., 320., 592.),
            NSWindowStyleMask::Borderless,
            NSBackingStoreType::Buffered,
            false,
        )
    };
    window.setAppearance(
        NSAppearance::appearanceNamed(unsafe { NSAppearanceNameDarkAqua }).as_deref(),
    );
    window.setContentView(Some(&panel));
    let bitmap = panel
        .bitmapImageRepForCachingDisplayInRect(panel.bounds())
        .ok_or("Panel bitmap failed")?;
    panel.cacheDisplayInRect_toBitmapImageRep(panel.bounds(), &bitmap);
    let data = unsafe {
        bitmap.representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new())
    }
    .ok_or("Panel PNG failed")?;
    let path = directory.join("panel.png");
    if !data.writeToFile_atomically(&NSString::from_str(&path.to_string_lossy()), true) {
        return Err("Panel write failed".into());
    }
    Ok(())
}

/// Draw the menu bar chip in each style on a light and a dark bar, so layout
/// changes can be judged without touching the real status item.
pub fn render_chips(directory: &Path) -> Result<(), String> {
    use crate::bar::{BarFill, Chip, Tint};
    use objc2_app_kit::*;
    let mtm = MainThreadMarker::new().ok_or("Main thread required")?;
    NSApplication::sharedApplication(mtm)
        .setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let r = |x, y, width, height| NSRect {
        origin: NSPoint { x, y },
        size: NSSize { width, height },
    };
    // style label, stacked?, and whether the style shows the meter and text
    let styles: [(&str, bool, bool); 3] = [
        ("Smart Bar", false, true),
        ("Icon", false, false),
        ("Stacked", true, true),
    ];
    // scenario label, symbol, segments, text, tint
    let cases: [(&str, &str, u8, &str, Tint); 4] = [
        ("healthy 6GHz", "wifi", 4, "6GHz", Tint::Normal),
        ("2.4GHz fallback", "wifi.exclamationmark", 3, "2.4GHz", Tint::Orange),
        ("weak", "wifi.exclamationmark", 1, "\u{2212}81 dB", Tint::Red),
        ("hotspot", "personalhotspot", 4, "212 MB", Tint::Orange),
    ];
    let bar_h = NSStatusBar::systemStatusBar().thickness();
    let row_h = bar_h + 26.;
    let width = 900.;
    let height = row_h * (styles.len() * cases.len()) as f64 + 20.;
    let canvas = NSView::new(mtm);
    canvas.setFrame(r(0., 0., width, height));
    let background = NSBox::new(mtm);
    background.setFrame(canvas.bounds());
    background.setBoxType(NSBoxType::Custom);
    background.setBorderWidth(0.);
    background.setFillColor(&NSColor::windowBackgroundColor());
    canvas.addSubview(&background);

    let mut y = height - row_h;
    for (style_name, stacked, detail) in styles {
        for (case_name, symbol, segments, text, tint) in cases.iter().copied() {
            let label = NSTextField::labelWithString(&NSString::from_str(&format!(
                "{style_name} · {case_name}"
            )), mtm);
            label.setFont(Some(&NSFont::systemFontOfSize(11.)));
            label.setFrame(r(12., y + 4., 190., 18.));
            canvas.addSubview(&label);
            // light bar on the left, dark bar on the right
            for (index, (bg, ink)) in [
                (NSColor::colorWithWhite_alpha(0.93, 1.), NSColor::blackColor()),
                (NSColor::colorWithWhite_alpha(0.16, 1.), NSColor::whiteColor()),
            ]
            .into_iter()
            .enumerate()
            {
                let x = 220. + index as f64 * 330.;
                let strip = NSBox::new(mtm);
                strip.setFrame(r(x, y, 300., bar_h));
                strip.setBoxType(NSBoxType::Custom);
                strip.setBorderWidth(0.);
                strip.setFillColor(&bg);
                canvas.addSubview(&strip);
                let chip = Chip {
                    symbol,
                    variable: f64::from(segments) / 4.0,
                    bar: (detail && tint != Tint::Normal || detail).then_some(BarFill {
                        segments,
                        dim: false,
                    }),
                    text: detail.then(|| text.to_string()),
                    tint,
                    stacked,
                };
                let image = crate::bar::chip_image(chip);
                // Template images carry no colour: flood them with the bar's ink.
                let view = NSImageView::new(mtm);
                let size = image.size();
                view.setFrame(r(x + 12., y, size.width, bar_h));
                view.setImage(Some(&image));
                if image.isTemplate() {
                    view.setContentTintColor(Some(&ink));
                }
                canvas.addSubview(&view);
            }
            y -= row_h;
        }
    }
    let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
            NSWindow::alloc(mtm),
            r(-3000., -3000., width, height),
            NSWindowStyleMask::Borderless,
            NSBackingStoreType::Buffered,
            false,
        )
    };
    window.setAppearance(
        NSAppearance::appearanceNamed(unsafe { NSAppearanceNameDarkAqua }).as_deref(),
    );
    window.setContentView(Some(&canvas));
    let bitmap = canvas
        .bitmapImageRepForCachingDisplayInRect(canvas.bounds())
        .ok_or("Chip bitmap failed")?;
    canvas.cacheDisplayInRect_toBitmapImageRep(canvas.bounds(), &bitmap);
    let data = unsafe {
        bitmap.representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new())
    }
    .ok_or("Chip PNG failed")?;
    let path = directory.join("chips.png");
    if !data.writeToFile_atomically(&NSString::from_str(&path.to_string_lossy()), true) {
        return Err("Chip write failed".into());
    }
    println!("{}", path.display());
    Ok(())
}
