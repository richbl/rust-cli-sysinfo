use super::prelude::*;
use crate::constants::{BATTERY_CRIT_PCT, BATTERY_WARN_PCT, INDENT, LABEL_WIDTH};

use starship_battery::units::ratio::percent;
use starship_battery::units::time::second;
use starship_battery::{Manager, State};

/// `BatteryReading` contains the plain, display-ready metrics collected for a single battery:
/// state of charge, current charge/discharge state, and the time remaining until full or empty
///
pub struct BatteryReading {
    pub soc_pct: f64,
    pub state: State,
    pub time_to_full_secs: Option<u64>,
    pub time_to_empty_secs: Option<u64>,
}

/// `BatteryInfo` contains a reading for every battery discovered in the system
/// An empty `Vec` is a valid, non-error outcome — it means the host reported zero batteries
#[derive(Default)]
pub struct BatteryInfo {
    pub batteries: Vec<BatteryReading>,
}

/// `BatteryService` is a struct for collecting and rendering battery status
pub struct BatteryService;

/// `BatteryService` implements the `Service` trait
impl Service for BatteryService {
    type Data = BatteryInfo;

    /// `collect()` enumerates every battery reported by the OS and reads its state of charge,
    /// charge/discharge state, and time-to-full/time-to-empty estimates
    ///
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn collect(&self) -> Result<Self::Data, AppError> {
        let manager = Manager::new()
            .map_err(|e| AppError::DataUnavailable(format!("battery manager unavailable: {e}")))?;

        let discovered = manager.batteries().map_err(|e| {
            AppError::DataUnavailable(format!("unable to enumerate batteries: {e}"))
        })?;

        let mut batteries = Vec::new();
        for battery in discovered {
            let battery = battery
                .map_err(|e| AppError::DataUnavailable(format!("unable to read battery: {e}")))?;

            batteries.push(BatteryReading {
                soc_pct: f64::from(battery.state_of_charge().get::<percent>()),
                state: battery.state(),
                time_to_full_secs: battery
                    .time_to_full()
                    .map(|t| t.get::<second>().max(0.0).round() as u64),
                time_to_empty_secs: battery
                    .time_to_empty()
                    .map(|t| t.get::<second>().max(0.0).round() as u64),
            });
        }

        Ok(BatteryInfo { batteries })
    }

    /// `render()` renders one line per battery, or `"No battery identified"` if none were found.
    /// The row's color reflects the single worst battery present — see [`aggregate_threshold`]
    ///
    fn render(&self, data: &Self::Data) -> Result<RenderedRow, AppError> {
        if data.batteries.is_empty() {
            return Ok(RenderedRow {
                value: "No battery identified".to_string(),
                threshold: Threshold::None,
            });
        }

        let separator = format!("\n{:width$}", "", width = INDENT.len() + LABEL_WIDTH + 1);
        let value = data
            .batteries
            .iter()
            .map(format_reading)
            .collect::<Vec<_>>()
            .join(&separator);

        Ok(RenderedRow {
            value,
            threshold: aggregate_threshold(&data.batteries),
        })
    }
}

/// `format_reading()` formats a single [`BatteryReading`] into its display string, branching on
/// [`State`]
///
fn format_reading(reading: &BatteryReading) -> String {
    let soc = reading.soc_pct;

    match reading.state {
        State::Charging => match reading.time_to_full_secs {
            Some(secs) => format!("{soc:.1}% (charging: {} to full)", format_hm(secs)),
            None => format!("{soc:.1}% (charging...)"),
        },
        State::Discharging => match reading.time_to_empty_secs {
            Some(secs) => format!("{soc:.1}% (discharging: {} remaining)", format_hm(secs)),
            None => format!("{soc:.1}% (discharging...)"),
        },
        State::Full => format!("{soc:.1}% (on AC power)"),
        State::Empty => format!("{soc:.1}% (empty)"),
        State::Unknown => format_unknown(soc),
    }
}

/// `format_unknown()` formats the `State::Unknown` case as `(on AC power, not charging)`
///
fn format_unknown(soc: f64) -> String {
    format!("{soc:.1}% (on AC power, not charging)")
}

