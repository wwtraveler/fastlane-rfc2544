//! Configuration types and parsing for fastlane-rfc2544
//!
//! Supports YAML and JSON config files, with comprehensive CLI overrides.
//! All arguments from the Python ByteBlower, Go master, and Lua Moongen implementations
//! are represented here.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

// ── Standard frame sizes per RFC 2544 §9.1 ──────────────────────────────

/// Standard RFC 2544 frame sizes in bytes (excluding CRC)
pub const STANDARD_FRAME_SIZES: &[u32] = &[64, 128, 256, 512, 1024, 1280, 1518];
pub const JUMBO_FRAME_SIZE: u32 = 9000;
pub const IMIX_SIZES: &[u32] = &[64, 128, 256, 512, 1024, 1280, 1518];

// ── Test types ───────────────────────────────────────────────────────────

/// RFC 2544 test types — core + extended from Go and Python implementations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestType {
    /// Throughput test (RFC 2544 §26.1)
    Throughput,
    /// Latency test (RFC 2544 §26.2)
    Latency,
    /// Frame loss test (RFC 2544 §26.3)
    FrameLoss,
    /// Back-to-back test (RFC 2544 §26.4)
    BackToBack,
    /// System recovery test (RFC 2544 §26.5)
    SystemRecovery,
    /// Reset test (RFC 2544 §26.6)
    Reset,
    /// Y.1564 service configuration test
    Y1564Config,
    /// Y.1564 service performance test
    Y1564Perf,
    /// Full Y.1564 test suite
    Y1564,
    /// RFC 2889 LAN switch forwarding rate test
    Rfc2889Forwarding,
    /// RFC 2889 address caching test
    Rfc2889Caching,
    /// RFC 2889 address learning test
    Rfc2889Learning,
    /// RFC 2889 broadcast forwarding test
    Rfc2889Broadcast,
    /// RFC 2889 congestion control test
    Rfc2889Congestion,
    /// RFC 6349 TCP throughput test
    Rfc6349Throughput,
    /// RFC 6349 path analysis test
    Rfc6349Path,
    /// Y.1731 delay measurement
    Y1731Delay,
    /// Y.1731 loss measurement
    Y1731Loss,
    /// Y.1731 synthetic loss measurement
    Y1731SLM,
    /// Y.1731 loopback
    Y1731Loopback,
    /// MEF configuration test
    MefConfig,
    /// MEF performance test
    MefPerf,
    /// Full MEF test suite
    Mef,
    /// TSN gate timing accuracy test
    TsnTiming,
    /// TSN traffic class isolation test
    TsnIsolation,
    /// TSN latency test
    TsnLatency,
    /// Full TSN test suite
    Tsn,
}

impl Default for TestType {
    fn default() -> Self { Self::Throughput }
}

impl std::fmt::Display for TestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestType::Throughput => write!(f, "throughput"),
            TestType::Latency => write!(f, "latency"),
            TestType::FrameLoss => write!(f, "frame_loss"),
            TestType::BackToBack => write!(f, "back_to_back"),
            TestType::SystemRecovery => write!(f, "system_recovery"),
            TestType::Reset => write!(f, "reset"),
            TestType::Y1564Config => write!(f, "y1564_config"),
            TestType::Y1564Perf => write!(f, "y1564_perf"),
            TestType::Y1564 => write!(f, "y1564"),
            TestType::Rfc2889Forwarding => write!(f, "rfc2889_forwarding"),
            TestType::Rfc2889Caching => write!(f, "rfc2889_caching"),
            TestType::Rfc2889Learning => write!(f, "rfc2889_learning"),
            TestType::Rfc2889Broadcast => write!(f, "rfc2889_broadcast"),
            TestType::Rfc2889Congestion => write!(f, "rfc2889_congestion"),
            TestType::Rfc6349Throughput => write!(f, "rfc6349_throughput"),
            TestType::Rfc6349Path => write!(f, "rfc6349_path"),
            TestType::Y1731Delay => write!(f, "y1731_delay"),
            TestType::Y1731Loss => write!(f, "y1731_loss"),
            TestType::Y1731SLM => write!(f, "y1731_slm"),
            TestType::Y1731Loopback => write!(f, "y1731_loopback"),
            TestType::MefConfig => write!(f, "mef_config"),
            TestType::MefPerf => write!(f, "mef_perf"),
            TestType::Mef => write!(f, "mef"),
            TestType::TsnTiming => write!(f, "tsn_timing"),
            TestType::TsnIsolation => write!(f, "tsn_isolation"),
            TestType::TsnLatency => write!(f, "tsn_latency"),
            TestType::Tsn => write!(f, "tsn"),
        }
    }
}

