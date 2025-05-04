//! Throughput test engine (RFC 2544 §26.1)
//!
//! Implements the binary search algorithm to find the maximum throughput
//! at which zero frames are lost, for each frame size.
//!
//! Algorithm (from ByteBlower Python implementation):
//! 1. Start at initial_bitrate (default: line rate)
//! 2. Run a trial, observe if frame loss <= tolerated_frame_loss
//! 3. If pass: raise min_bitrate, try higher
//!    If fail: lower max_bitrate
//! 4. Repeat until (max - min) / 2 < resolution
//!    or max_iterations reached

use std::time::{Duration, Instant};

use crate::config::{ThroughputConfig, Config, TestType};
use crate::results::{FrameResult, TrialResult, ThroughputTestResult};

/// Maximum trials per frame size (ByteBlower Python)
const MAX_TRIALS_PER_FRAME: u32 = 25;

/// Result of a single throughput trial
pub struct TrialOutput {
    pub passed: bool,
    pub test_bitrate: f64,
    pub tx_packets: u64,
    pub rx_packets: u64,
    pub duration: Duration,
    pub timestamp: Instant,
}

/// Run the throughput test for a single frame size using binary search.
/// This is the core RFC 2544 §26.1 algorithm.
pub fn run_throughput_trial(
    frame_size: u32,
    config: &Config,
    initial_bitrate: f64,
    tx_fn: impl Fn(u64) -> Result<(u64, u64), anyhow::Error>,
) -> anyhow::Result<ThroughputTestResult> {
    let trial_duration = config.trial_duration;
    let max_iterations = config.throughput.max_iterations;
    let resolution = config.throughput.resolution_pct;
    let tolerated_loss = config.throughput.tolerated_frame_loss;

    let mut min_bitrate = 0.0;
    let mut max_bitrate = initial_bitrate;
    let mut test_bitrate = initial_bitrate;
    let mut trial_count = 0u32;
    let mut trials = Vec::new();

    let test_start = Instant::now();

    loop {
        let diff = (max_bitrate - min_bitrate) / 2.0;

        // Check termination
        if (max_bitrate <= min_bitrate) || diff < resolution {
            break;
        }
        if trial_count > max_iterations || trial_count > MAX_TRIALS_PER_FRAME {
            break;
        }

        // Run a trial at test_bitrate
        let trial_start = Instant::now();
        let (tx, rx) = tx_fn(trial_duration.as_secs())?;
        let trial_duration_actual = trial_start.elapsed();

        let loss_ratio = 1.0 - (rx as f64 / tx as f64);
        let passed = loss_ratio <= tolerated_loss;

        let trial_result = TrialResult {
            bitrate: test_bitrate,
            passed,
            tx_packets: tx,
            rx_packets: rx,
            duration: trial_duration_actual,
            timestamp: Instant::now(),
        };
        trials.push(trial_result);

        // Update binary search bounds (ByteBlower algorithm)
        let (new_min, new_max, new_bitrate) = if test_bitrate > max_bitrate {
            (min_bitrate, test_bitrate, test_bitrate)
        } else if test_bitrate <= min_bitrate {
            let next_min = test_bitrate;
            let next_max = test_bitrate + diff;
            let next_bitrate = test_bitrate + diff;
            (next_min, next_max, next_bitrate)
        } else {
            if !passed {
                // Frame loss at test_bitrate — max drops
                let next_max = test_bitrate;
                let next_min = test_bitrate - diff;
                let next_bitrate = test_bitrate - diff;
                (next_min, next_max, next_bitrate)
            } else {
                // No loss — min rises
                let next_min = test_bitrate;
                let next_max = test_bitrate + diff;
                let next_bitrate = test_bitrate + diff;
                (next_min, next_max, next_bitrate)
            }
        };

        min_bitrate = new_min;
        max_bitrate = new_max;
        test_bitrate = new_bitrate;
        trial_count += 1;
    }

    let real_bitrate = if !trials.is_empty() {
        trials.last().unwrap().bitrate
    } else {
        test_bitrate
    };

    let frame_results: Vec<FrameResult> = trials
        .iter()
        .map(|t| FrameResult {
            frame_size,
            bitrate: t.bitrate,
            tx_packets: t.tx_packets,
            rx_packets: t.rx_packets,
            passed: t.passed,
        })
        .collect();

    Ok(ThroughputTestResult {
        frame_results,
        real_bitrate,
        min_bitrate,
        max_bitrate,
        trials,
        test_duration: test_start.elapsed(),
        test_type: TestType::Throughput,
    })
}

