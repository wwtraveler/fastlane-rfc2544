//! Latency test engine (RFC 2544 §26.2)
//!
//! Measures round-trip latency at various load levels.
//! Uses a timestamped packet flow with histogram-based analysis.

use std::collections::BTreeMap;
use std::time::{Duration, Instant, SystemTime};

use crate::config::{Config, LatencyConfig, TestType};
use crate::results::{FrameResult, LatencyTestResult, TrialResult};

/// A single latency measurement sample
#[derive(Debug, Clone)]
pub struct LatencySample {
    pub latency_us: u64,
    pub timestamp: Instant,
}

/// Latency histogram for statistical analysis
#[derive(Debug, Clone)]
pub struct LatencyHistogram {
    pub samples: Vec<LatencySample>,
    pub frame_size: u32,
    pub rate_mpps: f64,
    pub duration: Duration,
}

impl LatencyHistogram {
    /// Calculate minimum latency
    pub fn min(&self) -> Option<u64> {
        self.samples.iter().map(|s| s.latency_us).min()
    }

    /// Calculate maximum latency
    pub fn max(&self) -> Option<u64> {
        self.samples.iter().map(|s| s.latency_us).max()
    }

    /// Calculate average latency
    pub fn avg(&self) -> Option<f64> {
        let sum: u64 = self.samples.iter().map(|s| s.latency_us).sum();
        if self.samples.is_empty() {
            None
        } else {
            Some(sum as f64 / self.samples.len() as f64)
        }
    }

    /// Calculate standard deviation
    pub fn stddev(&self) -> Option<f64> {
        let avg = self.avg()?;
        let variance: f64 = self.samples.iter().map(|s| {
            let diff = s.latency_us as f64 - avg;
            diff * diff
        }).sum::<f64>() / self.samples.len() as f64;
        Some(variance.sqrt())
    }

    /// Calculate percentile
    pub fn percentile(&self, pct: f64) -> Option<u64> {
        if self.samples.is_empty() {
            return None;
        }
        let mut sorted = self.samples.clone();
        sorted.sort_by_key(|s| s.latency_us);
        let idx = (pct / 100.0 * sorted.len() as f64) as usize;
        let idx = idx.min(sorted.len() - 1);
        Some(sorted[idx].latency_us)
    }

    /// Calculate quartiles (Q1, Q2, Q3)
    pub fn quartiles(&self) -> (u64, u64, u64) {
        if self.samples.is_empty() {
            return (0, 0, 0);
        }
        let mut sorted = self.samples.clone();
        sorted.sort_by_key(|s| s.latency_us);
        let n = sorted.len();
        (
            sorted[n / 4].latency_us,
            sorted[n / 2].latency_us,
            sorted[3 * n / 4].latency_us,
        )
    }

    /// Convert to CSV row
    pub fn to_csv(&self) -> String {
        format!(
            "{},{},{},{},{}",
            self.latency_us(),
            self.samples.len(),
            self.frame_size,
            self.rate_mpps,
            self.duration.as_secs_f64()
        )
    }

    fn latency_us(&self) -> f64 {
        self.avg().unwrap_or(0.0)
    }
}

/// Run latency test at a single frame size and rate
pub fn run_latency_trial(
    frame_size: u32,
    rate_mpps: f64,
    config: &Config,
    generate_and_count: impl Fn(u32, Duration) -> Result<Vec<u64>, anyhow::Error>,
) -> anyhow::Result<LatencyHistogram> {
    let duration = config.trial_duration;

    let latencies = generate_and_count(frame_size, duration)?;

    let samples: Vec<LatencySample> = latencies
        .iter()
        .map(|&us| LatencySample {
            latency_us: us,
            timestamp: Instant::now(),
        })
        .collect();

    let rate = rate_mpps;

    Ok(LatencyHistogram {
        samples,
        frame_size,
        rate_mpps: rate,
        duration,
    })
}

/// Run latency test at multiple load levels
pub fn run_latency_full(
    config: &Config,
    generate_and_count: impl Fn(u32, f64, Duration) -> Result<Vec<u64>, anyhow::Error>,
) -> anyhow::Result<LatencyTestResult> {
    let lat_cfg = &config.latency;
    let frame_sizes = config.all_frame_sizes();

    let mut all_histograms = Vec::new();

    for frame_size in &frame_sizes {
        for &load_level in &lat_cfg.load_levels {
            // Load level is a fraction of throughput (0.1 to 1.0)
            let rate_mpps = calc_load_rate(*frame_size, load_level, config.line_rate_mbps);

            let histogram = run_latency_trial(
                *frame_size,
                rate_mpps,
                config,
                |fs, dur| generate_and_count(fs, rate_mpps, dur),
            )?;

            all_histograms.push(histogram);
        }
    }

    Ok(LatencyTestResult {
        histograms: all_histograms,
        test_duration: config.trial_duration * lat_cfg.load_levels.len() as u32,
        test_type: TestType::Latency,
    })
}

/// Calculate the rate at a given load level for a frame size.
pub fn calc_load_rate(frame_size: u32, load_level: f64, line_rate_mbps: u64) -> f64 {
    // load_level is 0.1 to 1.0 (10% to 100%)
    let frame_bps = (frame_size + 20) as f64 * 8.0;
    let line_rate_bps = (line_rate_mbps as f64) * 1_000_000.0;
    let rate_bps = line_rate_bps * load_level;
    rate_bps / frame_bps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram_empty() {
        let histo = LatencyHistogram {
            samples: vec![],
            frame_size: 64,
            rate_mpps: 1.0,
            duration: Duration::from_secs(1),
        };
        assert!(histo.min().is_none());
        assert!(histo.max().is_none());
        assert!(histo.avg().is_none());
        assert!(histo.stddev().is_none());
        assert!(histo.percentile(50.0).is_none());
    }

    #[test]
    fn test_histogram_stats() {
        let samples = vec![
            LatencySample { latency_us: 10, timestamp: Instant::now() },
            LatencySample { latency_us: 20, timestamp: Instant::now() },
            LatencySample { latency_us: 30, timestamp: Instant::now() },
            LatencySample { latency_us: 40, timestamp: Instant::now() },
            LatencySample { latency_us: 50, timestamp: Instant::now() },
        ];
        let histo = LatencyHistogram {
            samples,
            frame_size: 128,
            rate_mpps: 2.0,
            duration: Duration::from_secs(1),
        };
        assert_eq!(histo.min(), Some(10));
        assert_eq!(histo.max(), Some(50));
        assert_eq!(histo.avg(), Some(30.0));
        assert_eq!(histo.percentile(50.0), Some(30));
    }

    #[test]
    fn test_calc_load_rate() {
        let rate = calc_load_rate(64, 0.5, 10000);
        // 50% of 10 Gbps at 64-byte frame
        assert!(rate > 0.0);
        assert!(rate < 100.0); // mpps
    }
}