// ── Rate limiting modes (from Moongen Lua) ───────────────────────────────

/// Rate limiting mode for packet transmission
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateType {
    /// Constant Bit Rate (CBR) — steady rate, may use extra thread
    Cbr,
    /// Poison mode — send invalid packets at line rate, DUT ignores them
    Poison,
    /// Hardware rate limiting — most reliable, limited to NIC support
    Hw,
}

impl Default for RateType {
    fn default() -> Self { Self::Cbr }
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

// ── Output formats ───────────────────────────────────────────────────────

/// Result output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
    Html,
}

impl Default for OutputFormat {
    fn default() -> Self { Self::Text }
}

// ── Core configuration ───────────────────────────────────────────────────

/// Complete test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // Interface settings
    #[serde(default)]
    pub interface: String,
    #[serde(default = "default_line_rate")]
    pub line_rate_mbps: u64,
    #[serde(default)]
    pub auto_detect_nic: bool,

    // Test selection
    #[serde(default)]
    pub test_type: TestType,
    #[serde(default = "default_frame_size")]
    pub frame_sizes: Vec<u32>,
    #[serde(default)]
    pub include_jumbo: bool,

    // Timing (from Python ByteBlower defaults)
    #[serde(default = "default_trial_duration")]
    pub trial_duration: Duration,
    #[serde(default = "default_warmup")]
    pub warmup_period: Duration,

    // Throughput test (RFC 2544 §26.1)
    #[serde(default)]
    pub throughput: ThroughputConfig,

    // Latency test (RFC 2544 §26.2)
    #[serde(default)]
    pub latency: LatencyConfig,

    // Frame loss test (RFC 2544 §26.3)
    #[serde(default)]
    pub frame_loss: FrameLossConfig,

    // Back-to-back test (RFC 2544 §26.4)
    #[serde(default)]
    pub back_to_back: BackToBackConfig,

    // Packet generation
    #[serde(default)]
    pub udp_src_port: u16,
    #[serde(default)]
    pub udp_dst_port: u16,
    #[serde(default)]
    pub payload_pattern: u8,

    // Port config (from ByteBlower endpoint/portal modes)
    #[serde(default)]
    pub port_mode: PortMode,

    // Platform settings
    #[serde(default)]
    pub hw_timestamp: bool,
    #[serde(default)]
    pub use_pacing: bool,
    #[serde(default = "default_batch_size")]
    pub batch_size: u32,
    #[serde(default)]
    pub num_queues: u32,
    #[serde(default)]
    pub rate_type: RateType,
    #[serde(default)]
    pub verbose: bool,

    // Output
    #[serde(default)]
    pub output_format: OutputFormat,
    #[serde(default)]
    pub output_path: Option<PathBuf>,

    // Y.1564 EtherSAM config
    #[serde(default)]
    pub y1564: Y1564Config,

    // RFC 2889 LAN switch
    #[serde(default)]
    pub rfc2889: Rfc2889Config,

    // RFC 6349 TCP
    #[serde(default)]
    pub rfc6349: Rfc6349Config,

    // Y.1731 OAM
    #[serde(default)]
    pub y1731: Y1731Config,

    // MEF service activation
    #[serde(default)]
    pub mef: MefConfig,

    // TSN
    #[serde(default)]
    pub tsn: TsnConfig,

    // IMIX frame distribution (for mixed-size tests)
    #[serde(default)]
    pub imix: ImixConfig,

    // Multi-port / dual-NIC config
    #[serde(default)]
    pub rx_port: Option<u32>,
    #[serde(default)]
    pub tx_port: Option<u32>,
}

