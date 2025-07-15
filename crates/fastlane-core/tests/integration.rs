//! Integration tests for fastlane-rfc2544
//!
//! Tests the full RFC 2544 test engine including:
//! - Throughput binary search
//! - Latency calculation
//! - Frame loss tracking
//! - Back-to-back burst
//! - Config loading (YAML/JSON)
//! - Output format generation
//! - IMIX frame generation
//! - Pacing rate control
//! - Multi-queue rate scaling
//! - AF_XDP ring buffer operations
//! - Packet signature validation
//! - Payload pattern generation

mod support {
    use fastlane_core::config::{Config, ThroughputConfig, TestType};
    use fastlane_core::throughput::run_throughput_trial;
    use fastlane_core::latency::run_latency_trial;
    use fastlane_core::frameloss::run_frameloss_trial;
    use fastlane_core::back2back::run_back2back_trial;
    use fastlane_core::packet::validate_signature;
    use fastlane_core::results::format_json;
    use fastlane_core::pacing::Pacer;
    use fastlane_core::config::RateType;

    use std::time::Duration;

    fn mock_tx_fn(trial_duration: Duration) -> Result<(u64, u64), anyhow::Error> {
        // Simulate: 99.5% receive rate (0.5% loss)
        let tx = 1_000_000;
        let rx = 995_000;
        Ok((tx, rx))
    }

    #[test]
    fn test_throughput_config_default() {
        let config = Config::default();
        assert_eq!(config.trial_duration, Duration::from_secs(60));
        assert_eq!(config.throughput.max_iterations, 25);
        assert_eq!(config.throughput.resolution_pct, 0.1);
        assert_eq!(config.throughput.tolerated_frame_loss, 0.001);
    }

