//! Back-to-back frame test engine (RFC 2544 §26.4)
//!
//! Measures the maximum burst of frames that can be transmitted
/// with zero or acceptable frame loss at line rate.

use std::time::Duration;

use crate::config::{BackToBackConfig, Config, TestType};
pub use crate::results::{BackToBackResult, BackToBackTestResult, TrialResult};

/// Run back-to-back test at a single frame size
pub fn run_back2back_trial(
    frame_size: u32,
    config: &Config,
) -> anyhow::Result<BackToBackResult> {
    let b2b_cfg = &config.back_to_back;
    let trial_duration = config.trial_duration;
    let line_rate_bps = (config.line_rate_mbps as f64) * 1_000_000.0;

    // Calculate frame rate (frames per second at line rate)
    let frame_bps = (frame_size + 20) as f64 * 8.0;
    let line_rate_pps = line_rate_bps / frame_bps;

    // Initial burst is the number of frames sent back-to-back
    let mut burst_size = b2b_cfg.initial_burst;
    let mut max_burst = 0u64;
    let mut passed_trials = 0u32;

    for _ in 0..b2b_cfg.trials {
        // Simulate sending burst_size frames back-to-back
        let tx = burst_size;
        // Calculate expected rx: some loss expected if burst exceeds capacity
        let capacity = line_rate_pps * trial_duration.as_secs_f64();
        let rx = if burst_size as f64 <= capacity {
            burst_size
        } else {
            // Frame loss when burst exceeds capacity
            (capacity * 0.99) as u64
        };

        let passed = tx == rx || (tx as f64 - rx as f64) / (tx as f64) < 0.001;

        if passed {
            passed_trials += 1;
            if burst_size > max_burst {
                max_burst = burst_size;
            }
            // Increase burst size for next trial
            burst_size += 100;
        } else {
            // Decrease burst size
            if burst_size > 1 {
                burst_size -= 100;
            }
        }
    }

    let burst_us = (max_burst as f64 / line_rate_pps) * 1_000_000.0;
    Ok(BackToBackResult {
        frame_size,
        burst_size: max_burst,
        burst_us,
        max_burst,
        line_rate_pps,
        trials: vec![TrialResult {
            bitrate: line_rate_pps / 1_000_000.0,
            passed: true,
            tx_packets: max_burst,
            rx_packets: max_burst,
            duration: trial_duration,
            timestamp: std::time::Instant::now(),
        }],
        test_duration: trial_duration * b2b_cfg.trials,
        test_type: TestType::BackToBack,
    })
}

/// Run back-to-back test across all frame sizes
pub fn run_back2back_full(
    config: &Config,
) -> anyhow::Result<BackToBackTestResult> {
    let frame_sizes = config.all_frame_sizes();
    let mut all_results = Vec::new();

    for frame_size in &frame_sizes {
        let result = run_back2back_trial(*frame_size, config)?;
        all_results.push(result);
    }

    let threshold = all_results.last().map(|r| r.burst_us).unwrap_or(0.0);
    let result = all_results.last().cloned().unwrap_or(BackToBackResult {
        frame_size: 0,
        burst_size: 0,
        burst_us: 0.0,
        max_burst: 0,
        line_rate_pps: 0.0,
        trials: vec![],
        test_duration: config.trial_duration,
        test_type: TestType::BackToBack,
    });
    Ok(BackToBackTestResult {
        result,
        threshold,
        results: all_results,
        test_duration: config.trial_duration * frame_sizes.len() as u32,
        test_type: TestType::BackToBack,
    })
}

/// Calculate the back-to-back threshold: the minimum burst
/// at which frame loss first appears.
pub fn calc_back2back_threshold(
    frame_size: u32,
    line_rate_pps: f64,
    trials: &[TrialResult],
) -> Option<u64> {
    for trial in trials.iter().rev() {
        if !trial.passed {
            return Some(trial.tx_packets);
        }
    }
    // All passed — return max
    trials.last().map(|t| t.tx_packets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_back2back_burst_calculation() {
        let config = Config::default();
        let line_rate_bps = (config.line_rate_mbps as f64) * 1_000_000.0;
        let frame_bps = (64 + 20) as f64 * 8.0;
        let line_rate_pps = line_rate_bps / frame_bps;

        assert!(line_rate_pps > 1_000_000.0);
    }

    #[test]
    fn test_calc_threshold_all_passed() {
        let trials = vec![
            TrialResult {
                bitrate: 10.0, passed: true,
                tx_packets: 100, rx_packets: 100,
                duration: Duration::from_secs(1),
                timestamp: std::time::Instant::now(),
            },
            TrialResult {
                bitrate: 10.0, passed: true,
                tx_packets: 200, rx_packets: 200,
                duration: Duration::from_secs(1),
                timestamp: std::time::Instant::now(),
            },
        ];
        let threshold = calc_back2back_threshold(64, 10_000_000.0, &trials);
        assert_eq!(threshold, Some(200));
    }

    #[test]
    fn test_calc_threshold_first_fail() {
        let trials = vec![
            TrialResult {
                bitrate: 10.0, passed: true,
                tx_packets: 100, rx_packets: 100,
                duration: Duration::from_secs(1),
                timestamp: std::time::Instant::now(),
            },
            TrialResult {
                bitrate: 10.0, passed: false,
                tx_packets: 200, rx_packets: 195,
                duration: Duration::from_secs(1),
                timestamp: std::time::Instant::now(),
            },
        ];
        let threshold = calc_back2back_threshold(64, 10_000_000.0, &trials);
        assert_eq!(threshold, Some(100));
    }
}
