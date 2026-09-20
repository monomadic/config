//! Notification Centre banners for jobs that finish while you are looking at
//! something else.
//!
//! `UNUserNotificationCenter` needs a real bundle: asking for the current
//! centre from a bare executable throws, which is why this installs as a
//! generated `.app` rather than a binary in `~/.local/bin`. Run straight from
//! `cargo run` and there is no bundle identifier, so notifications quietly turn
//! themselves off instead of taking the app down.

use std::sync::atomic::{AtomicBool, Ordering};

use block2::RcBlock;
use objc2::runtime::Bool;
use objc2_foundation::{NSBundle, NSError, NSString};
use objc2_user_notifications::{
    UNAuthorizationOptions, UNMutableNotificationContent, UNNotificationRequest,
    UNUserNotificationCenter,
};

static AUTHORIZED: AtomicBool = AtomicBool::new(false);

pub struct Notifier {
    enabled: bool,
}

impl Notifier {
    /// Asks for permission once, at startup. The answer arrives asynchronously;
    /// until it does, posts are dropped rather than queued — a banner about a
    /// job that finished several minutes ago is worse than no banner.
    pub fn new() -> Self {
        if NSBundle::mainBundle().bundleIdentifier().is_none() {
            eprintln!("job-folder: no bundle identifier, notifications disabled");
            return Self { enabled: false };
        }

        let handler = RcBlock::new(|granted: Bool, error: *mut NSError| {
            AUTHORIZED.store(granted.as_bool(), Ordering::Relaxed);
            if !error.is_null() {
                eprintln!("job-folder: notification authorization failed");
            }
        });
        UNUserNotificationCenter::currentNotificationCenter()
            .requestAuthorizationWithOptions_completionHandler(
                UNAuthorizationOptions::Alert | UNAuthorizationOptions::Sound,
                &handler,
            );

        Self { enabled: true }
    }

    pub fn post(&self, title: &str, body: &str) {
        if !self.enabled || !AUTHORIZED.load(Ordering::Relaxed) {
            return;
        }

        let center = UNUserNotificationCenter::currentNotificationCenter();
        let content = UNMutableNotificationContent::new();
        content.setTitle(&NSString::from_str(title));
        content.setBody(&NSString::from_str(body));

        // A fresh identifier each time: reusing one replaces the banner already
        // on screen, and two jobs finishing back to back should produce two.
        let id = NSString::from_str(&format!("job-folder-{}", next_id()));
        let request =
            UNNotificationRequest::requestWithIdentifier_content_trigger(&id, &content, None);
        center.addNotificationRequest_withCompletionHandler(&request, None);
    }
}

fn next_id() -> u64 {
    use std::sync::atomic::AtomicU64;
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