/// Calculate the next bitrate for the binary search based on test status.
/// Mirrors the _rfc2544_throughput() function from ByteBlower Python.
pub fn calc_next_bitrate(
    test_status: bool,
    test_bitrate: f64,
    min_bitrate: f64,
    max_bitrate: f64,
    diff: f64,
) -> (f64, f64, f64) {
    let (mut min_b, mut max_b) = if min_bitrate > max_bitrate {
        (max_bitrate, min_bitrate)
    } else {
        (min_bitrate, max_bitrate)
    };

    let current_diff = (max_b - min_b) / 2.0;
    if current_diff == 0.0 {
        min_b = 0.5 * min_b;
    }

    if test_status {
        if min_b == 0.0 {
            // First pass — double the bitrate
            let new_bitrate = test_bitrate * 2.0;
            (test_bitrate, new_bitrate, new_bitrate)
        } else {
            let new_min = test_bitrate;
            let new_max = test_bitrate + current_diff;
            let new_bitrate = test_bitrate + current_diff;
            (new_min, new_max, new_bitrate)
        }
    } else {
        let new_max = test_bitrate;
        let new_bitrate = test_bitrate - current_diff;
        let new_min = new_bitrate;
        (new_min, new_max, new_bitrate)
    }
}

/// Calculate line rate from frame size and PPS.
pub fn pps_to_bps(pps: u64, frame_size: u32) -> f64 {
    // Frame size includes 20 bytes Ethernet header + FCS, plus padding
    (pps as f64) * ((frame_size + 20) as f64) * 8.0
}

/// Calculate PPS from bit rate and frame size.
pub fn bps_to_pps(bitrate_bps: f64, frame_size: u32) -> f64 {
    bitrate_bps / ((frame_size + 20) as f64 * 8.0)
}

/// Calculate required bit rate for a given frame size at a target PPS.
pub fn frame_rate_mbps(pps: f64, frame_size: u32) -> f64 {
    pps * ((frame_size + 20) as f64) * 8.0 / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pps_to_bps() {
        // 1 Mpps at 64 byte frames = 1e6 * 84 * 8 = 6.72 Gbps
        let bps = pps_to_bps(1_000_000, 64);
        assert!((bps - 6_720_000_000.0).abs() < 1000.0);
    }

    #[test]
    fn test_bps_to_pps() {
        let pps = bps_to_pps(6_720_000_000.0, 64);
        assert!((pps - 1_000_000.0).abs() < 1.0);
    }

    #[test]
    fn test_calc_next_bitrate_pass() {
        let (min, max, bt) = calc_next_bitrate(true, 500.0, 0.0, 500.0, 250.0);
        assert_eq!(min, 500.0);
        assert_eq!(max, 750.0);
        assert_eq!(bt, 750.0);
    }

    #[test]
    fn test_calc_next_bitrate_fail() {
        let (min, max, bt) = calc_next_bitrate(false, 500.0, 0.0, 500.0, 250.0);
        assert_eq!(max, 500.0);
        assert!((min - 375.0).abs() < 0.1);
        assert!((bt - 375.0).abs() < 0.1);
    }

    #[test]
    fn test_frame_rate_mbps() {
        // 1 Gbps at 64 byte frames:
        // pps = 1e9 / (84 * 8) = ~1.488 Mpps
        // rate = 1.488e6 * 84 * 8 = ~1e9
        let pps = bps_to_pps(1_000_000_000.0, 64);
        let rate = frame_rate_mbps(pps, 64);
        assert!((rate - 1_000_000_000.0).abs() < 100.0);
    }
}
