// Primes up to 71
pub const PRIMES: &[usize] = &[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71];

// Function to find the closest 1/p score
pub fn get_closest_prime_reciprocal(relative_frequency: f64) -> f64 {
    if relative_frequency == 0.0 {
        return 0.0;
    }
    if relative_frequency >= 1.0 { // "1 occurs all the time"
        return 1.0;
    }

    let mut closest_score = 0.0;
    let mut min_diff = f64::MAX;

    for &p in PRIMES {
        let score = 1.0 / (p as f64);
        let diff = (relative_frequency - score).abs();
        if diff < min_diff {
            min_diff = diff;
            closest_score = score;
        }
    }
    closest_score
}
