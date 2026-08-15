//! Render a menu's worth of job rows to a PNG, for looking at the design
//! without waiting for a real job to run.
//!
//! ```bash
//! cargo run --example render_rows -- /tmp/rows.png [light|dark]
//! ```
//!
//! It draws the rows exactly as the menu does — same views, same layout, same
//! fonts — into an offscreen bitmap, so what comes out is what a menu bar
//! would show.

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use job_core::observe::{Outcome, Run, Snapshot, State};
use job_core::row::{self, JobRow};
use objc2::{AnyThread, MainThreadMarker};
use objc2::rc::Retained;
use objc2_app_kit::{
    NSAffineTransformNSAppKitAdditions, NSAppearanceCustomization, NSBezierPath,
    NSAppearance, NSAppearanceNameDarkAqua, NSAppearanceNameAqua, NSBitmapImageFileType,
    NSBitmapImageRep, NSColor, NSDeviceRGBColorSpace, NSGraphicsContext,
};
use objc2_foundation::{NSAffineTransform, NSDictionary, NSPoint, NSRect, NSSize};

fn ago(seconds: u64) -> SystemTime {
    SystemTime::now() - Duration::from_secs(seconds)
}

/// A folder mid-encode: one job running with progress, a queue behind it, and
/// both kinds of outcome above.
fn sample(quiet: bool) -> Snapshot {
    let queued = |name: &str, at: u64| Run {
        name: name.to_string(),
        dir: PathBuf::from(format!("/Volumes/Jobs/_ready/2026-{name}")),
        state: State::Ready,
        started: Some(ago(at)),
        last_line: None,
        last_output: None,
        progress: None,
        status: None,
        local: false,
    };
    Snapshot {
        root: Some(job_core::observe::Root::new("/Volumes/Jobs")),
        connected: true,
        inbox: Vec::new(),
        jobs: vec![Run {
            name: "my night collection".to_string(),
            // $RENDER_ROWS_RUN_DIR points this at a folder holding a real
            // `<name>.log`, so the log button shows up in the render.
            dir: std::env::var_os("RENDER_ROWS_RUN_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from("/Volumes/Jobs/_running/20260806-041500-my night collection")
                }),
            state: State::Running,
            status: None,
            local: false,
            started: Some(ago(5883)),
            last_line: Some(if quiet {
                "encoding pass 2 of 2".to_string()
            } else {
                "45% · frame 64512 · 18.2 fps · eta 1:52:10".to_string()
            }),
            last_output: Some(if quiet { ago(2460) } else { ago(3) }),
            progress: if quiet { None } else { Some(0.45) },
        },
        // Suspended mid-encode, and a claimed job with nothing running it —
        // the two states the row used to report in small grey text at the far
        // right of itself, where nobody found them.
        Run {
            name: "silvialia dawn set".to_string(),
            dir: PathBuf::from("/Volumes/Jobs/_paused/20260806-030000-silvialia"),
            state: State::Paused,
            status: None,
            local: false,
            started: Some(ago(4200)),
            last_line: Some("62% · frame 41003 · 12.7 fps".to_string()),
            last_output: Some(ago(900)),
            progress: Some(0.62),
        },
        Run {
            name: "orphaned overnight batch".to_string(),
            dir: PathBuf::from("/Volumes/Jobs/_running/20260805-234500-orphan"),
            state: State::Running,
            status: None,
            local: false,
            started: Some(ago(40000)),
            last_line: Some("14% · frame 9210 · 9.1 fps".to_string()),
            last_output: Some(ago(36000)),
            progress: Some(0.14),
        },
        queued("beach walk 4k", 300),
        queued("interview b roll", 200)],
        recent: vec![
            Outcome {
                name: "pov gorgeous 1080p h264".to_string(),
                dir: PathBuf::from("/Volumes/Jobs/_ok/20260805-093000-pov"),
                ok: true,
                finished: epoch(ago(86400)),
                started: Some(ago(86400 + 1320)),
            },
            Outcome {
                name: "broken clip".to_string(),
                dir: PathBuf::from("/Volumes/Jobs/_failed/20260805-101500-broken"),
                ok: false,
                finished: epoch(ago(7200)),
                started: Some(ago(7200 + 240)),
            },
        ],
        errors: 1,
    }
}

fn epoch(time: SystemTime) -> i64 {
    time.duration_since(SystemTime::UNIX_EPOCH)
        .map(|since| since.as_secs() as i64)
        .unwrap_or(0)
}

