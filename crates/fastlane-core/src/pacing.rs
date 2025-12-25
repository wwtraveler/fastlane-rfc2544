//! Pacing — Rate control and packet timing for high-speed interfaces

use std::time::Duration;

/// Rate limiting type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateType {
    /// Constant Bit Rate
    Cbr,
    /// Send at line rate with invalid packets
    Poison,
    /// Hardware-assisted rate limiting
    Hw,
}

impl std::fmt::Display for RateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateType::Cbr => write!(f, "cbr"),
            RateType::Poison => write!(f, "poison"),
            RateType::Hw => write!(f, "hw"),
        }
    }
}

/// Pacing configuration
pub struct Pacer {
    pub rate_mpps: f64,
    pub frame_size: u32,
    pub rate_type: RateType,
    pub interval_ns: u64,
    pub batch_size: u32,
    pub warmup_period: u64,
}

impl Pacer {
    /// Create a new pacer with the given parameters
    pub fn new(rate_mpps: f64, frame_size: u32, batch_size: u32) -> Self {
        let interval_ns = calc_interval_ns(rate_mpps, frame_size);
        Self {
            rate_mpps,
            frame_size,
            rate_type: RateType::Cbr,
            interval_ns,
            batch_size,
            warmup_period: 2_000_000_000, // 2 seconds in nanoseconds
        }
    }

    /// Calculate the next batch interval
    pub fn next_interval(&self) -> Duration {
        Duration::from_nanos(self.interval_ns / (self.batch_size as u64))
    }

    /// Calculate the pacing interval in nanoseconds
    pub fn calc_interval_ns(&self) -> u64 {
        calc_interval_ns(self.rate_mpps, self.frame_size)
    }
}

/// Calculate the pacing interval in nanoseconds
/// Formula: interval_ns = 1e9 / (rate_mpps * 1e6) = 1e3 / rate_mpps
pub fn calc_interval_ns(rate_mpps: f64, frame_size: u32) -> u64 {
    let rate_mbps = rate_mpps * (frame_size as f64) / 8.0;
    (1e12 / rate_mbps) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_interval() {
        let interval = calc_interval_ns(1000.0, 128);
        assert!(interval > 0);
        assert!(interval < 1_000_000);
    }

    #[test]
    fn test_pacer_new() {
        let pacer = Pacer::new(1000.0, 128, 64);
        assert_eq!(pacer.frame_size, 128);
        assert_eq!(pacer.batch_size, 64);
        assert_eq!(pacer.rate_type, RateType::Cbr);
    }

    #[test]
    fn test_pacer_interval() {
        let pacer = Pacer::new(1000.0, 128, 64);
        assert_eq!(pacer.calc_interval_ns(), pacer.interval_ns);
    }

    #[test]
    fn test_rate_type_display() {
        assert_eq!(format!("{}", RateType::Cbr), "cbr");
        assert_eq!(format!("{}", RateType::Poison), "poison");
        assert_eq!(format!("{}", RateType::Hw), "hw");
    }
}
