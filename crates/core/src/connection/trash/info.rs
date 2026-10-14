//! Formato `.trashinfo` de freedesktop.org: de dónde vino un elemento de la
//! Papelera y cuándo se borró.

use std::time::{SystemTime, UNIX_EPOCH};

/// Ruta y fecha de borrado de un `.trashinfo`.
pub(super) fn parse_info(text: &str) -> Option<(String, Option<u64>)> {
    let mut in_group = false;
    let (mut path, mut deleted) = (None, None);
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            in_group = line == "[Trash Info]";
        } else if in_group {
            match line.split_once('=') {
                Some(("Path", value)) => path = decode_path(value),
                Some(("DeletionDate", value)) => deleted = parse_date(value),
                _ => {}
            }
        }
    }
    path.map(|p| (p, deleted))
}

/// Ruta codificada como en una URL, salvo las `/`.
pub(super) fn encode_path(path: &str) -> String {
    let mut out = String::new();
    for &b in path.as_bytes() {
        if b.is_ascii_alphanumeric() || b"-._~/".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn decode_path(text: &str) -> Option<String> {
    let mut bytes = Vec::new();
    let mut rest = text.as_bytes();
    while let Some((&b, tail)) = rest.split_first() {
        if b == b'%' {
            let hex = std::str::from_utf8(tail.get(..2)?).ok()?;
            bytes.push(u8::from_str_radix(hex, 16).ok()?);
            rest = &tail[2..];
        } else {
            bytes.push(b);
            rest = tail;
        }
    }
    String::from_utf8(bytes).ok()
}

pub(super) fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// Fecha en el formato de `DeletionDate` (`2026-10-14T09:30:00`). La
/// especificación la pide en hora local, pero aquí va en UTC: el equipo del
/// Miembro y el Servidor pueden estar en zonas distintas y la app la lee igual.
pub(super) fn format_date(secs: u64) -> String {
    let (y, m, d) = civil_from_days((secs / 86_400) as i64);
    let s = secs % 86_400;
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}", s / 3600, s % 3600 / 60, s % 60)
}

/// Lee una `DeletionDate` como UTC; ignora fracciones de segundo o zona horaria.
fn parse_date(text: &str) -> Option<u64> {
    let num = |range: std::ops::Range<usize>| text.get(range)?.parse::<i64>().ok();
    let (y, m, d) = (num(0..4)?, num(5..7)?, num(8..10)?);
    let (h, min, s) = (num(11..13)?, num(14..16)?, num(17..19)?);
    let separators = [(4, b'-'), (7, b'-'), (10, b'T'), (13, b':'), (16, b':')];
    if !separators.iter().all(|&(i, c)| text.as_bytes()[i] == c) || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    u64::try_from(days_from_civil(y, m, d) * 86_400 + h * 3600 + min * 60 + s).ok()
}

/// Días desde 1970-01-01 de una fecha del calendario gregoriano (algoritmo de Howard Hinnant).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_round_trip_through_the_encoding() {
        for path in ["/home/alice/notas.txt", "/srv/compartida/mis fotos/año 100%.jpg", "/a/b+c&d#e?"] {
            assert_eq!(decode_path(&encode_path(path)).as_deref(), Some(path));
        }
        assert_eq!(encode_path("/home/alice/mis fotos"), "/home/alice/mis%20fotos");
        assert_eq!(decode_path("/x/%C3%B1"), Some("/x/ñ".to_string()));
        assert_eq!(decode_path("/x/%Z1"), None);
        assert_eq!(decode_path("/x/%4"), None);
    }

    #[test]
    fn dates_round_trip() {
        for secs in [0, 951_782_400, 1_791_970_200, 4_102_444_799] {
            assert_eq!(parse_date(&format_date(secs)), Some(secs));
        }
        assert_eq!(format_date(1_791_970_200), "2026-10-14T09:30:00");
        assert_eq!(parse_date("2026-10-14T09:30:00.123+02:00"), Some(1_791_970_200));
        assert_eq!(parse_date("ayer"), None);
        assert_eq!(parse_date("2026-13-14T09:30:00"), None);
    }

    #[test]
    fn reads_a_trashinfo() {
        let text = "[Trash Info]\nPath=/home/alice/mis%20notas.txt\nDeletionDate=2026-10-14T09:30:00\n";
        assert_eq!(parse_info(text), Some(("/home/alice/mis notas.txt".to_string(), Some(1_791_970_200))));
        assert_eq!(parse_info("[Otra]\nPath=/x\n"), None);
    }
}