// ── Throughput config (binary search) ───────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputConfig {
    /// Initial rate as percentage of line rate (default: 100)
    #[serde(default = "default_initial_rate_pct")]
    pub initial_rate_pct: f64,
    /// Binary search resolution in % (from Go: default 0.1)
    #[serde(default = "default_resolution_pct")]
    pub resolution_pct: f64,
    /// Maximum binary search iterations (default: 25 per ByteBlower)
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
    /// Acceptable frame loss fraction (default: 0.001 = 0.1%)
    #[serde(default = "default_tolerated_frame_loss")]
    pub tolerated_frame_loss: f64,
}

impl Default for ThroughputConfig {
    fn default() -> Self {
        Self {
            initial_rate_pct: default_initial_rate_pct(),
            resolution_pct: default_resolution_pct(),
            max_iterations: default_max_iterations(),
            tolerated_frame_loss: default_tolerated_frame_loss(),
        }
    }
}

// ── Latency config ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyConfig {
    /// Number of latency samples per trial (default: 1000)
    #[serde(default = "default_latency_samples")]
    pub samples: u32,
    /// Load levels to test as fractions of throughput (default: 10% to 100%)
    #[serde(default = "default_load_levels")]
    pub load_levels: Vec<f64>,
    /// Rate limit for latency measurement packets (seconds)
    #[serde(default = "default_latency_rate_limit")]
    pub rate_limit: f64,
}

impl Default for LatencyConfig {
    fn default() -> Self {
        Self {
            samples: default_latency_samples(),
            load_levels: default_load_levels(),
            rate_limit: default_latency_rate_limit(),
        }
    }
}

// ── Frame loss config ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameLossConfig {
    /// Starting offered load % (default: 100)
    #[serde(default = "default_fl_start_pct")]
    pub start_pct: f64,
    /// Ending offered load % (default: 10)
    #[serde(default = "default_fl_end_pct")]
    pub end_pct: f64,
    /// Step size in % (default: 10)
    #[serde(default = "default_fl_step_pct")]
    pub step_pct: f64,
}

impl Default for FrameLossConfig {
    fn default() -> Self {
        Self {
            start_pct: default_fl_start_pct(),
            end_pct: default_fl_end_pct(),
            step_pct: default_fl_step_pct(),
        }
    }
}

// ── Back-to-back config ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackToBackConfig {
    /// Initial burst size (default: 1000)
    #[serde(default = "default_b2b_initial")]
    pub initial_burst: u64,
    /// Number of trials per burst size (default: 50)
    #[serde(default = "default_b2b_trials")]
    pub trials: u32,
}

impl Default for BackToBackConfig {
    fn default() -> Self {
        Self {
            initial_burst: default_b2b_initial(),
            trials: default_b2b_trials(),
        }
    }
}

// ── Y.1564 EtherSAM ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Y1564Service {
    pub service_id: u32,
    pub service_name: String,
    pub cir_mbps: f64,
    pub eir_mbps: f64,
    pub cbs_bytes: u32,
    pub ebs_bytes: u32,
    pub fd_threshold_ms: f64,
    pub fdv_threshold_ms: f64,
    pub flr_threshold_pct: f64,
    pub cos: u8,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Y1564Config {
    pub services: Vec<Y1564Service>,
    #[serde(default = "default_config_steps")]
    pub config_steps: Vec<f64>,
    #[serde(default = "default_step_duration")]
    pub step_duration: Duration,
    #[serde(default = "default_perf_duration")]
    pub perf_duration: Duration,
    #[serde(default)]
    pub run_config_test: bool,
    #[serde(default)]
    pub run_perf_test: bool,
}

impl Default for Y1564Config {
    fn default() -> Self {
        Self {
            services: vec![],
            config_steps: default_config_steps(),
            step_duration: default_step_duration(),
            perf_duration: default_perf_duration(),
            run_config_test: true,
            run_perf_test: true,
        }
    }
}

