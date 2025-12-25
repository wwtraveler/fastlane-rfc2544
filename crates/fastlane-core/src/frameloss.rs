//! Frame loss test engine (RFC 2544 §26.3)
//!
//! Measures frame loss ratio as a function of offered load.
/// Decreases load from line rate until no loss is observed.

use std::time::Duration;

use crate::config::{Config, FrameLossConfig, TestType};
pub use crate::results::{FrameLossResult, FrameLossTrial, FrameLossTestResult};

/// Run frame loss test at a single frame size
pub fn run_frameloss_trial(
    frame_size: u32,
    config: &Config,
    tx_fn: impl Fn(f64, Duration) -> Result<(u64, u64), anyhow::Error>,
) -> anyhow::Result<Vec<FrameLossTrial>> {
    let fl_cfg = &config.frame_loss;
    let trial_duration = config.trial_duration;

    let mut results = Vec::new();
    let mut rate_multi = fl_cfg.start_pct / 100.0;

    // Decrease rate until no frame loss
    while rate_multi >= 0.05 {
        let rate = config.line_rate_mbps as f64 * rate_multi;

        let (tx, rx) = tx_fn(rate, trial_duration)?;

        let frameloss_pct = (tx as f64 - rx as f64) / (tx as f64) * 100.0;

        let trial = FrameLossTrial {
            load_pct: rate_multi * 100.0,
            rate_percent: rate_multi * 100.0,
            frame_size,
            sent: tx,
            received: rx,
            lost: tx - rx,
            tx_packets: tx,
            rx_packets: rx,
            frameloss_pct,
            duration: trial_duration,
        };
        results.push(trial);

        let no_loss = tx == rx;
        if no_loss {
            break;
        }

        rate_multi -= fl_cfg.step_pct / 100.0;
    }

    Ok(results)
}

/// Run frame loss test across all frame sizes
pub fn run_frameloss_full(
    config: &Config,
    tx_fn: impl Fn(u32, f64, Duration) -> Result<(u64, u64), anyhow::Error>,
) -> anyhow::Result<FrameLossTestResult> {
    let trial_duration = config.trial_duration;
    let frame_sizes = config.all_frame_sizes();

    let mut all_results = Vec::new();

    for frame_size in &frame_sizes {
        let frame_results = run_frameloss_trial(
            *frame_size,
            config,
            |rate, dur| tx_fn(*frame_size, rate, dur),
        )?;

        let sent: u64 = frame_results.iter().map(|t| t.sent).sum();
        let received: u64 = frame_results.iter().map(|t| t.received).sum();
        let loss_pct = if sent > 0 {
            (sent as f64 - received as f64) / sent as f64 * 100.0
        } else {
            0.0
        };
        all_results.push(FrameLossResult {
            frame_size: *frame_size,
            sent,
            received,
            loss_pct,
            trials: frame_results,
        });
    }

    let loss_threshold = 1.0; // 1% frame loss threshold as default
    let result = all_results.last().cloned().unwrap_or(FrameLossResult {
        frame_size: 0,
        sent: 0,
        received: 0,
        loss_pct: 0.0,
        trials: vec![],
    });
    Ok(FrameLossTestResult {
        result,
        loss_threshold,
        results: all_results,
        test_duration: trial_duration * frame_sizes.len() as u32,
        test_type: TestType::FrameLoss,
    })
}

/// Run frame loss test using packet ratio method (from Moongen Lua).
/// This method gradually increases load while tracking received packets.
pub fn run_frameloss_ratio(
    frame_size: u32,
    config: &Config,
    tx_fn: impl Fn(f64, Duration) -> Result<(u64, u64), anyhow::Error>,
) -> anyhow::Result<Vec<FrameLossTrial>> {
    let fl_cfg = &config.frame_loss;
    let trial_duration = config.trial_duration;

    let mut results = Vec::new();
    let mut rate_multi = fl_cfg.start_pct / 100.0;

    let mut last_no_loss = false;

    while rate_multi >= 0.05 {
        let (tx, rx) = tx_fn(rate_multi * config.line_rate_mbps as f64, trial_duration)?;

        let frameloss_pct = (tx as f64 - rx as f64) / (tx as f64) * 100.0;

        let trial = FrameLossTrial {
            load_pct: rate_multi * 100.0,
            rate_percent: rate_multi * 100.0,
            frame_size,
            sent: tx,
            received: rx,
            lost: tx - rx,
            tx_packets: tx,
            rx_packets: rx,
            frameloss_pct,
            duration: trial_duration,
        };
        results.push(trial);

        let no_loss = tx == rx;
        if no_loss && last_no_loss {
            break;
        }
        last_no_loss = no_loss;

        rate_multi -= fl_cfg.step_pct / 100.0;
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frameloss_pct_calculation() {
        let tx: u64 = 1000;
        let rx: u64 = 990;
        let frameloss_pct = (tx as f64 - rx as f64) / (tx as f64) * 100.0;
        assert!((frameloss_pct - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_zero_frameloss() {
        let tx: u64 = 1000;
        let rx: u64 = 1000;
        let frameloss_pct = (tx as f64 - rx as f64) / (tx as f64) * 100.0;
        assert_eq!(frameloss_pct, 0.0);
    }

    #[test]
    fn test_full_frameloss() {
        let tx: u64 = 1000;
        let rx: u64 = 0;
        let frameloss_pct = (tx as f64 - rx as f64) / (tx as f64) * 100.0;
        assert_eq!(frameloss_pct, 100.0);
    }
}
