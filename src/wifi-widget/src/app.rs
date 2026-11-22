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
    cell::{Cell, RefCell},
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

/// Nearby networks shown in the menu, strongest first.
const MAX_NETWORKS: usize = 6;
/// Tick fast only while the menu is open; closed, the bar needs at most 1 Hz.
const TICK_OPEN: f64 = 0.25;
const TICK_CLOSED: f64 = 1.0;
/// Closed but connecting: fast enough to step the glyph's arcs.
const TICK_CONNECTING: f64 = 0.4;
type RowKey = (
    String,
    Option<(wifi::Band, i32)>,
    Option<&'static str>,
    bool,
);
struct Ui {
    timer: Option<Retained<NSTimer>>,
    interval: f64,
    /// When the current "connecting" spell began; drives the glyph animation.
    connecting_since: Option<Instant>,
    rows: Vec<Option<RowKey>>,
    tooltip: String,
    status: Retained<NSStatusItem>,
    card: Retained<crate::card::Card>,
    card_item: Retained<NSMenuItem>,
    known: Vec<(Retained<NSMenuItem>, Retained<crate::row::KnownRow>)>,
    scan: wifi::Scan,
    saved_names: Vec<String>,
    known_empty: Retained<NSMenuItem>,
    known_empty_label: Retained<objc2_app_kit::NSTextField>,
    /// Names in the rows, top to bottom. Kept across scan batches so rows
    /// stay put under the pointer; reset each time the menu opens.
    order: Vec<String>,
    location_item: Retained<NSMenuItem>,
    login_item: Retained<NSMenuItem>,
    portal_item: Retained<NSMenuItem>,
    refresh_item: Retained<objc2_app_kit::NSButton>,
    scan_spinner: Retained<objc2_app_kit::NSProgressIndicator>,
    styles: Vec<Retained<NSMenuItem>>,
    style_detail: Retained<objc2_app_kit::NSTextField>,
    message: Retained<NSMenuItem>,
    location: Retained<CLLocationManager>,
    _monitor: Monitor,
    store: Store,
    probes: Probes,
    hysteresis: Hysteresis,
    style: Style,
    next_poll: Instant,
    rapid_until: Instant,
    last_chip: String,
    bundled: bool,
    login_enabled: bool,
    diagnostics_at: Option<Instant>,
}
#[derive(Default)]
struct WidgetIvars {
    ui: RefCell<Option<Ui>>,
    menu_open: Cell<bool>,
}
define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = WidgetIvars]
    struct Widget;
    unsafe impl NSObjectProtocol for Widget {}
    unsafe impl NSApplicationDelegate for Widget {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn launched(&self, _note: &NSNotification) {
            let manager = self.ivars().ui.borrow().as_ref().map(|ui| (ui.location.clone(),ui.bundled));
            if let Some((manager,true)) = manager {
                unsafe { if manager.authorizationStatus() == CLAuthorizationStatus::NotDetermined { manager.requestWhenInUseAuthorization(); } }
            }
        }
    }
    unsafe impl CLLocationManagerDelegate for Widget {
        #[unsafe(method(locationManagerDidChangeAuthorization:))]
        unsafe fn authorization(&self, _manager: &CLLocationManager) {
            // A grant makes SSIDs visible; pre-grant results are worthless.
            if let Some(ui) = self.ivars().ui.borrow_mut().as_mut() { ui.scan.start(); }
            events::changed();
        }
    }
    unsafe impl NSMenuDelegate for Widget {
        #[unsafe(method(menuWillOpen:))]
        fn menu_open(&self, _menu: &NSMenu) {
            self.ivars().menu_open.set(true);
            if let Some(ui) = self.ivars().ui.borrow_mut().as_mut() { ui.order.clear(); ui.next_poll = Instant::now(); wifi::refresh_known_networks(); ui.scan.start_if_stale(); ui.login_enabled = ui.bundled && login::enabled(); }
            self.set_timer(TICK_OPEN);
            self.refresh();
        }
        #[unsafe(method(menuDidClose:))]
        fn menu_close(&self, _menu: &NSMenu) {
            // AppKit may close the menu during a view update. The next refresh
            // adjusts the timer without borrowing the UI from this callback.
            self.ivars().menu_open.set(false);
        }
    }
    impl Widget {
        #[unsafe(method(tick:))]
        fn tick(&self, _timer: &NSTimer) { self.refresh(); }
        #[unsafe(method(woke:))]
        fn woke(&self, _note: &NSNotification) { events::changed(); }
        #[unsafe(method(refreshAction:))]
        fn refresh_action(&self, _sender: &NSMenuItem) {
            if let Some(ui) = self.ivars().ui.borrow_mut().as_mut() {
                if ui.scan.busy() { return; } wifi::refresh_known_networks(); ui.scan.start(); ui.next_poll = Instant::now(); ui.probes.schedule(Instant::now(),Duration::ZERO);
            }
            self.refresh();
        }
        #[unsafe(method(styleAction:))]
        fn style_action(&self, sender: &NSMenuItem) {
            if let Some(ui) = self.ivars().ui.borrow_mut().as_mut() {
                let style = match sender.tag() { 1 => Style::Icon, 2 => Style::Stacked, _ => Style::Smart };
                match settings::save(style) {
                    Ok(()) => { ui.style = style; ui.message.setHidden(true); }
                    Err(error) => { set_title(&ui.message,&format!("Could not save style: {error}")); ui.message.setHidden(false); }
                }
            }
            self.refresh();
        }
        #[unsafe(method(locationAction:))]
        fn location_action(&self, _sender: &NSMenuItem) {
            let manager = self.ivars().ui.borrow().as_ref().map(|ui| (ui.location.clone(),ui.bundled));
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
                if let Some(ui) = self.ivars().ui.borrow_mut().as_mut() { set_title(&ui.message,&format!("Login Items: {error}")); ui.message.setHidden(false); }
                open_url("x-apple.systempreferences:com.apple.LoginItems-Settings.extension");
            }
            if let Some(ui) = self.ivars().ui.borrow_mut().as_mut() { ui.login_enabled = ui.bundled && login::enabled(); }
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
fn set_text(field: &objc2_app_kit::NSTextField, text: &str) {
    if field.stringValue().to_string() != text {
        field.setStringValue(&NSString::from_str(text));
    }
}
pub(crate) fn open_url(url: &str) {
    if let Some(url) = NSURL::URLWithString(&NSString::from_str(url)) {
        NSWorkspace::sharedWorkspace().openURL(&url);
    }
}
impl Widget {
    fn new(mtm: MainThreadMarker, diagnostics: bool) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(WidgetIvars::default());
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
        for _ in 0..MAX_NETWORKS {
            let row = crate::row::KnownRow::new(mtm);
            let item = this.item(&menu, "Nearby network", Some(sel!(joinNetwork:)));
            unsafe { item.setTarget(Some(row.as_ref())); }
            item.setView(Some(&row));
            item.setHidden(true);
            known.push((item, row));
        }
        let known_empty = this.item(&menu, "No other visible networks found", None);
        let (empty_view, known_empty_label) =
            crate::row::placeholder_row("No other visible networks found", mtm);
        known_empty.setView(Some(&empty_view));
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
        for (index, title) in ["Smart Bar", "Icon", "Stacked"].iter().enumerate() {
            let item = this.item(&style_menu, title, Some(sel!(styleAction:)));
            item.setTag(index as isize);
            styles.push(item);
        }
        let style_item = this.item(&menu, "Style", None);
        let (style_view, style_detail) = crate::row::style_row(mtm);
        style_item.setView(Some(&style_view));
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
        *this.ivars().ui.borrow_mut() = Some(Ui {
            timer: None,
            interval: 0.0,
            connecting_since: None,
            rows: vec![None; MAX_NETWORKS],
            tooltip: String::new(),
            status,
            card,
            card_item,
            known,
            scan: {
                let mut scan = wifi::Scan::default();
                scan.start();
                wifi::refresh_known_networks();
                scan
            },
            saved_names: Vec::new(),
            known_empty,
            known_empty_label,
            order: Vec::new(),
            location_item,
            login_item,
            portal_item,
            refresh_item,
            scan_spinner,
            styles,
            style_detail,
            message,
            location,
            _monitor: Monitor::start(),
            store: Store::default(),
            probes: Probes::new(now),
            hysteresis: Hysteresis::default(),
            style: settings::load(),
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
    fn set_timer(&self, interval: f64) {
        let timer = unsafe {
            NSTimer::timerWithTimeInterval_target_selector_userInfo_repeats(
                interval,
                self,
                sel!(tick:),
                None,
                true,
            )
        };
        // Let macOS coalesce wakeups with other timers.
        timer.setTolerance(interval * 0.2);
        unsafe { NSRunLoop::mainRunLoop().addTimer_forMode(&timer, NSRunLoopCommonModes) };
        let old = self.ivars().ui.borrow_mut().as_mut().and_then(|ui| {
            ui.interval = interval;
            ui.timer.replace(timer)
        });
        if let Some(old) = old {
            old.invalidate();
        }
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
        if self.ivars().ui.try_borrow().is_err() {
            return;
        }
        let connecting = self.update();
        let want = {
            let state = self.ivars().ui.borrow();
            let Some(ui) = state.as_ref() else {
                return;
            };
            let want = if self.ivars().menu_open.get() {
                TICK_OPEN
            } else if connecting {
                TICK_CONNECTING
            } else {
                TICK_CLOSED
            };
            if ui.interval == want {
                return;
            }
            want
        };
        self.set_timer(want);
    }
    /// One pass over the model and the views. Returns whether the menu bar
    /// icon is showing "connecting", which needs a faster tick to animate.
    fn update(&self) -> bool {
        crate::join::tick();
        let now = Instant::now();
        let (read, invalidate, quality) = events::take();
        let mut state = self.ivars().ui.borrow_mut();
        let Some(ui) = state.as_mut() else {
            return false;
        };
        if invalidate {
            ui.store.invalidate_link();
            ui.hysteresis = Hysteresis::default();
            ui.rapid_until = now + Duration::from_secs(30);
            ui.probes.schedule(now, Duration::from_secs(2));
        }
        if read || (quality && self.ivars().menu_open.get()) || now >= ui.next_poll {
            let generation = ui.store.generation();
            ui.store.update_link(wifi::read(), now);
            if generation != ui.store.generation() {
                ui.hysteresis = Hysteresis::default();
                ui.rapid_until = now + Duration::from_secs(30);
                ui.probes.schedule(now, Duration::from_secs(2));
            }
            ui.next_poll = now
                + Duration::from_secs(if self.ivars().menu_open.get() || now < ui.rapid_until {
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
        // Associated is not connected. Until the internet check answers (or
        // settles on a portal or a dead uplink) the icon animates its arcs
        // over an empty meter, and the same while a join is under way.
        let joining = crate::join::progress()
            .is_some_and(|j| j.phase == crate::join::Phase::Working);
        let connecting = joining
            || (associated
                && !wifi_widget::model::internet_confirmed(&snapshot, now)
                && !matches!(bar_state, State::NoInternet | State::LoginRequired));
        let frame = if connecting {
            let since = *ui.connecting_since.get_or_insert(now);
            (now.saturating_duration_since(since).as_millis() / 400 % 3) as u8 + 1
        } else {
            ui.connecting_since = None;
            0
        };
        let chip_key = format!(
            "{:?}/{:?}/{segments}/{text}/{associated}/{frame}",
            ui.style, bar_state
        );
        if chip_key != ui.last_chip {
            let alarm = !matches!(
                bar_state,
                State::Healthy | State::Unknown | State::WifiOff | State::Disconnected
            );
            let symbol = match bar_state {
                _ if connecting => "wifi",
                State::WifiOff | State::Disconnected => "wifi.slash",
                State::MeteredFallback => "personalhotspot",
                _ if alarm => "wifi.exclamationmark",
                _ => "wifi",
            };
            let tint = if ui.style == Style::Icon || connecting {
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
            let shows_detail = ui.style != Style::Icon;
            let show_bar = shows_detail && associated && bar_state != State::NoInternet;
            let chip = Chip {
                symbol,
                variable: if connecting {
                    f64::from(frame) / 3.0
                } else {
                    f64::from(segments) / 4.0
                },
                bar: show_bar.then_some(BarFill {
                    segments: if connecting { 0 } else { segments },
                    dim: snapshot.signal.is_none(),
                }),
                text: (shows_detail && !text.is_empty()).then_some(text),
                tint,
                stacked: ui.style == Style::Stacked,
            };
            if let Some(button) = ui.status.button(self.mtm()) {
                button.setImage(Some(&bar::chip_image(chip)));
                button.setImagePosition(NSCellImagePosition::ImageOnly);
            }
            ui.last_chip = chip_key;
        }
        ui.scan.tick();
        if let Some(names) = wifi::take_known_networks() {
            ui.saved_names = names;
        }
        let mut lines = summary(&snapshot, now);
        if connecting {
            lines[0] = "Wi-Fi — Connecting…".into();
        }
        let label = format!("{}; {}; {}", lines[0], lines[1], lines[3]);
        if label != ui.tooltip
            && let Some(button) = ui.status.button(self.mtm())
        {
            let text = NSString::from_str(&label);
            button.setToolTip(Some(&text));
            button.setAccessibilityLabel(Some(&text));
            ui.tooltip = label;
        }
        // Everything below only feeds the open menu; menuWillOpen refreshes
        // before it is drawn. Skipping it keeps scutil and view redraws idle.
        if !self.ivars().menu_open.get() {
            return connecting;
        }
        let connected = snapshot
            .reading
            .as_ref()
            .and_then(|r| r.value.ssid.as_deref());
        // A join in progress keeps its row, where it was clicked, until its
        // outcome has been shown, even once it is the connected network.
        let joining = crate::join::progress();
        let joining_name = joining.as_ref().map(|j| j.name.as_str());
        let mut top: Vec<String> = ui
            .scan
            .visible_names()
            .into_iter()
            .filter(|name| Some(name.as_str()) != connected || Some(name.as_str()) == joining_name)
            .take(MAX_NETWORKS)
            .collect();
        if let Some(name) = joining_name
            && !top.iter().any(|n| n == name)
        {
            top.truncate(MAX_NETWORKS - 1);
            top.push(name.to_owned());
        }
        // The strongest six are always the ones shown, but a new scan batch
        // must not re-sort them: a click joins, so a row that moves under the
        // pointer joins the wrong network. Survivors keep their place and
        // newcomers go to the end.
        ui.order.retain(|name| top.contains(name));
        for name in top {
            if !ui.order.contains(&name) {
                ui.order.push(name);
            }
        }
        let names = ui.order.clone();
        // While a scan is running the section spinner already says so; the
        // placeholder only speaks for a finished, empty scan.
        for (index, (item, row)) in ui.known.iter().enumerate() {
            item.setHidden(index >= names.len());
            let key = names.get(index).map(|name| {
                (
                    name.clone(),
                    ui.scan.reading(name),
                    ui.scan.security(name),
                    ui.saved_names.contains(name),
                )
            });
            let progress = joining
                .as_ref()
                .filter(|j| names.get(index) == Some(&j.name))
                .map(|j| (j.phase, j.detail.clone()));
            if row.set_progress(progress)
                && let Some(menu) = ui.status.menu(self.mtm())
            {
                menu.itemChanged(item);
            }
            if key == ui.rows[index] {
                continue;
            }
            if let Some((name, reading, security, saved)) = &key {
                row.set_name(name);
                row.set_reading(*reading);
                row.set_security(*security);
                row.set_saved(*saved);
            }
            ui.rows[index] = key;
        }
        let auth = unsafe { ui.location.authorizationStatus() };
        let authorized = matches!(
            auth,
            CLAuthorizationStatus::AuthorizedAlways | CLAuthorizationStatus::AuthorizedWhenInUse
        );
        // Without Location every scanned network arrives nameless, which is
        // not the same as there being none.
        let empty = if !authorized {
            "Allow Location to see nearby network names"
        } else if ui.scan.failed() {
            "Couldn’t scan for networks — try Refresh"
        } else {
            "No other visible networks found"
        };
        set_text(&ui.known_empty_label, empty);
        set_title(&ui.known_empty, empty);
        ui.known_empty.setHidden(!names.is_empty() || ui.scan.busy());
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
        if ui.card.update(content, now)
            && let Some(menu) = ui.status.menu(self.mtm())
        {
            menu.itemChanged(&ui.card_item);
        }
        ui.location_item.setHidden(authorized);
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
        // Only on a change of state: setting a tooltip on every tick restarts
        // its hover delay, so it would never get to appear.
        let scanning = ui.scan.busy();
        if ui.scan_spinner.isHidden() == scanning {
            ui.refresh_item.setEnabled(!scanning);
            ui.refresh_item.setHidden(scanning);
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
        let selected = match ui.style {
            Style::Smart => 0,
            Style::Icon => 1,
            Style::Stacked => 2,
        };
        for (index, item) in ui.styles.iter().enumerate() {
            item.setState(if index == selected {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            });
        }
        let current = match ui.style {
            Style::Smart => "Smart Bar",
            Style::Icon => "Icon",
            Style::Stacked => "Stacked",
        };
        set_text(&ui.style_detail, current);
        connecting
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
    widget.set_timer(TICK_CLOSED);
    app.run();
}
