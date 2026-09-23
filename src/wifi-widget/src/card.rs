//! Native v3 connected card. Real controls provide accessibility and copying;
//! only the graphite surfaces, dividers and signal meter are custom drawing.
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, rc::Retained, sel,
};
use objc2_app_kit::*;
use objc2_foundation::{NSObjectProtocol, NSPoint, NSRect, NSSize, NSString};
use std::{
    cell::{Cell, RefCell},
    time::{Duration, Instant},
};
use wifi_widget::{
    model::{LinkHealth, Snapshot, State},
    probe::ProbeStatus,
    signal::Signal,
};
const WIDTH: f64 = 320.0;
#[derive(Clone, Debug, PartialEq)]
pub struct Content {
    pub headline: String,
    pub name: String,
    pub band: String,
    pub spec: String,
    pub verdict: String,
    pub level: Option<f64>,
    pub snr: bool,
    pub metrics: [String; 3],
    pub internet: String,
    pub ip: Option<String>,
    pub gateway: Option<String>,
    pub mac: Option<String>,
    pub node: Option<String>,
    pub state: State,
    pub associated: bool,
}
impl Content {
    pub fn from_snapshot(s: &Snapshot, now: Instant) -> Self {
        let associated = matches!(s.link, LinkHealth::Associated | LinkHealth::Redacted);
        let r = s.reading.as_ref().map(|r| &r.value);
        let headline = match s.headline {
            State::WifiOff => "Off",
            State::Disconnected => "Disconnected",
            State::NoInternet => "No internet",
            State::LoginRequired => "Login required",
            State::Weak => "Weak signal",
            State::MeteredFallback => "Metered",
            State::BandFallback => "Band fallback",
            State::Healthy => "Connected",
            State::Unknown if associated => "Connected",
            _ => "Unavailable",
        }
        .into();
        let (verdict, level, snr) = match s.signal {
            Some(Signal::Snr(v)) => (
                format!("{v} dB · {:?}", s.signal.unwrap().tier()),
                Some((f64::from(v) / 50.0).clamp(0.0, 1.0)),
                true,
            ),
            Some(Signal::Rssi(v)) => (
                format!("{} dBm · {:?}", minus(v), s.signal.unwrap().tier()),
                Some(((f64::from(v) + 100.0) / 60.0).clamp(0.0, 1.0)),
                false,
            ),
            None => ("Signal unavailable".into(), None, false),
        };
        let measured = |value: &Option<wifi_widget::model::Sample<i32>>| {
            value
                .as_ref()
                .and_then(|v| v.fresh_value(now))
                .map(|v| minus(*v))
                .unwrap_or_else(|| "—".into())
        };
        let internet = match s
            .probe
            .as_ref()
            .filter(|p| p.age(now) < Duration::from_secs(65))
            .map(|p| &p.value)
        {
            Some(ProbeStatus::Reachable { latency }) => {
                format!("ONLINE · {} ms", latency.as_millis())
            }
            Some(ProbeStatus::Captive { .. }) => "Login required".into(),
            Some(p) if p.is_failure() && s.failures >= 2 => "Endpoint unreachable".into(),
            Some(p) if p.is_failure() => "Checking again…".into(),
            Some(ProbeStatus::UnexpectedResponse) => "Unexpected response".into(),
            _ => "Checking…".into(),
        };
        Self {
            ip: None,
            gateway: None,
            headline,
            associated,
            state: s.headline,
            name: r
                .and_then(|r| r.ssid.as_ref())
                .map(|v| clean(v))
                .unwrap_or_else(|| "Wi-Fi".into()),
            band: r
                .and_then(|r| r.band)
                .map(|b| b.label().replace("GHz", " GHz"))
                .unwrap_or_default(),
            spec: r
                .map(|r| {
                    [
                        r.phy.map(str::to_owned),
                        r.channel.map(|n| format!("ch {n}")),
                        r.width_mhz.map(|n| format!("{n} MHz")),
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" · ")
                })
                .unwrap_or_default(),
            verdict,
            level,
            snr,
            metrics: [
                measured(&s.rssi),
                measured(&s.noise),
                r.and_then(|r| r.rate_mbps)
                    .map(|v| format!("{v:.0}"))
                    .unwrap_or_else(|| "—".into()),
            ],
            internet,
            mac: r.and_then(|r| r.mac.clone()),
            node: r.and_then(|r| r.bssid.clone()),
        }
    }
    fn height(&self) -> f64 {
        if !self.associated {
            76.0
        } else if self.node.is_some() {
            322.0
        } else {
            300.0
        }
    }
}
fn minus(v: i32) -> String {
    v.to_string().replace('-', "−")
}
fn clean(s: &str) -> String {
    s.chars().filter(|c| !c.is_control()).collect()
}
struct Parts {
    labels: Vec<Retained<NSTextField>>,
    glyph: Retained<NSImageView>,
    buttons: Vec<Retained<NSButton>>,
    copy_icons: Vec<Retained<NSImageView>>,
}
pub struct CardIvars {
    content: RefCell<Content>,
    parts: RefCell<Option<Parts>>,
    copied: Cell<Option<(usize, Instant)>>,
    hover: Cell<Option<usize>>,
}
define_class!(
    #[unsafe(super(NSView))]
    #[thread_kind = MainThreadOnly]
    #[name = "WiFiConnectedCard"]
    #[ivars = CardIvars]
    pub struct Card;
    unsafe impl NSObjectProtocol for Card {}
    impl Card {
        #[unsafe(method(isFlipped))]
        fn flipped(&self) -> bool { true }
        #[unsafe(method(drawRect:))]
        fn draw(&self, _rect: NSRect) {
            let c = self.ivars().content.borrow(); let ink = accent(c.state);
            // No background fill: the menu's own material must show through,
            // or this item reads as a different shade from the rows below it.
            fill(rect(11.0,6.0,14.0,14.0), &ink.colorWithAlphaComponent(0.18), 7.0);
            fill(rect(14.0,9.0,8.0,8.0),&ink,4.0);
            if !c.associated { return; }
            fill(rect(7.0,30.0,306.0,c.height()-34.0),&rgb(0x17181b),10.0);
            fill(rect(16.0,130.0,288.0,c.height()-147.0),&rgb(0x1e1f22),7.0);
            let outline=NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(rect(7.5,30.5,305.0,c.height()-35.0),10.0,10.0);
            rgb(0x46474b).set(); outline.setLineWidth(0.5);outline.stroke();
            if !c.band.is_empty() && let Some(parts)=self.ivars().parts.borrow().as_ref() {
                let frame=parts.labels[2].frame();
                let pill=NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(rect(frame.origin.x,46.0,frame.size.width,14.0),7.0,7.0);
                rgb(if c.state==State::BandFallback { 0xffb340 } else { 0x55565b }).set();pill.setLineWidth(0.75);pill.stroke();
            }
            let track = rect(66.0,95.0,220.0,10.0);
            fill(track,&rgb(0x3c3d40),3.0);
            if let Some(level) = c.level {
                if c.snr {
                    let context = NSGraphicsContext::currentContext().unwrap(); context.saveGraphicsState();
                    NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(track,3.0,3.0).addClip();
                    for (start,width,color) in [(0.0,0.3,0xcb4b43),(0.3,0.2,0xb98930),(0.5,0.3,0x617da4),(0.8,0.2,0x359d50)] {
                        fill(rect(66.0+220.0*start,95.0,220.0*width,10.0),&rgb(color),0.0);
                    }
                    context.restoreGraphicsState();
                } else { fill(rect(66.0,95.0,220.0*level,10.0),&ink,3.0); }
                let x = 66.0 + 220.0*level;
                fill(rect(x-2.0,91.0,4.0,18.0),&rgb(0x17181b),2.0);
                fill(rect(x-1.0,91.0,2.0,18.0),&NSColor::whiteColor(),1.0);
            }
            for y in [185.0,213.0] { fill(rect(23.0,y,274.0,0.5),&rgb(0x424347),0.0); }
            if let Some(index) = self.ivars().hover.get() { fill(rect(21.0,217.0+index as f64*22.0,278.0,22.0),&rgb(0x36373b),4.0); }
        }
        #[unsafe(method(copyFact:))]
        fn copy_fact(&self, sender: &NSButton) {
            let index = sender.tag() as usize;
            let value = { let c = self.ivars().content.borrow(); [&c.ip, &c.gateway, &c.mac, &c.node].get(index).and_then(|v| (*v).clone()) };
            if let Some(value) = value {
                let board = NSPasteboard::generalPasteboard(); board.clearContents();
                if unsafe { board.setString_forType(&NSString::from_str(&value),NSPasteboardTypeString) } {
                    self.ivars().copied.set(Some((index,Instant::now()+Duration::from_millis(1100))));
                    self.sync(Instant::now());
                }
            }
        }
        #[unsafe(method(mouseMoved:))]
        fn moved(&self, event: &NSEvent) {
            let p = self.convertPoint_fromView(event.locationInWindow(),None);
            let index = if (21.0..299.0).contains(&p.x) && (217.0..305.0).contains(&p.y) { Some(((p.y-217.0)/22.0) as usize) } else { None };
            if self.ivars().hover.replace(index) != index { self.sync(Instant::now()); self.setNeedsDisplay(true); }
        }
        #[unsafe(method(mouseExited:))]
        fn exited(&self, _event: &NSEvent) { self.ivars().hover.set(None); self.sync(Instant::now()); self.setNeedsDisplay(true); }
    }
);
impl Card {
    pub fn new(content: Content, mtm: MainThreadMarker) -> Retained<Self> {
        let height = content.height();
        let this = Self::alloc(mtm).set_ivars(CardIvars {
            content: RefCell::new(content),
            parts: RefCell::new(None),
            copied: Cell::new(None),
            hover: Cell::new(None),
        });
        let this: Retained<Self> =
            unsafe { msg_send![super(this),initWithFrame:rect(0.0,0.0,WIDTH,height)] };
        this.setAccessibilityElement(false);
        let mut labels = Vec::new();
        for (x, y, w, h, size) in [
            (31., 4., 274., 20., 13.),
            (66., 39., 193., 18., 13.),
            (264., 42., 58., 15., 9.),
            (66., 58., 237., 15., 10.),
            (66., 74., 220., 16., 9.),
            (23., 111., 88., 23., 16.),
            (116., 111., 88., 23., 16.),
            (209., 111., 88., 23., 16.),
            (23., 132., 88., 15., 9.),
            (116., 132., 88., 15., 9.),
            (209., 132., 88., 15., 9.),
            (23., 159., 65., 18., 10.),
            (93., 159., 224., 18., 10.),
            (15., 38., 290., 25., 12.),
            (23., 231., 65., 18., 10.),
            (23., 253., 65., 18., 10.),
            (145., 159., 172., 18., 10.),
            (23., 187., 65., 18., 10.),
            (23., 209., 65., 18., 10.),
            (54., 108., 24., 13., 8.),
            (120., 108., 24., 13., 8.),
            (164., 108., 24., 13., 8.),
            (230., 108., 24., 13., 8.),
            (274., 108., 24., 13., 8.),
            (220., 74., 66., 16., 9.),
        ] {
            let label = NSTextField::labelWithString(&NSString::from_str(""), mtm);
            label.setFrame(rect(
                x,
                match labels.len() {
                    0 | 13 => y,
                    5..=10 => y + 28.,
                    11 | 12 | 14..=18 => y + 34.,
                    _ => y + 4.,
                },
                w,
                h,
            ));
            label.setFont(Some(&NSFont::systemFontOfSize(size)));
            if [1, 5, 6, 7].contains(&labels.len()) {
                label.setFont(Some(&NSFont::boldSystemFontOfSize(size)));
            }
            label.setTextColor(Some(&rgb(if [0, 1, 5, 6, 7, 12].contains(&labels.len()) {
                0xf2f2f3
            } else {
                0xa8a9ae
            })));
            label.setLineBreakMode(NSLineBreakMode::ByTruncatingTail);
            if [2, 5, 6, 7, 8, 9, 10, 19, 20, 21, 22, 23].contains(&labels.len()) {
                label.setAlignment(NSTextAlignment::Center);
            }
            if labels.len() == 4 || labels.len() == 24 {
                label.setAlignment(NSTextAlignment::Right);
            }
            this.addSubview(&label);
            labels.push(label);
        }
        let glyph = NSImageView::new(mtm);
        glyph.setFrame(rect(20.0, 63.0, 34.0, 34.0));
        glyph.setImageScaling(NSImageScaling::ScaleProportionallyUpOrDown);
        this.addSubview(&glyph);
        let mut buttons = Vec::new();
        let mut copy_icons = Vec::new();
        for index in 0..4 {
            let button = unsafe {
                NSButton::buttonWithTitle_target_action(
                    &NSString::from_str(""),
                    Some(this.as_ref()),
                    Some(sel!(copyFact:)),
                    mtm,
                )
            };
            // Copy controls sit on graphite in either system appearance.
            button.setAppearance(
                NSAppearance::appearanceNamed(unsafe { NSAppearanceNameDarkAqua }).as_deref(),
            );
            button.setTag(index);
            button.setBordered(false);
            button.setAlignment(NSTextAlignment::Left);
            button.setFont(Some(&NSFont::monospacedSystemFontOfSize_weight(
                10.0,
                unsafe { NSFontWeightRegular },
            )));
            button.setFrame(rect(94.0, 217.0 + index as f64 * 22.0, 206.0, 22.0));
            this.addSubview(&button);
            buttons.push(button);
            let icon = NSImageView::new(mtm);
            icon.setFrame(rect(282.0, 222.0 + index as f64 * 22.0, 12.0, 12.0));
            icon.setImage(
                NSImage::imageWithSystemSymbolName_accessibilityDescription(
                    &NSString::from_str("doc.on.doc"),
                    None,
                )
                .as_deref(),
            );
            icon.setContentTintColor(Some(&rgb(0xa8a9ae)));
            icon.setAccessibilityElement(false);
            icon.setHidden(true);
            this.addSubview(&icon);
            copy_icons.push(icon);
        }
        *this.ivars().parts.borrow_mut() = Some(Parts {
            labels,
            glyph,
            buttons,
            copy_icons,
        });
        let tracking = unsafe {
            NSTrackingArea::initWithRect_options_owner_userInfo(
                objc2::AnyThread::alloc(),
                NSRect::ZERO,
                NSTrackingAreaOptions::MouseMoved
                    | NSTrackingAreaOptions::MouseEnteredAndExited
                    | NSTrackingAreaOptions::ActiveAlways
                    | NSTrackingAreaOptions::InVisibleRect,
                Some(this.as_ref()),
                None,
            )
        };
        this.addTrackingArea(&tracking);
        this.sync(Instant::now());
        this
    }
    pub fn update(&self, content: Content, now: Instant) -> bool {
        let resized = self.bounds().size.height != content.height();
        let changed = *self.ivars().content.borrow() != content;
        if changed {
            // Never leave "Copied" attached to a fact from a different network.
            {
                let previous = self.ivars().content.borrow();
                if previous.mac != content.mac
                    || previous.node != content.node
                    || previous.ip != content.ip
                    || previous.gateway != content.gateway
                {
                    self.ivars().copied.set(None);
                }
            }
            let height = content.height();
            *self.ivars().content.borrow_mut() = content;
            self.setFrameSize(NSSize {
                width: WIDTH,
                height,
            });
            self.sync(now);
            self.setNeedsDisplay(true);
        } else if self
            .ivars()
            .copied
            .get()
            .is_some_and(|(_, until)| now >= until)
        {
            self.ivars().copied.set(None);
            self.sync(now);
        }
        resized
    }
    fn sync(&self, now: Instant) {
        let c = self.ivars().content.borrow();
        let parts = self.ivars().parts.borrow();
        let Some(p) = parts.as_ref() else {
            return;
        };
        let (internet_status, internet_detail) =
            c.internet.split_once(" · ").unwrap_or((&c.internet, ""));
        let quality = c.verdict.rsplit_once(" · ").map(|(_, q)| q).unwrap_or("");
        let texts: [&str; 25] = [
            &c.headline,
            &c.name,
            &c.band,
            &c.spec,
            &c.verdict,
            &c.metrics[0],
            &c.metrics[1],
            &c.metrics[2],
            "signal · dBm",
            "noise · dBm",
            "Mbps",
            "Internet",
            internet_status,
            match c.state {
                State::WifiOff => "Wi-Fi is turned off.",
                State::Disconnected => "Choose a network in Wi-Fi Settings.",
                _ => "Waiting for the Wi-Fi interface…",
            },
            "MAC",
            "Node",
            internet_detail,
            "IP",
            "Gateway",
            "0",
            "15",
            "25",
            "40",
            "50",
            quality,
        ];
        for (i, (label, text)) in p.labels.iter().zip(texts).enumerate() {
            label.setStringValue(&NSString::from_str(text));
            label.setHidden(if i == 0 {
                false
            } else if i == 13 {
                c.associated
            } else {
                !c.associated
            });
        }
        for label in &p.labels[19..24] {
            label.setHidden(!c.associated || !c.snr);
        }
        p.labels[24].setHidden(!c.associated || quality.is_empty());
        let quality_width = p.labels[24].intrinsicContentSize().width + 4.0;
        p.labels[24].setFrame(rect(286.0 - quality_width, 78.0, quality_width, 16.0));
        if let Some((number, _)) = c.verdict.rsplit_once(" · ") {
            p.labels[4].setStringValue(&NSString::from_str(&format!("{number} ·")));
            p.labels[4].setFrame(rect(66.0, 78.0, 220.0 - quality_width, 16.0));
        } else {
            p.labels[4].setFrame(rect(66.0, 78.0, 220.0, 16.0));
        }
        p.labels[24].setTextColor(Some(&rgb(match quality {
            "Poor" => 0xcb4b43,
            "Fair" => 0xb98930,
            "Good" => 0x617da4,
            "Excellent" => 0x359d50,
            _ => 0xa8a9ae,
        })));
        let name_width = (p.labels[1].intrinsicContentSize().width + 10.0).min(170.0);
        p.labels[1].setFrame(rect(66.0, 43.0, name_width, 18.0));
        let pill_width = p.labels[2].intrinsicContentSize().width + 8.0;
        p.labels[2].setFrame(rect(66.0 + name_width + 6.0, 47.0, pill_width, 14.0));
        p.labels[14].setHidden(!c.associated || c.mac.is_none());
        p.labels[15].setHidden(!c.associated || c.node.is_none());
        let status_width = p.labels[12].intrinsicContentSize().width + 6.0;
        p.labels[12].setFrame(rect(93.0, 193.0, status_width, 18.0));
        p.labels[16].setFrame(rect(93.0 + status_width, 193.0, 204.0 - status_width, 18.0));
        p.labels[2].setHidden(!c.associated || c.band.is_empty());
        p.labels[12].setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
            10.0,
            unsafe { NSFontWeightMedium },
        )));
        p.labels[12].setTextColor(Some(&rgb(if c.internet.starts_with("ONLINE") {
            0x48d76b
        } else if c.state == State::NoInternet {
            0xff5b52
        } else if c.state == State::LoginRequired {
            0xffb340
        } else {
            0xa8a9ae
        })));
        p.labels[12].setToolTip(Some(&NSString::from_str("Result of the HTTP reachability check against captive.apple.com; timing includes connection and response.")));
        for (index, label) in [(5, "Signal"), (6, "Noise"), (7, "Transmit rate")] {
            let unit = if index == 7 { "Mbps" } else { "dBm" };
            p.labels[index].setAccessibilityLabel(Some(&NSString::from_str(&format!(
                "{label}: {} {unit}",
                c.metrics[index - 5]
            ))));
        }
        p.labels[0].setTextColor(Some(&NSColor::labelColor()));
        p.labels[13].setTextColor(Some(&NSColor::secondaryLabelColor()));
        p.glyph.setHidden(!c.associated);
        let symbol = match c.state {
            State::NoInternet | State::LoginRequired => "wifi.exclamationmark",
            State::MeteredFallback => "personalhotspot",
            _ => "wifi",
        };
        p.glyph.setImage(
            NSImage::imageWithSystemSymbolName_accessibilityDescription(
                &NSString::from_str(symbol),
                Some(&NSString::from_str(&c.verdict)),
            )
            .as_deref(),
        );
        p.glyph.setContentTintColor(Some(&accent(c.state)));
        for (index, button) in p.buttons.iter().enumerate() {
            let value = [&c.ip, &c.gateway, &c.mac, &c.node][index];
            button.setHidden(!c.associated || (index >= 2 && value.is_none()));
            if value.is_none() {
                button.setTitle(&NSString::from_str("—"));
            }
            button.setEnabled(value.is_some());
            p.copy_icons[index].setHidden(
                !c.associated || value.is_none() || self.ivars().hover.get() != Some(index),
            );
            if let Some(value) = value {
                let key = ["IP", "Gateway", "MAC", "Node"][index];
                let private = index == 2
                    && value
                        .split(':')
                        .next()
                        .and_then(|v| u8::from_str_radix(v, 16).ok())
                        .is_some_and(|v| v & 2 != 0);
                let title = if self
                    .ivars()
                    .copied
                    .get()
                    .is_some_and(|(i, until)| index == i && now < until)
                {
                    String::from("Copied")
                } else {
                    format!("{value}{}", if private { "  PRIVATE" } else { "" })
                };
                button.setTitle(&NSString::from_str(&title));

                button.setToolTip(Some(&NSString::from_str("Click to copy")));
                button.setAccessibilityLabel(Some(&NSString::from_str(&format!(
                    "Copy {key} address {value}"
                ))));
            }
        }
    }
}
fn accent(state: State) -> Retained<NSColor> {
    rgb(match state {
        State::Healthy => 0x48d76b,
        State::NoInternet | State::Weak => 0xff5b52,
        State::LoginRequired | State::MeteredFallback | State::BandFallback => 0xffb340,
        _ => 0xa8a9ae,
    })
}
fn rgb(hex: u32) -> Retained<NSColor> {
    NSColor::colorWithSRGBRed_green_blue_alpha(
        f64::from((hex >> 16) & 255) / 255.,
        f64::from((hex >> 8) & 255) / 255.,
        f64::from(hex & 255) / 255.,
        1.,
    )
}
fn rect(x: f64, y: f64, width: f64, height: f64) -> NSRect {
    NSRect {
        origin: NSPoint { x, y },
        size: NSSize { width, height },
    }
}
fn fill(r: NSRect, color: &NSColor, radius: f64) {
    color.set();
    NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(r, radius, radius).fill();
}

