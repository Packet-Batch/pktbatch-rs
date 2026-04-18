use rand::Rng;

/// Generates a random number between the specified minimum and maximum values (inclusive).
///
/// # Arguments
/// * `min` - The minimum value for the random number (inclusive).
/// * `max` - The maximum value for the random number (inclusive).
///
/// # Returns
/// A random number of type `u16` between `min` and `max`.
pub fn rand_num(min: u16, max: u16) -> u16 {
    rand::thread_rng().gen_range(min..=max)
}