// ── RFC 2889 LAN switch ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rfc2889Config {
    #[serde(default = "default_rfc2889_port_count")]
    pub port_count: u32,
    #[serde(default = "default_rfc2889_addr_count")]
    pub address_count: u32,
    #[serde(default = "default_rfc2889_trial_duration")]
    pub trial_duration: Duration,
    #[serde(default = "default_rfc2889_loss_pct")]
    pub acceptable_loss_pct: f64,
}

impl Default for Rfc2889Config {
    fn default() -> Self {
        Self {
            port_count: default_rfc2889_port_count(),
            address_count: default_rfc2889_addr_count(),
            trial_duration: default_rfc2889_trial_duration(),
            acceptable_loss_pct: default_rfc2889_loss_pct(),
        }
    }
}

// ── RFC 6349 TCP ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rfc6349Config {
    #[serde(default = "default_rfc6349_target")]
    pub target_rate_mbps: f64,
    #[serde(default = "default_rfc6349_mss")]
    pub mss: u32,
    #[serde(default = "default_rfc6349_rwnd")]
    pub rwnd: u32,
    #[serde(default = "default_rfc6349_duration")]
    pub test_duration: Duration,
    #[serde(default = "default_rfc6349_streams")]
    pub parallel_streams: u32,
}

impl Default for Rfc6349Config {
    fn default() -> Self {
        Self {
            target_rate_mbps: default_rfc6349_target(),
            mss: default_rfc6349_mss(),
            rwnd: default_rfc6349_rwnd(),
            test_duration: default_rfc6349_duration(),
            parallel_streams: default_rfc6349_streams(),
        }
    }
}

// ── Y.1731 OAM ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Y1731Config {
    #[serde(default = "default_y1731_mep_id")]
    pub mep_id: u32,
    #[serde(default = "default_y1731_meg_level")]
    pub meg_level: u8,
    #[serde(default = "default_y1731_meg_id")]
    pub meg_id: String,
    #[serde(default = "default_y1731_ccm_interval")]
    pub ccm_interval: u32,
    #[serde(default = "default_y1731_probe_count")]
    pub probe_count: u32,
    #[serde(default = "default_y1731_probe_interval")]
    pub probe_interval: Duration,
}

impl Default for Y1731Config {
    fn default() -> Self {
        Self {
            mep_id: default_y1731_mep_id(),
            meg_level: default_y1731_meg_level(),
            meg_id: default_y1731_meg_id(),
            ccm_interval: default_y1731_ccm_interval(),
            probe_count: default_y1731_probe_count(),
            probe_interval: default_y1731_probe_interval(),
        }
    }
}

// ── MEF ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MefConfig {
    pub cir_mbps: f64,
    pub eir_mbps: f64,
    pub cbs_bytes: u32,
    pub ebs_bytes: u32,
    pub fd_threshold_us: f64,
    pub fdv_threshold_us: f64,
    #[serde(default = "default_mef_flr_pct")]
    pub flr_threshold_pct: f64,
    #[serde(default = "default_mef_avail_pct")]
    pub avail_threshold_pct: f64,
    #[serde(default = "default_mef_config_duration")]
    pub config_duration: Duration,
    #[serde(default = "default_mef_perf_duration")]
    pub perf_duration: Duration,
}

impl Default for MefConfig {
    fn default() -> Self {
        Self {
            cir_mbps: 100.0,
            eir_mbps: 0.0,
            cbs_bytes: 12000,
            ebs_bytes: 0,
            fd_threshold_us: 10000.0,
            fdv_threshold_us: 5000.0,
            flr_threshold_pct: default_mef_flr_pct(),
            avail_threshold_pct: default_mef_avail_pct(),
            config_duration: default_mef_config_duration(),
            perf_duration: default_mef_perf_duration(),
        }
    }
}

