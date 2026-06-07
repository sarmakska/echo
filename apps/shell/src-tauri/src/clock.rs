// Used by the live voice worker (built only under `--features voice`); the unit
// tests exercise it in every build.
#![allow(dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};

/// Convert a count of days since the Unix epoch to a civil (year, month, day),
/// using Howard Hinnant's algorithm. Valid across the full proleptic Gregorian
/// range. Kept here (not in echo-memory) so the memory store stays clock-free.
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn now_secs() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// Today's episode key, "YYYY/MM/DD" in UTC.
pub fn today_utc() -> String {
    let (y, m, d) = civil_from_days(now_secs().div_euclid(86_400));
    format!("{y:04}/{m:02}/{d:02}")
}

/// Current timestamp as an ISO-8601 UTC string, "YYYY-MM-DDTHH:MM:SSZ".
pub fn now_iso() -> String {
    let secs = now_secs();
    let (y, m, d) = civil_from_days(secs.div_euclid(86_400));
    let sod = secs.rem_euclid(86_400);
    let (hh, mm, ss) = (sod / 3600, (sod % 3600) / 60, sod % 60);
    format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_day_zero_is_1970_01_01() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
    }

    #[test]
    fn day_10957_is_2000_01_01() {
        // 2000-01-01 is 10957 days after 1970-01-01.
        assert_eq!(civil_from_days(10_957), (2000, 1, 1));
    }

    #[test]
    fn today_has_episode_key_shape() {
        let t = today_utc();
        assert_eq!(t.len(), 10);
        assert_eq!(t.as_bytes()[4], b'/');
        assert_eq!(t.as_bytes()[7], b'/');
    }
}
