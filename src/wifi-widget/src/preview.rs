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