// ── TSN ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsnConfig {
    #[serde(default = "default_tsn_classes")]
    pub num_classes: u32,
    #[serde(default = "default_tsn_cycle_ns")]
    pub cycle_time_ns: u64,
    #[serde(default = "default_tsn_latency_ns")]
    pub max_latency_ns: u64,
    #[serde(default = "default_tsn_jitter_ns")]
    pub max_jitter_ns: u64,
    #[serde(default = "default_tsn_sync_ns")]
    pub max_sync_offset_ns: u64,
    #[serde(default = "default_tsn_duration")]
    pub test_duration: Duration,
    #[serde(default = "default_tsn_frame_size")]
    pub frame_size: u32,
}

impl Default for TsnConfig {
    fn default() -> Self {
        Self {
            num_classes: default_tsn_classes(),
            cycle_time_ns: default_tsn_cycle_ns(),
            max_latency_ns: default_tsn_latency_ns(),
            max_jitter_ns: default_tsn_jitter_ns(),
            max_sync_offset_ns: default_tsn_sync_ns(),
            test_duration: default_tsn_duration(),
            frame_size: default_tsn_frame_size(),
        }
    }
}

// ── IMIX ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImixConfig {
    /// Frame distribution for IMIX test: [64:10%, 128:20%, 256:30%, 512:25%, 1024:10%, 1280:3%, 1518:2%]
    #[serde(default = "default_imix_distribution")]
    pub distribution: Vec<f64>,
    /// Number of IMIX frame sets
    #[serde(default = "default_imix_count")]
    pub count: u32,
}

impl Default for ImixConfig {
    fn default() -> Self {
        Self {
            distribution: default_imix_distribution(),
            count: default_imix_count(),
        }
    }
}

// ── Port mode ─────────────────────────────────────────────────────────────

/// Port mode: physical interface (ByteBlower Port) or endpoint (ByteBlower Endpoint)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortMode {
    /// Physical port mode
    Port,
    /// Endpoint mode (with NAT discovery)
    Endpoint,
}

impl Default for PortMode {
    fn default() -> Self { Self::Port }
}

// ── Helper default functions ──────────────────────────────────────────────

fn default_line_rate() -> u64 { 10000 } // 10 Gbps
fn default_trial_duration() -> Duration { Duration::from_secs(60) }
fn default_warmup() -> Duration { Duration::from_secs(2) }
fn default_resolution_pct() -> f64 { 0.1 }
fn default_max_iterations() -> u32 { 25 }
fn default_tolerated_frame_loss() -> f64 { 0.001 }
fn default_initial_rate_pct() -> f64 { 100.0 }
fn default_latency_samples() -> u32 { 1000 }
fn default_load_levels() -> Vec<f64> {
    (1..=10).map(|i| i as f64 * 10.0).collect()
}
fn default_latency_rate_limit() -> f64 { 0.001 } // seconds
fn default_fl_start_pct() -> f64 { 100.0 }
fn default_fl_end_pct() -> f64 { 10.0 }
fn default_fl_step_pct() -> f64 { 10.0 }
fn default_b2b_initial() -> u64 { 1000 }
fn default_b2b_trials() -> u32 { 50 }
fn default_batch_size() -> u32 { 32 }
fn default_config_steps() -> Vec<f64> { vec![25.0, 50.0, 75.0, 100.0] }
fn default_step_duration() -> Duration { Duration::from_secs(60) }
fn default_perf_duration() -> Duration { Duration::from_secs(900) }
fn default_rfc2889_port_count() -> u32 { 2 }
fn default_rfc2889_addr_count() -> u32 { 8192 }
fn default_rfc2889_trial_duration() -> Duration { Duration::from_secs(60) }
fn default_rfc2889_loss_pct() -> f64 { 0.0 }
fn default_rfc6349_target() -> f64 { 0.0 }
fn default_rfc6349_mss() -> u32 { 1460 }
fn default_rfc6349_rwnd() -> u32 { 65535 }
fn default_rfc6349_duration() -> Duration { Duration::from_secs(30) }
fn default_rfc6349_streams() -> u32 { 1 }
fn default_y1731_mep_id() -> u32 { 1 }
fn default_y1731_meg_level() -> u8 { 4 }
fn default_y1731_meg_id() -> String { "DEFAULT-MEG".to_string() }
fn default_y1731_ccm_interval() -> u32 { 1000 }
fn default_y1731_probe_count() -> u32 { 100 }
fn default_y1731_probe_interval() -> Duration { Duration::from_secs(1) }
fn default_mef_flr_pct() -> f64 { 0.01 }
fn default_mef_avail_pct() -> f64 { 99.99 }
fn default_mef_config_duration() -> Duration { Duration::from_secs(60) }
fn default_mef_perf_duration() -> Duration { Duration::from_secs(900) }
fn default_tsn_classes() -> u32 { 8 }
fn default_tsn_cycle_ns() -> u64 { 1_000_000 }
fn default_tsn_latency_ns() -> u64 { 100_000 }
fn default_tsn_jitter_ns() -> u64 { 10_000 }
fn default_tsn_sync_ns() -> u64 { 1_000 }
fn default_tsn_duration() -> Duration { Duration::from_secs(60) }
fn default_tsn_frame_size() -> u32 { 128 }
fn default_imix_distribution() -> Vec<f64> {
    vec![10.0, 20.0, 30.0, 25.0, 10.0, 3.0, 2.0]
}
fn default_imix_count() -> u32 { 1000 }