/// `format_hm()` formats a duration in seconds as `Hh MMm` (the hour component is omitted when
/// zero), truncating down to the nearest whole minute
///
fn format_hm(seconds: u64) -> String {
    let mins_total = seconds / 60;
    let hours = mins_total / 60;
    let mins = mins_total % 60;

    if hours > 0 {
        format!("{hours:02}h:{mins:02}m")
    } else {
        format!("{mins:02}m")
    }
}

/// `effective_soc_for_threshold()` returns the real state of charge for a [`BatteryReading`], for
/// [`Threshold::CheckInverse`] purposes
///
fn effective_soc_for_threshold(reading: &BatteryReading) -> f64 {
    reading.soc_pct
}

/// `aggregate_threshold()` reduces every battery's reading down to the single [`Threshold`] used
/// to color the whole row
///
fn aggregate_threshold(batteries: &[BatteryReading]) -> Threshold {
    let worst_effective_soc = batteries
        .iter()
        .map(effective_soc_for_threshold)
        .fold(f64::INFINITY, f64::min);

    Threshold::CheckInverse {
        value: worst_effective_soc,
        warn: BATTERY_WARN_PCT,
        crit: BATTERY_CRIT_PCT,
    }
}

/// `descriptor()` is this service's registration point, discovered automatically by
/// `build.rs`
///
pub fn descriptor(_ctx: &ServiceContext) -> (ServiceMeta, Box<dyn ErasedService>) {
    (
        ServiceMeta {
            token: "BAT",
            label: "Battery",
            description: "Battery status",
            sort_order: 130,
        },
        Box::new(BatteryService),
    )
}
#[cfg(test)]
mod tests {
    use super::*;

    // format_hm() tests

    /// `format_hm_zero_seconds_shows_zero_minutes()` asserts that zero seconds formats as `0m`
    ///
    #[test]
    fn format_hm_zero_seconds_shows_zero_minutes() {
        assert_eq!(format_hm(0), "00m");
    }

    /// `format_hm_under_an_hour_omits_hours()` asserts that durations under an hour omit the
    /// hour component
    ///
    #[test]
    fn format_hm_under_an_hour_omits_hours() {
        assert_eq!(format_hm(31 * 60), "31m");
    }

    /// `format_hm_truncates_to_the_nearest_minute()` asserts that partial minutes are truncated,
    /// not rounded
    ///
    #[test]
    fn format_hm_truncates_to_the_nearest_minute() {
        assert_eq!(format_hm(31 * 60 + 45), "31m");
    }

    /// `format_hm_over_an_hour_includes_hours_and_minutes()` asserts that durations over an hour
    /// include both components
    ///
    #[test]
    fn format_hm_over_an_hour_includes_hours_and_minutes() {
        assert_eq!(format_hm(9 * 3600 + 42 * 60), "09h:42m");
    }

    /// `format_hm_exact_hour_shows_zero_minutes()` asserts that an exact hour still shows an
    /// explicit `0m`
    ///
    #[test]
    fn format_hm_exact_hour_shows_zero_minutes() {
        assert_eq!(format_hm(2 * 3600), "02h:00m");
    }

    // format_reading() tests

    /// `make_reading()` helper constructs a `BatteryReading` for tests without touching real
    /// system battery hardware
    ///
    fn make_reading(
        soc_pct: f64,
        state: State,
        time_to_full_secs: Option<u64>,
        time_to_empty_secs: Option<u64>,
    ) -> BatteryReading {
        BatteryReading {
            soc_pct,
            state,
            time_to_full_secs,
            time_to_empty_secs,
        }
    }

    /// `charging_with_estimate_shows_soc_and_time_to_full()` asserts the charging + known
    /// time-to-full format
    ///
    #[test]
    fn charging_with_estimate_shows_soc_and_time_to_full() {
        let r = make_reading(76.0, State::Charging, Some(31 * 60), None);
        assert_eq!(format_reading(&r), "76.0% (charging: 31m to full)");
    }

    /// `charging_without_estimate_omits_time()` asserts that a missing time-to-full estimate
    /// still renders cleanly
    ///
    #[test]
    fn charging_without_estimate_omits_time() {
        let r = make_reading(12.0, State::Charging, None, None);
        assert_eq!(format_reading(&r), "12.0% (charging...)");
    }

