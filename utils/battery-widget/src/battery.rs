use std::collections::HashMap;
use std::process::Command;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BatteryState {
    Discharging,
    Charging,
    Idle, // on AC, full or not charging
}

#[derive(Clone)]
pub struct BatteryInfo {
    pub percent: i32,
    pub state: BatteryState,
    pub time_remaining: Option<String>, // "3:12", None when macOS has no estimate
    pub watts: f64,                     // magnitude of battery power flow
    pub health_percent: i64,
    pub cycle_count: i64,
    pub low_power_mode: bool,
    pub on_ac: bool,
}

fn run(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("{program}: {e}"))?;
    if !output.status.success() {
        return Err(format!("{program} exited with {}", output.status));
    }
    String::from_utf8(output.stdout).map_err(|e| format!("{program}: {e}"))
}

pub fn read_battery() -> Result<BatteryInfo, String> {
    let batt = run("pmset", &["-g", "batt"])?;
    let on_ac = batt.contains("'AC Power'");

    // " -InternalBattery-0 (id=...)\t87%; discharging; 3:12 remaining present: true"
    let detail = batt
        .lines()
        .find(|line| line.contains("%;"))
        .ok_or("no battery found in pmset output")?;
    let mut fields = detail.split(';');

    let percent = fields
        .next()
        .and_then(|f| f.split_whitespace().last())
        .and_then(|t| t.strip_suffix('%'))
        .and_then(|t| t.parse::<i32>().ok())
        .ok_or("no percentage in pmset output")?;

    // "not charging" also contains "charging", so match the exact state word.
    let state = match fields.next().map(str::trim) {
        Some("discharging") => BatteryState::Discharging,
        Some("charging") | Some("finishing charge") => BatteryState::Charging,
        _ => BatteryState::Idle,
    };

    let time_remaining = fields.next().and_then(|f| {
        let token = f.split_whitespace().next()?;
        (f.contains("remaining") && token.contains(':')).then(|| token.to_string())
    });

    let ioreg = run("ioreg", &["-rn", "AppleSmartBattery"])?;
    let values = parse_ioreg(&ioreg);
    let get = |key: &str| values.get(key).copied().unwrap_or(0);

    let watts = (get("Amperage") as f64 * get("Voltage") as f64 / 1e6).abs();

    // Apple silicon reports NominalChargeCapacity/DesignCapacity in mAh;
    // fall back to AppleRawMaxCapacity for older machines.
    let mut max_capacity = get("NominalChargeCapacity");
    if max_capacity == 0 {
        max_capacity = get("AppleRawMaxCapacity");
    }
    let design = get("DesignCapacity");
    let health_percent = if design > 0 && max_capacity > 0 {
        (100.0 * max_capacity as f64 / design as f64).round() as i64
    } else {
        0
    };

    let low_power_mode = run("pmset", &["-g"]).is_ok_and(|out| {
        out.lines().any(|line| {
            let mut fields = line.split_whitespace();
            fields.next() == Some("lowpowermode") && fields.next() == Some("1")
        })
    });

    Ok(BatteryInfo {
        percent,
        state,
        time_remaining,
        watts,
        health_percent,
        cycle_count: get("CycleCount"),
        low_power_mode,
        on_ac,
    })
}

/// Parse `"Key" = value` integer lines. ioreg prints negative numbers (e.g.
/// Amperage while discharging) as wrapped unsigned 64-bit decimals.
fn parse_ioreg(text: &str) -> HashMap<String, i64> {
    let mut values = HashMap::new();
    for line in text.lines() {
        let Some((key_part, value_part)) = line.split_once(" = ") else {
            continue;
        };
        let Some(key) = key_part
            .trim()
            .strip_prefix('"')
            .and_then(|k| k.strip_suffix('"'))
        else {
            continue;
        };
        let token = value_part.trim();
        let value = token
            .parse::<i64>()
            .or_else(|_| token.parse::<u64>().map(|v| v as i64));
        if let Ok(value) = value {
            values.insert(key.to_string(), value);
        }
    }
    values
}
