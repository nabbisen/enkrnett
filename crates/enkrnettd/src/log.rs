//! Log lines (ENK-API-028 to 030): one line per event to standard error, which `procd` hands to
//! `logd`. No station or client address, no name the operator entered, no secret.

use std::fmt::Display;

/// Writes `enkrnettd <event> key=value ...`.
pub fn event(name: &str, fields: &[(&str, &dyn Display)]) {
    let mut line = format!("enkrnettd {name}");
    for (k, v) in fields {
        line.push(' ');
        line.push_str(k);
        line.push('=');
        line.push_str(&v.to_string());
    }
    eprintln!("{line}");
}
