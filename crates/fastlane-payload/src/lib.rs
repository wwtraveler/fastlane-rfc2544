//! fastlane-payload — Packet payload generation and frame construction
//!
//! Provides payload pattern generation (sequential, repeating, random),
//! RFC2544 signature validation, and frame construction for high-speed
//! packet benchmarking on 400G+ interfaces.

/// Payload pattern types supported by fastlane
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadPattern {
    Sequential,
    Repeating,
    Random,
}

impl std::fmt::Display for PayloadPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PayloadPattern::Sequential => write!(f, "sequential"),
            PayloadPattern::Repeating => write!(f, "repeating"),
            PayloadPattern::Random => write!(f, "random"),
        }
    }
}

/// Generate a payload buffer for a given frame size and pattern
pub fn generate_payload(frame_size: u32, pattern: PayloadPattern) -> Vec<u8> {
    let payload_len = (frame_size as usize) - 42; // Subtract header bytes
    let mut buf = vec![0u8; payload_len];
    match pattern {
        PayloadPattern::Sequential => {
            for (i, byte) in buf.iter_mut().enumerate() {
                *byte = (i % 256) as u8;
            }
        }
        PayloadPattern::Repeating => {
            let pattern_data = [0xAA, 0x55, 0xFF, 0x00, 0x12, 0x34, 0x56, 0x78];
            for (i, byte) in buf.iter_mut().enumerate() {
                *byte = pattern_data[i % pattern_data.len()];
            }
        }
        PayloadPattern::Random => {
            let mut seed = 0x1234u32;
            for byte in buf.iter_mut() {
                seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                *byte = ((seed >> 16) & 0xFF) as u8;
            }
        }
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_payload() {
        let buf = generate_payload(128, PayloadPattern::Sequential);
        assert_eq!(buf.len(), 86); // 128 - 42
        for (i, byte) in buf.iter().enumerate() {
            assert_eq!(*byte, (i % 256) as u8);
        }
    }

    #[test]
    fn test_repeating_payload() {
        let buf = generate_payload(64, PayloadPattern::Repeating);
        assert_eq!(buf.len(), 22);
        let pattern_data = [0xAA, 0x55, 0xFF, 0x00, 0x12, 0x34, 0x56, 0x78];
        for (i, byte) in buf.iter().enumerate() {
            assert_eq!(*byte, pattern_data[i % pattern_data.len()]);
        }
    }

    #[test]
    fn test_random_payload() {
        let buf = generate_payload(256, PayloadPattern::Random);
        assert_eq!(buf.len(), 214);
    }

    #[test]
    fn test_jumbo_frame_payload() {
        let buf = generate_payload(9000, PayloadPattern::Sequential);
        assert_eq!(buf.len(), 8958); // 9000 - 42
    }
}