    /// `discharging_with_estimate_shows_soc_and_time_remaining()` asserts the discharging +
    /// known time-to-empty format
    ///
    #[test]
    fn discharging_with_estimate_shows_soc_and_time_remaining() {
        let r = make_reading(76.0, State::Discharging, None, Some(9 * 3600 + 42 * 60));
        assert_eq!(format_reading(&r), "76.0% (discharging: 09h:42m remaining)");
    }

    /// `discharging_without_estimate_omits_time()` asserts that a missing time-to-empty estimate
    /// still renders cleanly
    ///
    #[test]
    fn discharging_without_estimate_omits_time() {
        let r = make_reading(3.0, State::Discharging, None, None);
        assert_eq!(format_reading(&r), "3.0% (discharging...)");
    }

    /// `full_shows_soc_and_on_ac_power()` asserts the fully-charged-and-plugged-in format
    ///
    #[test]
    fn full_shows_soc_and_on_ac_power() {
        let r = make_reading(100.0, State::Full, None, None);
        assert_eq!(format_reading(&r), "100.0% (on AC power)");
    }

    /// `empty_shows_soc_and_empty_marker()` asserts the fully-drained format
    ///
    #[test]
    fn empty_shows_soc_and_empty_marker() {
        let r = make_reading(0.0, State::Empty, None, None);
        assert_eq!(format_reading(&r), "0.0% (empty)");
    }

    /// `unknown_state_shows_soc_and_infers_ac_power()` asserts that `Unknown` is reported as
    /// "on AC power, not charging" — see [`format_unknown`] for the reasoning
    ///
    #[test]
    fn unknown_state_shows_soc_and_infers_ac_power() {
        let r = make_reading(50.0, State::Unknown, None, None);
        assert_eq!(format_reading(&r), "50.0% (on AC power, not charging)");
    }

    // effective_soc_for_threshold() tests

    /// `discharging_uses_real_soc_for_threshold()` asserts that a discharging battery's real SOC
    /// drives its threshold color
    ///
    #[test]
    fn discharging_uses_real_soc_for_threshold() {
        let r = make_reading(8.0, State::Discharging, None, None);
        assert!((effective_soc_for_threshold(&r) - 8.0).abs() < f64::EPSILON);
    }

    /// `empty_uses_real_soc_for_threshold()` asserts that an empty battery's real SOC drives its
    /// threshold color (typically ~0, which is always critical)
    ///
    #[test]
    fn empty_uses_real_soc_for_threshold() {
        let r = make_reading(0.0, State::Empty, None, None);
        assert!((effective_soc_for_threshold(&r) - 0.0).abs() < f64::EPSILON);
    }

    /// `charging_uses_real_soc_for_threshold()` asserts that a charging battery's real SOC
    /// drives its threshold color, same as discharging
    ///
    #[test]
    fn charging_uses_real_soc_for_threshold() {
        let r = make_reading(4.0, State::Charging, None, None);
        assert!((effective_soc_for_threshold(&r) - 4.0).abs() < f64::EPSILON);
    }

    /// `full_uses_real_soc_for_threshold()` asserts that a full battery's real SOC drives its
    /// threshold color (in practice this is always ~100.0, i.e. green)
    ///
    #[test]
    fn full_uses_real_soc_for_threshold() {
        let r = make_reading(100.0, State::Full, None, None);
        assert!((effective_soc_for_threshold(&r) - 100.0).abs() < f64::EPSILON);
    }

    /// `unknown_uses_real_soc_for_threshold()` asserts that an `Unknown`-state battery's real SOC
    /// drives its threshold color, same as every other state
    ///
    #[test]
    fn unknown_uses_real_soc_for_threshold() {
        let r = make_reading(4.0, State::Unknown, None, None);
        assert!((effective_soc_for_threshold(&r) - 4.0).abs() < f64::EPSILON);
    }

    // aggregate_threshold() tests