#[cfg(test)]
mod tests {
    use super::*;
    use wifi_widget::{model::Store, wifi::LinkReading};
    #[test]
    fn unavailable_measurements_never_become_displayed_signal() {
        let now = Instant::now();
        let mut store = Store::default();
        store.update_link(
            Some(LinkReading {
                power: true,
                associated: true,
                rssi: Some(-52),
                noise: None,
                ..Default::default()
            }),
            now,
        );
        let current = Content::from_snapshot(&store.snapshot(now), now);
        assert!(!current.snr);
        assert!(current.verdict.contains("dBm"));
        assert_eq!(current.metrics[1], "—");
        let later = now + Duration::from_secs(11);
        let stale = Content::from_snapshot(&store.snapshot(later), later);
        assert!(stale.level.is_none());
        assert_eq!(stale.metrics[0], "—");
    }
    #[test]
    fn redacted_and_off_cards_have_no_stale_identity() {
        let now = Instant::now();
        let mut store = Store::default();
        store.update_link(
            Some(LinkReading {
                power: true,
                associated: true,
                rssi: Some(-52),
                noise: Some(-93),
                ..Default::default()
            }),
            now,
        );
        let c = Content::from_snapshot(&store.snapshot(now), now);
        assert_eq!(c.name, "Wi-Fi");
        assert!(c.node.is_none());
        assert!(c.associated);
        store.update_link(Some(LinkReading::default()), now);
        let c = Content::from_snapshot(&store.snapshot(now), now);
        assert!(!c.associated);
        assert_eq!(c.height(), 76.);
    }
}
