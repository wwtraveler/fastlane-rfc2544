//! fastlane-core: RFC 2544 test engine
//!
//! Implements all four core RFC 2544 tests plus extended protocol tests:
//! - **Throughput** (Section 26.1): Binary search for max rate with 0% frame loss
//! - **Latency** (Section 26.2): Round-trip time at various load levels
//! - **Frame Loss** (Section 26.3): Frame loss ratio vs. offered load
//! - **Back-to-Back** (Section 26.4): Maximum burst capacity
//!
//! Extended tests (from Go and Python implementations):
//! - Y.1564 EtherSAM, RFC 2889 LAN Switch, RFC 6349 TCP, Y.1731 OAM, MEF, TSN

pub mod config;
pub mod throughput;
pub mod latency;
pub mod frameloss;
pub mod back2back;
pub mod packet;
pub mod results;
pub mod pacing;

// Explicit imports to avoid glob re-export conflicts
pub use config::RateType;
pub use results::{ThroughputResult, LatencyResult};
pub use throughput::{ThroughputTestResult, TrialResult};
pub use latency::LatencyTestResult;
pub use frameloss::{FrameLossResult, FrameLossTestResult, FrameLossTrial};
pub use back2back::{BackToBackResult, BackToBackTestResult};
pub use packet::*;
pub use results::*;
pub use pacing::Pacer;