    /// `assert_is_checkinverse_matching()` asserts `threshold` is a `Threshold::CheckInverse`
    /// whose fields match the given expected values
    ///
    fn assert_is_checkinverse_matching(threshold: &Threshold, expected_value: f64) {
        assert!(
            matches!(threshold, Threshold::CheckInverse { .. }),
            "expected Threshold::CheckInverse, got a different variant"
        );

        if let Threshold::CheckInverse { value, warn, crit } = threshold {
            assert!((value - expected_value).abs() < f64::EPSILON);
            assert!((warn - BATTERY_WARN_PCT).abs() < f64::EPSILON);
            assert!((crit - BATTERY_CRIT_PCT).abs() < f64::EPSILON);
        }
    }

    /// `aggregate_single_discharging_battery_uses_its_soc()` asserts that a single discharging
    /// battery's own SOC becomes the row's threshold value
    ///
    #[test]
    fn aggregate_single_discharging_battery_uses_its_soc() {
        let batteries = vec![make_reading(64.0, State::Discharging, None, Some(1800))];
        assert_is_checkinverse_matching(&aggregate_threshold(&batteries), 64.0);
    }

    /// `aggregate_single_charging_battery_uses_its_soc()` asserts that a single charging
    /// battery's own SOC becomes the row's threshold value
    ///
    #[test]
    fn aggregate_single_charging_battery_uses_its_soc() {
        let batteries = vec![make_reading(4.0, State::Charging, None, None)];
        assert_is_checkinverse_matching(&aggregate_threshold(&batteries), 4.0);
    }

    /// `aggregate_takes_the_worst_of_multiple_batteries()` asserts that when one battery is safe
    /// (on AC) and another is critically low, the aggregate reflects the critical one
    ///
    #[test]
    fn aggregate_takes_the_worst_of_multiple_batteries() {
        let batteries = vec![
            make_reading(100.0, State::Full, None, None),
            make_reading(5.0, State::Discharging, None, Some(300)),
        ];
        assert_is_checkinverse_matching(&aggregate_threshold(&batteries), 5.0);
    }

    // render() tests

    /// `render_no_batteries_reports_none_identified()` asserts the no-battery-present case
    ///
    #[test]
    fn render_no_batteries_reports_none_identified() {
        let data = BatteryInfo::default();
        let row = BatteryService.render(&data).unwrap();
        assert_eq!(row.value, "No battery identified");
        assert!(matches!(row.threshold, Threshold::None));
    }

    /// `render_single_battery_uses_the_formatted_reading()` asserts that a single battery
    /// renders via `format_reading()`
    ///
    #[test]
    fn render_single_battery_uses_the_formatted_reading() {
        let data = BatteryInfo {
            batteries: vec![make_reading(87.5, State::Discharging, None, Some(3600))],
        };
        let row = BatteryService.render(&data).unwrap();
        assert_eq!(row.value, "87.5% (discharging: 01h:00m remaining)");
        assert_is_checkinverse_matching(&row.threshold, 87.5);
    }

    /// `render_multiple_batteries_joins_with_a_separator_line()` asserts that multiple batteries
    /// are each placed on their own line, and that the row's threshold reflects the worst one
    ///
    #[test]
    fn render_multiple_batteries_joins_with_a_separator_line() {
        let data = BatteryInfo {
            batteries: vec![
                make_reading(100.0, State::Full, None, None),
                make_reading(64.0, State::Discharging, None, Some(1800)),
            ],
        };
        let row = BatteryService.render(&data).unwrap();
        assert!(row.value.contains("100.0% (on AC power)"));
        assert!(row.value.contains("64.0% (discharging: 30m remaining)"));
        assert!(
            row.value.contains('\n'),
            "multiple batteries must be separated onto their own lines"
        );
        assert_is_checkinverse_matching(&row.threshold, 64.0);
    }

    // collect() test — battery presence is host-dependent, so this only asserts that collection
    // does not error; an empty Vec (no batteries, e.g. on a desktop/server/CI runner) is valid

    /// `collect_returns_ok_on_supported_os()` asserts that battery collection succeeds (an empty
    /// `Vec` is a valid, non-error outcome on a battery-less host) on every implemented platform
    ///
    #[test]
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    fn collect_returns_ok_on_supported_os() {
        assert!(BatteryService.collect().is_ok());
    }
}