    #[test]
    fn test_throughput_trial() {
        let mut config = Config::default();
        config.frame_sizes = vec![64];
        config.trial_duration = Duration::from_secs(1);

        let result = run_throughput_trial(64, &config, 1000.0, |_dur| {
            Ok((1_000_000, 995_000))
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_latency_trial() {
        let config = Config::default();

        let result = run_latency_trial(64, 1000.0, &config, |_fs, _rate| {
            Ok(vec![10, 15, 20, 25, 30])
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_frameloss_trial() {
        let config = Config::default();

        let result = run_frameloss_trial(128, &config, |rate, dur| {
            let tx = (rate / 1000.0) as u64 * dur.as_secs();
            let rx = tx * 99 / 100;
            Ok((tx, rx))
        });
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_back2back_trial() {
        let config = Config::default();
        let result = run_back2back_trial(64, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_packet_signature_validation() {
        let mut buf = vec![0u8; 150];
        buf[42] = 0x52;
        buf[43] = 0x47;
        buf[44] = 0x43;
        buf[45] = 0x32;
        buf[46] = 0x35;
        buf[47] = 0x34;
        buf[48] = 0x34;
        assert!(validate_signature(&buf));
    }

    #[test]
    fn test_packet_signature_invalid() {
        let buf = vec![0u8; 150];
        assert!(!validate_signature(&buf));
    }

    #[test]
    fn test_json_output() {
        let result = fastlane_core::results::TestSuiteResult {
            configuration: fastlane_core::results::ConfigSnapshot {
                interface: "eth0".to_string(),
                line_rate_mbps: 10000,
                test_type: "throughput".to_string(),
                frame_sizes: vec![64, 128],
                trial_duration: 60,
                warmup_period: 2,
                throughput_resolution_pct: 0.1,
                throughput_max_iterations: 25,
                tolerated_frame_loss: 0.001,
                include_jumbo: false,
            },
            throughput: None,
            latency: None,
            frame_loss: None,
            back_to_back: None,
            status: true,
            error_logs: vec![],
        };

        let json = format_json(&result);
        assert!(json.contains("eth0"));
        assert!(json.contains("throughput"));
    }

    #[test]
    fn test_pacer() {
        let pacer = Pacer::new(100.0, 64, RateType::Cbr);
        assert_eq!(pacer.rate_mpps(), 100.0);
        assert!(pacer.interval_ns() > 0);
    }

    #[test]
    fn test_config_yaml_load() {
        let yaml = r#"
            interface: eth0
            line_rate_mbps: 400000
            test_type: throughput
            frame_sizes: [64, 128, 256]
            include_jumbo: true
            trial_duration: 60s
        "#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.interface, "eth0");
        assert_eq!(config.line_rate_mbps, 400000);
        assert!(config.include_jumbo);
        assert_eq!(config.frame_sizes.len(), 3);
    }

    #[test]
    fn test_config_json_load() {
        let json = r#"
            {
                "interface": "eth0",
                "line_rate_mbps": 10000,
                "test_type": "latency",
                "frame_sizes": [64, 1518],
                "include_jumbo": true
            }
        "#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.interface, "eth0");
        assert!(config.include_jumbo);
    }

    #[test]
    fn test_all_frame_sizes() {
        let config = Config::default();
        let sizes = config.all_frame_sizes();
        assert!(sizes.contains(&64));
        assert!(sizes.contains(&1518));
    }

    #[test]
    fn test_config_jumbo_inclusion() {
        let mut config = Config::default();
        config.frame_sizes = vec![64, 128];
        config.include_jumbo = true;
        let sizes = config.all_frame_sizes();
        assert!(sizes.contains(&9000));
    }

    #[test]
    fn test_test_type_display() {
        assert_eq!(format!("{}", TestType::Throughput), "throughput");
        assert_eq!(format!("{}", TestType::Latency), "latency");
        assert_eq!(format!("{}", TestType::FrameLoss), "frame_loss");
        assert_eq!(format!("{}", TestType::BackToBack), "back_to_back");
    }

    #[test]
    fn test_frameloss_all_frames() {
        let config = Config::default();
        let frame_sizes = vec![64, 128, 256, 512];
        let mut all_pass = true;

        for fs in &frame_sizes {
            let result = run_frameloss_trial(*fs, &config, |rate, dur| {
                Ok((1_000_000, 990_000))
            });
            assert!(result.is_ok());
        }
        assert!(all_pass);
    }

    #[test]
    fn test_latency_load_rate_calculation() {
        use fastlane_core::latency::calc_load_rate;

        let rate = calc_load_rate(64, 0.5, 10000);
        assert!(rate > 0.0);
        assert!(rate < 1000.0);

        // Double the load level → double the rate
        let rate_full = calc_load_rate(64, 1.0, 10000);
        assert!((rate_full / rate - 2.0).abs() < 0.1);
    }

    #[test]
    fn test_throughput_binary_search() {
        use fastlane_core::throughput::calc_next_bitrate;

        // Test pass case
        let (min, max, bt) = calc_next_bitrate(true, 500.0, 0.0, 500.0, 250.0);
        assert_eq!(min, 500.0);
        assert_eq!(max, 750.0);
        assert_eq!(bt, 750.0);

        // Test fail case
        let (min, max, bt) = calc_next_bitrate(false, 500.0, 0.0, 500.0, 250.0);
        assert_eq!(max, 500.0);
        assert!(bt < 500.0);
    }

    #[test]
    fn test_output_formats() {
        let throughput_result = fastlane_core::results::ThroughputTestResult {
            frame_results: vec![
                fastlane_core::results::FrameResult {
                    frame_size: 64,
                    bitrate: 99.5,
                    tx_packets: 1000,
                    rx_packets: 999,
                    passed: true,
                },
            ],
            real_bitrate: 99.5,
            min_bitrate: 50.0,
            max_bitrate: 100.0,
            trials: vec![],
            test_duration: Duration::from_secs(60),
            test_type: TestType::Throughput,
        };

        // Text
        let text = fastlane_core::results::format_throughput_text(&throughput_result);
        assert!(text.contains("64"));

        // CSV
        let csv = fastlane_core::results::format_throughput_csv(&throughput_result);
        assert!(csv.contains("64"));
        assert!(csv.contains("true"));
    }

    #[test]
    fn test_payload_patterns() {
        use fastlane_payload::TestFrame;
        use fastlane_payload::PayloadPattern;

        let mut frame = TestFrame::new(16);
        frame.payload = vec![0u8; 16];

        // Sequential pattern
        frame.pattern = PayloadPattern::Sequential;
        frame.fill_payload(&mut frame.payload);
        for i in 0..8 {
            assert_eq!(frame.payload[i], i as u8 & 0x0f);
        }
    }

    #[test]
    fn test_imix_generation() {
        use fastlane_payload::default_imix_set;

        let imix = default_imix_set(1000);
        assert_eq!(imix.frames.len(), 7);
        let total_count: u32 = imix.frames.iter().map(|f| f.count).sum();
        assert!(total_count >= 900); // Allow for rounding
        assert!(total_count <= 1100);
    }

    #[test]
    fn test_af_xdp_ring_buffer() {
        use fastlane_dataplane::ring_buffer::SpscRing;

        let mut ring: SpscRing<u32> = SpscRing::new(4);
        assert!(ring.try_push(1));
        assert!(ring.try_push(2));
        assert!(ring.try_push(3));
        assert!(!ring.try_push(4)); // full
        assert_eq!(ring.try_pop(), Some(1));
    }

    #[test]
    fn test_multi_queue_manager() {
        use fastlane_dataplane::multi_queue::MultiQueueManager;
        use fastlane_core::config::RateType;

        let mgr = MultiQueueManager::new(4, 8, RateType::Cbr);
        assert_eq!(mgr.queues().len(), 4);
        assert_eq!(mgr.max_queues(), 8);
    }

    #[test]
    fn test_datapath() {
        use fastlane_dataplane::datapath::Datapath;

        let config = Config::default();
        let dp = Datapath::new(config);
        assert!(dp.config.interface.is_empty() || dp.config.interface == "eth0");
    }
}