fn default_frame_size() -> Vec<u32> {
    STANDARD_FRAME_SIZES.to_vec()
}

// ── Default Config ─────────────────────────────────────────────────────────

impl Default for Config {
    fn default() -> Self {
        Self {
            interface: String::new(),
            line_rate_mbps: default_line_rate(),
            auto_detect_nic: true,
            test_type: TestType::default(),
            frame_sizes: default_frame_size(),
            include_jumbo: false,
            trial_duration: default_trial_duration(),
            warmup_period: default_warmup(),
            throughput: ThroughputConfig::default(),
            latency: LatencyConfig::default(),
            frame_loss: FrameLossConfig::default(),
            back_to_back: BackToBackConfig::default(),
            udp_src_port: 42,
            udp_dst_port: 42,
            payload_pattern: 0x0f,
            port_mode: PortMode::default(),
            hw_timestamp: true,
            use_pacing: true,
            batch_size: default_batch_size(),
            num_queues: 1,
            rate_type: RateType::default(),
            verbose: false,
            output_format: OutputFormat::default(),
            output_path: None,
            y1564: Y1564Config::default(),
            rfc2889: Rfc2889Config::default(),
            rfc6349: Rfc6349Config::default(),
            y1731: Y1731Config::default(),
            mef: MefConfig::default(),
            tsn: TsnConfig::default(),
            imix: ImixConfig::default(),
            rx_port: None,
            tx_port: None,
        }
    }
}

// ── Config loading helpers ─────────────────────────────────────────────────

impl Config {
    /// Load configuration from a YAML file
    pub fn from_yaml(path: &str) -> anyhow::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let cfg: Self = serde_yaml::from_str(&data)?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Load configuration from a JSON file
    pub fn from_json(path: &str) -> anyhow::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let cfg: Self = serde_json::from_str(&data)?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Save configuration to a YAML file
    pub fn to_yaml(&self, path: &str) -> anyhow::Result<()> {
        let data = serde_yaml::to_string(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.interface.is_empty() {
            anyhow::bail!("interface is required");
        }
        if self.trial_duration.as_secs() == 0 {
            anyhow::bail!("trial_duration must be > 0");
        }
        if self.throughput.resolution_pct <= 0.0 || self.throughput.resolution_pct > 10.0 {
            anyhow::bail!("throughput resolution must be between 0 and 10%");
        }
        Ok(())
    }

    /// Get the full set of frame sizes including jumbo
    pub fn all_frame_sizes(&self) -> Vec<u32> {
        let mut sizes = self.frame_sizes.clone();
        if self.include_jumbo && !sizes.contains(&JUMBO_FRAME_SIZE) {
            sizes.push(JUMBO_FRAME_SIZE);
        }
        sizes.sort();
        sizes
    }
}