fn main() {
    let mtm = MainThreadMarker::new().expect("must run on the main thread");
    let mut args = std::env::args().skip(1);
    let out = args.next().unwrap_or_else(|| "/tmp/rows.png".to_string());
    let dark = args.next().map(|mode| mode == "dark").unwrap_or(true);
    let quiet = args.next().map(|mode| mode == "quiet").unwrap_or(false);

    // Set on the views, not globally: label and system colours resolve
    // against the drawing view's effective appearance, which is exactly how
    // they resolve inside a real menu.
    let appearance = NSAppearance::appearanceNamed(unsafe {
        if dark {
            NSAppearanceNameDarkAqua
        } else {
            NSAppearanceNameAqua
        }
    });

    let sections = row::sections(&sample(quiet), 5, 5);
    let layout = row::layout(sections.iter().flat_map(|section| section.rows.iter()));
    let pad = 10.0;

    // One unbroken list, the way the menu draws it: the sections carry
    // neither header text nor a separator between them.
    // Hover can't be reached without a mouse, and it is half of what a button
    // communicates, so the render fakes one over the first row's pause button.
    let mut items: Vec<(Option<Retained<JobRow>>, f64)> = Vec::new();
    for section in &sections {
        for spec in &section.rows {
            let view = JobRow::new(spec.clone(), &layout, mtm);
            view.setAppearance(appearance.as_deref());
            if items.is_empty() {
                view.preview_pointer(Some(0));
            }
            items.push((Some(view), layout.height()));
        }
    }

    let total: f64 = items.iter().map(|(_, height)| height).sum();
    let canvas = NSSize {
        width: layout.width() + pad * 2.0,
        height: total + pad * 2.0,
    };

    let Some(bitmap) = (unsafe { NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bytesPerRow_bitsPerPixel(
        NSBitmapImageRep::alloc(),
        std::ptr::null_mut(),
        (canvas.width * 2.0) as isize,
        (canvas.height * 2.0) as isize,
        8,
        4,
        true,
        false,
        NSDeviceRGBColorSpace,
        0,
        0,
    ) }) else {
        eprintln!("could not allocate the bitmap");
        return;
    };
    bitmap.setSize(canvas);

    let Some(context) = NSGraphicsContext::graphicsContextWithBitmapImageRep(&bitmap) else {
        eprintln!("could not make a context");
        return;
    };
    NSGraphicsContext::saveGraphicsState_class();
    NSGraphicsContext::setCurrentContext(Some(&context));

    // Stand-in for the menu's own material, so contrast reads honestly.
    let backdrop = if dark {
        NSColor::colorWithSRGBRed_green_blue_alpha(0.16, 0.16, 0.17, 1.0)
    } else {
        NSColor::colorWithSRGBRed_green_blue_alpha(0.96, 0.96, 0.97, 1.0)
    };
    backdrop.set();
    objc2_app_kit::NSRectFill(NSRect {
        origin: NSPoint { x: 0.0, y: 0.0 },
        size: canvas,
    });

    // Each row draws in its own coordinate space, so the context is walked
    // down the canvas between them rather than the frames being moved.
    let mut y = canvas.height - pad;
    for (view, height) in &items {
        y -= height;
        match view {
            Some(view) => {
                NSGraphicsContext::saveGraphicsState_class();
                let shift = NSAffineTransform::transform();
                shift.translateXBy_yBy(pad, y);
                shift.concat();
                view.displayRectIgnoringOpacity_inContext(view.bounds(), &context);
                NSGraphicsContext::restoreGraphicsState_class();
            }
            None => {
                NSColor::secondaryLabelColor()
                    .colorWithAlphaComponent(0.35)
                    .set();
                NSBezierPath::fillRect(NSRect {
                    origin: NSPoint {
                        x: pad + 10.0,
                        y: (y + height / 2.0).round(),
                    },
                    size: NSSize {
                        width: canvas.width - pad * 2.0 - 20.0,
                        height: 1.0,
                    },
                });
            }
        }
    }

    NSGraphicsContext::restoreGraphicsState_class();

    let Some(data) = (unsafe {
        bitmap.representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new())
    }) else {
        eprintln!("could not encode the png");
        return;
    };
    let bytes = data.to_vec();
    std::fs::write(&out, bytes).expect("write png");
    println!("wrote {out}");
}
