use crate::{
    bar::{self, BarFill, Chip, Tint},
    login,
};
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::Retained,
    runtime::{ProtocolObject, Sel},
    sel,
};
use objc2_app_kit::{
    NSAccessibility, NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate,
    NSCellImagePosition, NSControlStateValueOff, NSControlStateValueOn, NSMenu, NSMenuDelegate,
    NSMenuItem, NSStatusBar, NSStatusItem, NSVariableStatusItemLength, NSWorkspace,
    NSWorkspaceDidWakeNotification,
};
use objc2_core_location::{CLAuthorizationStatus, CLLocationManager, CLLocationManagerDelegate};
use objc2_foundation::{
    NSBundle, NSNotification, NSObject, NSObjectProtocol, NSRunLoop, NSRunLoopCommonModes,
    NSString, NSTimer, NSURL, NSUserDefaults,
};
use std::{
    cell::RefCell,
    time::{Duration, Instant},
};
use wifi_widget::{
    events::{self, Monitor},
    model::{LinkHealth, Snapshot, State, Store},
    probe::ProbeStatus,
    runtime::Probes,
    settings::{self, Style},
    signal::{Hysteresis, Signal},
    wifi,
};

struct Ui {
    status: Retained<NSStatusItem>,
    card: Retained<crate::card::Card>,
    card_item: Retained<NSMenuItem>,
    known: Vec<(Retained<NSMenuItem>, Retained<crate::row::KnownRow>)>,
    scan: wifi::Scan,
    saved_names: Vec<String>,
    known_empty: Retained<NSMenuItem>,
    location_item: Retained<NSMenuItem>,
    login_item: Retained<NSMenuItem>,
    portal_item: Retained<NSMenuItem>,
    refresh_item: Retained<objc2_app_kit::NSButton>,
    scan_spinner: Retained<objc2_app_kit::NSProgressIndicator>,
    styles: Vec<Retained<NSMenuItem>>,
    message: Retained<NSMenuItem>,
    location: Retained<CLLocationManager>,
    _monitor: Monitor,
    store: Store,
    probes: Probes,
    hysteresis: Hysteresis,
    style: Style,
    open: bool,
    next_poll: Instant,
    rapid_until: Instant,
    last_chip: String,
    bundled: bool,
    login_enabled: bool,
    diagnostics_at: Option<Instant>,
}
define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = RefCell<Option<Ui>>]
    struct Widget;
    unsafe impl NSObjectProtocol for Widget {}
    unsafe impl NSApplicationDelegate for Widget {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn launched(&self, _note: &NSNotification) {
            let manager = self.ivars().borrow().as_ref().map(|ui| (ui.location.clone(),ui.bundled));
            if let Some((manager,true)) = manager {
                unsafe { if manager.authorizationStatus() == CLAuthorizationStatus::NotDetermined { manager.requestWhenInUseAuthorization(); } }
            }
        }
    }
    unsafe impl CLLocationManagerDelegate for Widget {
        #[unsafe(method(locationManagerDidChangeAuthorization:))]
        unsafe fn authorization(&self, _manager: &CLLocationManager) { events::changed(); }
    }
    unsafe impl NSMenuDelegate for Widget {
        #[unsafe(method(menuWillOpen:))]
        fn menu_open(&self, _menu: &NSMenu) {
            if let Some(ui) = self.ivars().borrow_mut().as_mut() { ui.open = true; ui.next_poll = Instant::now(); ui.saved_names = wifi::known_networks(); ui.scan.start_if_stale(); ui.login_enabled = ui.bundled && login::enabled(); }
            self.refresh();
        }
        #[unsafe(method(menuDidClose:))]
        fn menu_close(&self, _menu: &NSMenu) {
            if let Some(ui) = self.ivars().borrow_mut().as_mut() { ui.open = false; }
        }
    }
    impl Widget {
        #[unsafe(method(tick:))]
        fn tick(&self, _timer: &NSTimer) { self.refresh(); }
        #[unsafe(method(woke:))]
        fn woke(&self, _note: &NSNotification) { events::changed(); }
        #[unsafe(method(refreshAction:))]
        fn refresh_action(&self, _sender: &NSMenuItem) {
            if let Some(ui) = self.ivars().borrow_mut().as_mut() {
                if ui.scan.busy() { return; } ui.saved_names = wifi::known_networks(); ui.scan.start(); ui.next_poll = Instant::now(); ui.probes.schedule(Instant::now(),Duration::ZERO);
            }
            self.refresh();
        }
        #[unsafe(method(styleAction:))]
        fn style_action(&self, sender: &NSMenuItem) {
            if let Some(ui) = self.ivars().borrow_mut().as_mut() {
                let style = if sender.tag() == 1 { Style::Icon } else { Style::Smart };
                match settings::save(style) {
                    Ok(()) => { ui.style = style; ui.message.setHidden(true); }
                    Err(error) => { set_title(&ui.message,&format!("Could not save style: {error}")); ui.message.setHidden(false); }
                }
            }
            self.refresh();
        }
        #[unsafe(method(locationAction:))]
        fn location_action(&self, _sender: &NSMenuItem) {
            let manager = self.ivars().borrow().as_ref().map(|ui| (ui.location.clone(),ui.bundled));
            if let Some((manager,true)) = manager {
                unsafe {
                    if manager.authorizationStatus() == CLAuthorizationStatus::NotDetermined { manager.requestWhenInUseAuthorization(); }
                    else { open_url("x-apple.systempreferences:com.apple.preference.security?Privacy_LocationServices"); }
                }
            }
        }
        #[unsafe(method(loginAction:))]
        fn login_action(&self, _sender: &NSMenuItem) {
            if let Err(error) = login::toggle() {
                if let Some(ui) = self.ivars().borrow_mut().as_mut() { set_title(&ui.message,&format!("Login Items: {error}")); ui.message.setHidden(false); }
                open_url("x-apple.systempreferences:com.apple.LoginItems-Settings.extension");
            }
            if let Some(ui) = self.ivars().borrow_mut().as_mut() { ui.login_enabled = ui.bundled && login::enabled(); }
            self.refresh();
        }
        #[unsafe(method(settingsAction:))]
        fn settings_action(&self, _sender: &NSMenuItem) { open_url("x-apple.systempreferences:com.apple.wifi-settings-extension"); }
        #[unsafe(method(portalAction:))]
        fn portal_action(&self, _sender: &NSMenuItem) { open_url("http://captive.apple.com"); }
        #[unsafe(method(quitAction:))]
        fn quit_action(&self, _sender: &NSMenuItem) { NSApplication::sharedApplication(self.mtm()).terminate(None); }
    }
);
fn set_title(item: &NSMenuItem, text: &str) {
    if item.title().to_string() != text {
        item.setTitle(&NSString::from_str(text));
    }
}
fn open_url(url: &str) {
    if let Some(url) = NSURL::URLWithString(&NSString::from_str(url)) {
        NSWorkspace::sharedWorkspace().openURL(&url);
    }
}
impl Widget {
    fn new(mtm: MainThreadMarker, diagnostics: bool) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(RefCell::new(None));
        let this: Retained<Self> = unsafe { msg_send![super(this), init] };
        // Match menu-tidy's autosaved placement. Unnamed, newly created items
        // can land on the hidden side of its spacer. Seed only once: subsequent
        // launches preserve the user's Command-dragged position.
        let defaults = NSUserDefaults::standardUserDefaults();
        let position = NSString::from_str("NSStatusItem Preferred Position wifi-widget");
        if defaults.objectForKey(&position).is_none() {
            defaults.setDouble_forKey(0.0, &position);
        }
        let status =
            NSStatusBar::systemStatusBar().statusItemWithLength(NSVariableStatusItemLength);
        status.setAutosaveName(Some(&NSString::from_str("wifi-widget")));
        let menu = NSMenu::new(mtm);
        menu.setAutoenablesItems(false);
        menu.setDelegate(Some(ProtocolObject::from_ref(&*this)));
        status.setMenu(Some(&menu));
        let top_padding = objc2_app_kit::NSView::new(mtm);
        top_padding.setFrame(objc2_foundation::NSRect::new(
            objc2_foundation::NSPoint::new(0., 0.),
            objc2_foundation::NSSize::new(320., 4.),
        ));
        let padding_item = this.item(&menu, "", None);
        padding_item.setView(Some(&top_padding));
        padding_item.setEnabled(false);
        let card = crate::card::Card::new(
            crate::card::Content::from_snapshot(
                &Store::default().snapshot(Instant::now()),
                Instant::now(),
            ),
            mtm,
        );
        let card_item = this.item(&menu, "Wi-Fi status", None);
        card_item.setView(Some(&card));
        card_item.setEnabled(true);
        let (section, refresh_item, scan_spinner) =
            crate::row::section(Some(this.as_ref()), Some(sel!(refreshAction:)), mtm);
        let heading = this.item(&menu, "Nearby Networks", None);
        heading.setView(Some(&section));
        heading.setEnabled(true);
        let mut known = Vec::new();
        for _ in 0..8 {
            let row = crate::row::KnownRow::new(mtm);
            let item = this.item(&menu, "Nearby network", Some(sel!(settingsAction:)));
            item.setView(Some(&row));
            item.setHidden(true);
            known.push((item, row));
        }
        let known_empty = this.item(&menu, "Scanning for nearby networks…", None);
        menu.addItem(&NSMenuItem::separatorItem(mtm));
        let portal_item = this.item(&menu, "Open Login Page", Some(sel!(portalAction:)));
        portal_item.setHidden(true);
        let location_item = this.item(
            &menu,
            "Allow Location for Network Names…",
            Some(sel!(locationAction:)),
        );
        let style_menu = NSMenu::new(mtm);
        style_menu.setAutoenablesItems(false);
        let mut styles = Vec::new();
        for (index, title) in ["Smart Bar", "Icon"].iter().enumerate() {
            let item = this.item(&style_menu, title, Some(sel!(styleAction:)));
            item.setTag(index as isize);
            styles.push(item);
        }
        let style_item = this.item(&menu, "Style", None);
        style_item.setEnabled(true);
        style_item.setSubmenu(Some(&style_menu));
        let login_item = this.item(&menu, "Open at Login", Some(sel!(loginAction:)));
        this.item(&menu, "Wi-Fi Settings…", Some(sel!(settingsAction:)));
        let message = this.item(&menu, "", None);
        message.setHidden(true);
        menu.addItem(&NSMenuItem::separatorItem(mtm));
        let quit = this.item(&menu, "Quit WiFi Widget", Some(sel!(quitAction:)));
        quit.setKeyEquivalent(&NSString::from_str("q"));
        let location = unsafe { CLLocationManager::new() };
        unsafe {
            location.setDelegate(Some(ProtocolObject::from_ref(&*this)));
        }
        let bundled = NSBundle::mainBundle()
            .bundlePath()
            .to_string()
            .ends_with(".app");
        let now = Instant::now();
        *this.ivars().borrow_mut() = Some(Ui {
            status,
            card,
            card_item,
            known,
            scan: {
                let mut scan = wifi::Scan::default();
                scan.start();
                scan
            },
            saved_names: Vec::new(),
            known_empty,
            location_item,
            login_item,
            portal_item,
            refresh_item,
            scan_spinner,
            styles,
            message,
            location,
            _monitor: Monitor::start(),
            store: Store::default(),
            probes: Probes::new(now),
            hysteresis: Hysteresis::default(),
            style: settings::load(),
            open: false,
            next_poll: now,
            rapid_until: now + Duration::from_secs(30),
            last_chip: String::new(),
            bundled,
            login_enabled: bundled && login::enabled(),
            diagnostics_at: diagnostics.then_some(now + Duration::from_secs(1)),
        });
        unsafe {
            NSWorkspace::sharedWorkspace()
                .notificationCenter()
                .addObserver_selector_name_object(
                    &this,
                    sel!(woke:),
                    Some(NSWorkspaceDidWakeNotification),
                    None,
                );
        }
        this
    }
    fn item(&self, menu: &NSMenu, title: &str, selector: Option<Sel>) -> Retained<NSMenuItem> {
        let item = NSMenuItem::new(self.mtm());
        item.setTitle(&NSString::from_str(title));
        item.setEnabled(selector.is_some());
        if let Some(selector) = selector {
            unsafe {
                item.setTarget(Some(self.as_ref()));
                item.setAction(Some(selector));
            }
        }
        menu.addItem(&item);
        item
    }
    fn refresh(&self) {
        crate::join::tick(self.mtm());
        let now = Instant::now();
        let (read, invalidate) = events::take();
        let mut state = self.ivars().borrow_mut();
        let Some(ui) = state.as_mut() else {
            return;
        };
        if invalidate {
            ui.store.invalidate_link();
            ui.hysteresis = Hysteresis::default();
            ui.rapid_until = now + Duration::from_secs(30);
            ui.probes.schedule(now, Duration::from_secs(2));
        }
        if read || now >= ui.next_poll {
            let generation = ui.store.generation();
            ui.store.update_link(wifi::read(), now);
            if generation != ui.store.generation() {
                ui.hysteresis = Hysteresis::default();
                ui.rapid_until = now + Duration::from_secs(30);
                ui.probes.schedule(now, Duration::from_secs(2));
            }
            ui.next_poll = now
                + Duration::from_secs(if ui.open || now < ui.rapid_until {
                    1
                } else {
                    5
                });
        }
        let associated = matches!(
            ui.store.snapshot(now).link,
            LinkHealth::Associated | LinkHealth::Redacted
        );
        ui.probes.tick(&mut ui.store, associated, now);
        if ui.diagnostics_at.is_some_and(|at| now >= at) {
            ui.diagnostics_at = None;
            eprintln!(
                "status.visible={} status.length={}",
                ui.status.isVisible(),
                ui.status.length()
            );
            if let Some(button) = ui.status.button(self.mtm()) {
                eprintln!(
                    "button.frame={:?} image.size={:?}",
                    button.frame(),
                    button.image().map(|i| i.size())
                );
                if let Some(window) = button.window() {
                    eprintln!(
                        "window.visible={} window.frame={:?} screen.frame={:?}",
                        window.isVisible(),
                        window.frame(),
                        window.screen().map(|s| s.frame())
                    );
                }
            }
        }
        let snapshot = ui.store.snapshot(now);
        let tier = ui.hysteresis.update(snapshot.signal, now);
        let segments = tier.map(|t| t.segments()).unwrap_or(0);
        let bar_state = wifi_widget::model::headline_state_for_tier(&snapshot, now, tier);
        let text = tag(&snapshot, bar_state);
        let chip_key = format!(
            "{:?}/{:?}/{segments}/{text}/{associated}",
            ui.style, bar_state
        );
        if chip_key != ui.last_chip {
            let alarm = !matches!(
                bar_state,
                State::Healthy | State::Unknown | State::WifiOff | State::Disconnected
            );
            let symbol = match bar_state {
                State::WifiOff | State::Disconnected => "wifi.slash",
                State::MeteredFallback => "personalhotspot",
                _ if alarm => "wifi.exclamationmark",
                _ => "wifi",
            };
            let tint = if ui.style == Style::Icon {
                Tint::Normal
            } else {
                match bar_state {
                    State::NoInternet | State::Weak => Tint::Red,
                    State::LoginRequired | State::MeteredFallback | State::BandFallback => {
                        Tint::Orange
                    }
                    _ => Tint::Normal,
                }
            };
            let show_bar = ui.style == Style::Smart && associated && bar_state != State::NoInternet;
            let chip = Chip {
                symbol,
                variable: f64::from(segments) / 4.0,
                bar: show_bar.then_some(BarFill {
                    segments,
                    dim: snapshot.signal.is_none(),
                }),
                text: (ui.style == Style::Smart && !text.is_empty()).then_some(text),
                tint,
            };
            if let Some(button) = ui.status.button(self.mtm()) {
                button.setImage(Some(&bar::chip_image(chip)));
                button.setImagePosition(NSCellImagePosition::ImageOnly);
            }
            ui.last_chip = chip_key;
        }
        let connected = snapshot
            .reading
            .as_ref()
            .and_then(|r| r.value.ssid.as_deref());
        ui.scan.tick();
        let names: Vec<_> = ui
            .scan
            .visible_names()
            .into_iter()
            .filter(|name| Some(name.as_str()) != connected)
            .collect();
        if let Some(menu) = unsafe { ui.known_empty.menu() } {
            while ui.known.len() < names.len() {
                let row = crate::row::KnownRow::new(self.mtm());
                let item = NSMenuItem::new(self.mtm());
                item.setView(Some(&row));
                item.setEnabled(true);
                menu.insertItem_atIndex(&item, menu.indexOfItem(&ui.known_empty));
                ui.known.push((item, row));
            }
        }
        ui.known_empty
            .setTitle(&NSString::from_str(if ui.scan.busy() {
                "Scanning for nearby networks…"
            } else {
                "No other visible networks found"
            }));
        for (index, (item, row)) in ui.known.iter().enumerate() {
            item.setHidden(index >= names.len());
            if let Some(name) = names.get(index) {
                row.set_name(name);
                row.set_reading(ui.scan.reading(name));
                row.set_security(ui.scan.security(name));
                row.set_cached(ui.scan.is_cached());
                row.set_saved(ui.saved_names.contains(name));
            }
        }
        ui.known_empty.setHidden(!names.is_empty());
        let mut content = crate::card::Content::from_snapshot(&snapshot, now);
        if content.associated {
            let reading = snapshot.reading.as_ref().map(|r| &r.value);
            (content.ip, content.gateway) = wifi_widget::addresses::read(
                reading.and_then(|r| r.interface.as_deref()),
                &format!(
                    "{}:{}",
                    ui.store.generation(),
                    reading.and_then(|r| r.bssid.as_deref()).unwrap_or("")
                ),
            );
        }
        let lines = summary(&snapshot, now);
        if ui.card.update(content, now)
            && let Some(menu) = ui.status.menu(self.mtm())
        {
            menu.itemChanged(&ui.card_item);
        }
        if let Some(button) = ui.status.button(self.mtm()) {
            let label = NSString::from_str(&format!("{}; {}; {}", lines[0], lines[1], lines[3]));
            button.setToolTip(Some(&label));
            button.setAccessibilityLabel(Some(&label));
        }
        let auth = unsafe { ui.location.authorizationStatus() };
        ui.location_item.setHidden(matches!(
            auth,
            CLAuthorizationStatus::AuthorizedAlways | CLAuthorizationStatus::AuthorizedWhenInUse
        ));
        ui.location_item.setEnabled(ui.bundled);
        set_title(
            &ui.location_item,
            if !ui.bundled {
                "Open the .app to Allow Location"
            } else if auth == CLAuthorizationStatus::NotDetermined {
                "Allow Location for Network Names…"
            } else {
                "Location Access in System Settings…"
            },
        );
        ui.login_item.setEnabled(ui.bundled);
        ui.login_item.setState(if ui.login_enabled {
            NSControlStateValueOn
        } else {
            NSControlStateValueOff
        });
        ui.portal_item
            .setHidden(snapshot.headline != State::LoginRequired);
        ui.refresh_item
            .setToolTip(Some(&NSString::from_str(if ui.scan.busy() {
                "Scanning nearby networks…"
            } else {
                "Refresh Wi-Fi readings and nearby networks"
            })));
        let scanning = ui.scan.busy();
        ui.refresh_item.setEnabled(!scanning);
        ui.refresh_item.setHidden(scanning);
        if ui.scan_spinner.isHidden() == scanning {
            ui.scan_spinner.setHidden(!scanning);
            if scanning {
                unsafe {
                    ui.scan_spinner.startAnimation(None);
                }
            } else {
                unsafe {
                    ui.scan_spinner.stopAnimation(None);
                }
            }
        }
        for (index, item) in ui.styles.iter().enumerate() {
            item.setState(if (index == 1) == (ui.style == Style::Icon) {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            });
        }
    }
}
fn tag(s: &Snapshot, state: State) -> String {
    match state {
        State::WifiOff | State::Disconnected | State::NoInternet => String::new(),
        State::LoginRequired => "Login".into(),
        State::Weak => s
            .rssi
            .as_ref()
            .map(|r| format!("−{} dBm", r.value.abs()))
            .unwrap_or_default(),
        State::MeteredFallback => "Metered".into(),
        _ => s
            .reading
            .as_ref()
            .and_then(|r| r.value.band)
            .map(|b| b.label().to_string())
            .unwrap_or_default(),
    }
}
fn summary(s: &Snapshot, now: Instant) -> Vec<String> {
    let state = match s.headline {
        State::WifiOff => "Off",
        State::Disconnected => "Disconnected",
        State::NoInternet => "No Internet",
        State::LoginRequired => "Login Required",
        State::Weak => "Weak Signal",
        State::MeteredFallback => "Metered",
        State::BandFallback => "Band Fallback",
        State::Healthy => "Connected",
        State::Unknown => {
            if matches!(s.link, LinkHealth::Associated | LinkHealth::Redacted) {
                if s.signal.is_none() {
                    "Connected · signal unavailable"
                } else if s
                    .signal
                    .is_some_and(|signal| signal.tier() == wifi_widget::signal::Tier::Fair)
                {
                    "Connected · fair signal"
                } else {
                    "Connected · internet not confirmed"
                }
            } else {
                "Wi-Fi unavailable"
            }
        }
    };
    let mut lines = vec![
        format!("Wi-Fi — {state}"),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
    ];
    if !matches!(s.link, LinkHealth::Associated | LinkHealth::Redacted) {
        return lines;
    }
    if let Some(reading) = &s.reading {
        let r = &reading.value;
        // Keep control characters in network names from creating visual rows.
        lines[1] = r
            .ssid
            .as_ref()
            .map(|s| s.chars().filter(|c| !c.is_control()).take(60).collect())
            .unwrap_or_else(|| "Wi-Fi · network name unavailable".into());
        lines[2] = [
            r.phy.map(str::to_string),
            r.band.map(|b| b.label().into()),
            r.channel.map(|c| format!("ch {c}")),
            r.width_mhz.map(|w| format!("{w} MHz")),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" · ");
        lines[4] = format!(
            "Signal {}   Noise {}   {}",
            dbm(&s.rssi, now),
            dbm(&s.noise, now),
            r.rate_mbps
                .map(|r| format!("{r:.0} Mbps"))
                .unwrap_or_else(|| "Rate unknown".into())
        );
        lines[6] = r
            .mac
            .as_ref()
            .map(|m| format!("MAC  {m}"))
            .unwrap_or_default();
        lines[7] = r
            .bssid
            .as_ref()
            .map(|m| format!("Node  {m}"))
            .unwrap_or_default();
    }
    lines[3] = match s.signal {
        Some(Signal::Snr(v)) => format!("{v} dB SNR · {:?}", s.signal.unwrap().tier()),
        Some(Signal::Rssi(v)) => format!(
            "{v} dBm RSSI · {:?} · noise unavailable",
            s.signal.unwrap().tier()
        ),
        None => "Signal unavailable · waiting for a fresh measurement".into(),
    };
    lines[5] = format!(
        "Internet  {}",
        match s
            .probe
            .as_ref()
            .filter(|p| p.age(now) < Duration::from_secs(65))
            .map(|p| &p.value)
        {
            Some(ProbeStatus::Reachable { latency }) =>
                format!("Endpoint reachable · {} ms", latency.as_millis()),
            Some(ProbeStatus::Captive { .. }) => "Login required".into(),
            Some(p) if p.is_failure() =>
                if s.failures >= 2 {
                    "Endpoint unreachable".into()
                } else {
                    "Check failed · awaiting confirmation".into()
                },
            Some(ProbeStatus::UnexpectedResponse) => "Unexpected response".into(),
            _ => "Not checked yet".into(),
        }
    );
    lines
}
fn dbm(sample: &Option<wifi_widget::model::Sample<i32>>, now: Instant) -> String {
    sample
        .as_ref()
        .and_then(|s| s.fresh_value(now))
        .map(|v| format!("{v} dBm"))
        .unwrap_or_else(|| "—".into())
}
pub fn run(diagnostics: bool) {
    let mtm = MainThreadMarker::new().expect("AppKit must run on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    let widget = Widget::new(mtm, diagnostics);
    app.setDelegate(Some(ProtocolObject::from_ref(&*widget)));
    widget.refresh();
    unsafe {
        let timer = NSTimer::timerWithTimeInterval_target_selector_userInfo_repeats(
            0.25,
            &widget,
            sel!(tick:),
            None,
            true,
        );
        NSRunLoop::mainRunLoop().addTimer_forMode(&timer, NSRunLoopCommonModes);
        app.run();
        timer.invalidate();
    }
}
