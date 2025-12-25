//! Results — Test result data structures and output formatting
//!
//! All structs here are the canonical definitions shared by throughput.rs,
//! frameloss.rs, back2back.rs, latency.rs, and output formatting modules.
//!
//! Duration types are serialized as f64 seconds for JSON/CSV compatibility.

use serde::{Deserialize, Serialize, Serializer, Deserializer};
use std::time::Duration;
use crate::latency::LatencyHistogram;

/// Returns the Unix epoch as an Instant
fn epoch_instant() -> std::time::Instant {
    let now = std::time::Instant::now();
    // Use SystemTime's duration_since which works with UNIX_EPOCH (SystemTime)
    let dur = std::time::UNIX_EPOCH.duration_since(std::time::UNIX_EPOCH).unwrap();
    let epoch_as_instant = now - dur;
    epoch_as_instant
}

/// Default Instant for serde skip
fn default_instant() -> std::time::Instant {
    std::time::Instant::now()
}

/// Serialize Instant as f64 (seconds since epoch)
pub fn serialize_instant<S>(instant: &std::time::Instant, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_f64(instant.duration_since(epoch_instant()).as_secs_f64())
}

/// Deserialize f64 (seconds) to Instant
pub fn deserialize_instant<'de, D>(d: D) -> Result<std::time::Instant, D::Error>
where
    D: Deserializer<'de>,
{
    let secs = f64::deserialize(d)?;
    Ok(epoch_instant() + std::time::Duration::from_secs_f64(secs))
}

/// Serialize Duration as f64 (seconds) for JSON compatibility
pub fn serialize_duration<S>(d: &Duration, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_f64(d.as_secs_f64())
}

/// Deserialize f64 (seconds) to Duration
pub fn deserialize_duration<'de, D>(d: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let secs = f64::deserialize(d)?;
    Ok(Duration::from_secs_f64(secs))
}

/// FrameResult — result from a single frame size test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameResult {
    pub frame_size: u32,
    pub bitrate: f64,
    pub tx_packets: u64,
    pub rx_packets: u64,
    pub passed: bool,
}

/// ThroughputResult per frame size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputResult {
    pub frame_size: u32,
    pub pps: u64,
    pub mbps: f64,
}

/// Full throughput test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputTestResult {
    pub frame_results: Vec<FrameResult>,
    pub real_bitrate: f64,
    pub min_bitrate: f64,
    pub max_bitrate: f64,
    pub trials: Vec<TrialResult>,
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    pub test_duration: Duration,
    pub test_type: crate::config::TestType,
}

/// LatencyResult per frame size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyResult {
    pub frame_size: u32,
    pub min_us: f64,
    pub max_us: f64,
    pub avg_us: f64,
    pub stddev_us: f64,
    pub percentiles: Vec<(u32, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyTestResult {
    pub result: LatencyHistogram,
    pub histograms: Vec<LatencyHistogram>,
    pub load_level: f64,
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    pub test_duration: Duration,
    pub test_type: crate::config::TestType,
}

/// FrameLossTrial — data from a single frame loss trial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameLossTrial {
    pub load_pct: f64,
    pub frame_size: u32,
    pub sent: u64,
    pub received: u64,
    pub lost: u64,
    pub tx_packets: u64,
    pub rx_packets: u64,
    pub frameloss_pct: f64,
    pub rate_percent: f64,
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    pub duration: Duration,
}

/// FrameLossResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameLossResult {
    pub frame_size: u32,
    pub sent: u64,
    pub received: u64,
    pub loss_pct: f64,
    pub trials: Vec<FrameLossTrial>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameLossTestResult {
    pub result: FrameLossResult,
    pub loss_threshold: f64,
    pub results: Vec<FrameLossResult>,
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    pub test_duration: Duration,
    pub test_type: crate::config::TestType,
}

/// BackToBackResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackToBackResult {
    pub frame_size: u32,
    pub burst_size: u64,
    pub burst_us: f64,
    pub max_burst: u64,
    pub line_rate_pps: f64,
    pub trials: Vec<TrialResult>,
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    pub test_duration: Duration,
    pub test_type: crate::config::TestType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackToBackTestResult {
    pub result: BackToBackResult,
    pub threshold: f64,
    pub results: Vec<BackToBackResult>,
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    pub test_duration: Duration,
    pub test_type: crate::config::TestType,
}

/// TrialResult — the core trial type used across all tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialResult {
    pub bitrate: f64,
    pub passed: bool,
    pub tx_packets: u64,
    pub rx_packets: u64,
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    pub duration: Duration,
    #[serde(serialize_with = "serialize_instant", deserialize_with = "deserialize_instant")]
    pub timestamp: std::time::Instant,
}

/// Generic test result for output formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_type: String,
    pub frame_size: u32,
    pub result: f64,
    pub unit: String,
}

/// Format result as text
pub fn format_text(results: &[TestResult]) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "{:<20} {:>10} {:>15} {:>10}\n",
        "TEST", "FRAME", "RESULT", "UNIT"
    ));
    output.push_str(&"----------------------------------------------\n".to_string());
    for r in results {
        output.push_str(&format!(
            "{:<20} {:>10} {:>15.4} {:>10}\n",
            r.test_type, r.frame_size, r.result, r.unit
        ));
    }
    output
}

/// Format result as JSON
pub fn format_json(results: &[TestResult]) -> String {
    serde_json::to_string_pretty(&results).unwrap_or_default()
}

/// Format result as CSV
pub fn format_csv(results: &[TestResult]) -> String {
    let mut output = "test_type,frame_size,result,unit\n".to_string();
    for r in results {
        output.push_str(&format!(
            "{},{},{},{}\n",
            r.test_type, r.frame_size, r.result, r.unit
        ));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_text() {
        let results = vec![TestResult {
            test_type: "throughput".to_string(),
            frame_size: 64,
            result: 500.0,
            unit: "Mbps".to_string(),
        }];
        let text = format_text(&results);
        assert!(text.contains("throughput"));
        assert!(text.contains("64"));
    }

    #[test]
    fn test_format_json() {
        let results = vec![TestResult {
            test_type: "latency".to_string(),
            frame_size: 128,
            result: 0.5,
            unit: "us".to_string(),
        }];
        let json = format_json(&results);
        assert!(json.contains("latency"));
    }

    #[test]
    fn test_format_csv() {
        let results = vec![TestResult {
            test_type: "frame_loss".to_string(),
            frame_size: 256,
            result: 0.01,
            unit: "%".to_string(),
        }];
        let csv = format_csv(&results);
        assert!(csv.contains("frame_loss"));
        assert!(csv.contains("256"));
    }

    #[test]
    fn test_trial_result_serde() {
        let t = TrialResult {
            bitrate: 1000.0,
            passed: true,
            tx_packets: 1000000,
            rx_packets: 999999,
            duration: Duration::from_secs(1),
            timestamp: std::time::Instant::now(),
        };
        let json = serde_json::to_string(&t).unwrap();
        let t2: TrialResult = serde_json::from_str(&json).unwrap();
        assert_eq!(t2.bitrate, 1000.0);
    }
}
