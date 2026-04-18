/// Retrieves the total numbers of CPUs available on the system.
///
/// NOTE: This uses the `num_cpus` crate to retrieve the CPU counts.
///
/// # Returns
/// The total number of CPUs available on the system.
pub fn get_cpu_count() -> usize {
    num_cpus::get()
}

/// Retrieves the processor's timestamp counter (TSC) value. This is a high-resolution timer that counts the number of cycles since reset.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn get_cpu_rdtsc() -> u32 {
    unsafe { std::arch::x86_64::_rdtsc() as u32 }
}

/// Fallback: Uses system time in nanoseconds.
#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
pub fn get_cpu_rdtsc() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos()
}
