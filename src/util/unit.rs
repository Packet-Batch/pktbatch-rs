/// Formats a byte count or byte rate into a human-readable string.
///
/// # Arguments
/// * `bytes` - The value to format (bytes or bytes/sec depending on `sec`).
/// * `sec` - If `true`, formats as a rate (B/s, KB/s, …); otherwise as a total (B, KB, …).
///
/// # Returns
/// A formatted string with appropriate unit prefix.
pub fn format_byt(bytes: f64, sec: bool) -> String {
    let bits = bytes * 8.0;

    let (value, prefix) = if bits >= 1_000_000_000.0 {
        (bits / 1_000_000_000.0, "G")
    } else if bits >= 1_000_000.0 {
        (bits / 1_000_000.0, "M")
    } else if bits >= 1_000.0 {
        (bits / 1_000.0, "K")
    } else {
        (bits, "")
    };

    if sec {
        format!("{:.2} {}bps", value, prefix)
    } else {
        format!("{:.2} {}b", value, prefix)
    }
}
/// Formats a packet count or packet rate into a human-readable string.
///
/// # Arguments
/// * `packets` - The value to format (packets or packets/sec depending on `sec`).
/// * `sec` - If `true`, formats as a rate (pps, Kpps, …); otherwise as a total (pkts, Kpkts, …).
///
/// # Returns
/// A formatted string with appropriate unit prefix.
pub fn format_pkt(packets: f64, sec: bool) -> String {
    let (value, prefix) = if packets >= 1_000_000_000.0 {
        (packets / 1_000_000_000.0, "G")
    } else if packets >= 1_000_000.0 {
        (packets / 1_000_000.0, "M")
    } else if packets >= 1_000.0 {
        (packets / 1_000.0, "K")
    } else {
        (packets, "")
    };

    // Use whole numbers for sub-1000 counts/rates; decimals otherwise
    let precision = if prefix.is_empty() { 0 } else { 2 };

    if sec {
        format!("{:.prec$} {prefix}pps", value, prec = precision)
    } else {
        format!("{:.prec$} {prefix}pkts", value, prec = precision)
    }
}
