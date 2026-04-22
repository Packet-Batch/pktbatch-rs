/// Formats the current bytes per second value into a human-readable string with appropriate units.
///
/// # Arguments
/// * `bps` - The bytes per second value to format.
///
/// # Returns
/// A formatted string representing the bytes per second value with appropriate units (bps, Kbps, Mbps, Gbps).
pub fn format_bps(bps: f64) -> String {
    if bps >= 1_000_000_000.0 {
        format!("{:.2} Gbps", bps / 1_000_000_000.0)
    } else if bps >= 1_000_000.0 {
        format!("{:.2} Mbps", bps / 1_000_000.0)
    } else if bps >= 1_000.0 {
        format!("{:.2} Kbps", bps / 1_000.0)
    } else {
        format!("{:.0} bps", bps)
    }
}

/// Formats the current packets per second value into a human-readable string with appropriate units.
///
/// # Arguments
/// * `pps` - The packets per second value to format.
///
/// # Returns
/// A formatted string representing the packets per second value with appropriate units (pps, Kpps, Mpps).
pub fn format_pps(pps: f64) -> String {
    if pps >= 1_000_000.0 {
        format!("{:.2} Mpps", pps / 1_000_000.0)
    } else if pps >= 1_000.0 {
        format!("{:.2} Kpps", pps / 1_000.0)
    } else {
        format!("{:.0} pps", pps)
    }
}
