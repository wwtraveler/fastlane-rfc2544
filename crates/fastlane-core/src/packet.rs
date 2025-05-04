//! Packet generation and parsing for RFC 2544 test packets
//!
//! RFC 2544 test packets use a custom Ethernet/IPv4/UDP header with
/// a signature pattern for identification.

use std::mem;

/// Magic signature bytes for RFC 2544 test packets
/// Mirrors the 7-byte signature from the Go master implementation
pub const RFC2544_SIGNATURE: [u8; 7] = [0x52, 0x46, 0x43, 0x32, 0x35, 0x34, 0x34];

/// Packet layout per RFC 2544:
/// Offset  Size   Field
/// ------  ----   -----
/// 0       14     Ethernet header (dst MAC + src MAC + EtherType)
/// 14      20     IPv4 header
/// 34      8      UDP header
/// 42      7      Signature ("RFC2544")
/// 49      4      Sequence number
/// 53      8      TX timestamp (nanoseconds)
/// 61      4      Stream ID
/// 65      1      Flags
/// 66      N      Payload (RFC 2544 recommended pattern)
pub const RFC2544_PACKET_HEADER_SIZE: usize = 66;

/// Ethernet header: 6 bytes dst MAC + 6 bytes src MAC + 2 bytes EtherType
pub const ETHERNET_HEADER_SIZE: usize = 14;
/// IPv4 header: 20 bytes
pub const IPV4_HEADER_SIZE: usize = 20;
/// UDP header: 8 bytes (src port + dst port + length + checksum)
pub const UDP_HEADER_SIZE: usize = 8;

/// Packet signature structure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketSignature {
    /// The 7-byte magic signature
    pub magic: [u8; 7],
    /// Monotonically increasing sequence number
    pub seq: u32,
    /// TX timestamp in nanoseconds
    pub timestamp_ns: u64,
    /// Stream ID for multi-stream tests
    pub stream_id: u32,
    /// Flags bitfield
    pub flags: u8,
}

impl Default for PacketSignature {
    fn default() -> Self {
        Self {
            magic: RFC2544_SIGNATURE,
            seq: 0,
            timestamp_ns: 0,
            stream_id: 0,
            flags: 0,
        }
    }
}

/// Build the test packet signature
pub fn build_signature(seq: u32, timestamp_ns: u64, stream_id: u32) -> PacketSignature {
    PacketSignature {
        magic: RFC2544_SIGNATURE,
        seq,
        timestamp_ns,
        stream_id,
        flags: 0,
    }
}

/// Validate that a packet has the RFC 2544 signature
pub fn validate_signature(buf: &[u8]) -> bool {
    if buf.len() < RFC2544_PACKET_HEADER_SIZE + 7 {
        return false;
    }
    buf[ETHERNET_HEADER_SIZE + IPV4_HEADER_SIZE + UDP_HEADER_SIZE
        ..ETHERNET_HEADER_SIZE + IPV4_HEADER_SIZE + UDP_HEADER_SIZE + 7]
        == RFC2544_SIGNATURE
}

/// Calculate the RFC 2544 recommended payload pattern
/// Each byte is (index mod 16) & 0x0f
pub fn build_payload_pattern(buf: &mut [u8], frame_size: u32) {
    let udp_payload_len = (frame_size as usize) - 46;
    for i in 0..buf.len().min(udp_payload_len) {
        buf[i] = (i as u8) & 0x0f;
    }
}

/// Ethernet header for test packets
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EthHeader {
    pub dst_mac: [u8; 6],
    pub src_mac: [u8; 6],
    pub ether_type: u16, // 0x0800 = IPv4
}

/// UDP header for test packets
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UdpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub length: u16,
    pub checksum: u16,
}

/// Calculate UDP checksum for a packet
pub fn calc_udp_checksum(src_ip: u32, dst_ip: u32, udp_len: u16, payload: &[u8]) -> u16 {
    let mut sum: u32 = 0;

    // UDP pseudo-header
    sum += (src_ip & 0xffff) + ((src_ip >> 16) & 0xffff);
    sum += (dst_ip & 0xffff) + ((dst_ip >> 16) & 0xffff);
    sum += 0x11; // UDP protocol
    sum += udp_len as u32;

    // UDP header
    sum += 0x0042; // UDP src/dst ports (42)
    sum += udp_len as u32;

    // Payload
    let mut i = 0;
    while i < payload.len() {
        if i + 1 < payload.len() {
            sum += (payload[i] as u32) | ((payload[i + 1] as u32) << 8);
        } else {
            sum += payload[i] as u32;
        }
        i += 2;
    }

    // Fold carries
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }

    !sum as u16
}

/// Frame size to PPS conversion for 400G interface
/// At 400 Gbps with 64-byte frames:
/// PPS = 400e9 / (84 * 8) = ~595.2 Mpps
pub fn line_rate_pps_400g(frame_size: u32) -> f64 {
    let line_rate_bps = 400_000_000_000.0; // 400 Gbps
    let frame_bps = (frame_size + 20) as f64 * 8.0;
    line_rate_bps / frame_bps
}

/// Optimal batch size for 400G interface based on frame size
/// Larger frames = smaller batches (fewer, larger batches)
/// Smaller frames = larger batches (more, smaller batches)
pub fn optimal_batch_size(frame_size: u32) -> u32 {
    // Base: 64 bytes -> 128, 1518 bytes -> 64
    let base: u32 = 128;
    base * 64 / frame_size.clamp(64, 1518)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_validation() {
        let mut buf = vec![0u8; RFC2544_PACKET_HEADER_SIZE + 7];
        buf[ETHERNET_HEADER_SIZE + IPV4_HEADER_SIZE + UDP_HEADER_SIZE
            ..ETHERNET_HEADER_SIZE + IPV4_HEADER_SIZE + UDP_HEADER_SIZE + 7]
            .copy_from_slice(&RFC2544_SIGNATURE);
        assert!(validate_signature(&buf));
    }

    #[test]
    fn test_payload_pattern() {
        let mut buf = vec![0u8; 100];
        build_payload_pattern(&mut buf, 150);
        for i in 0..10 {
            assert_eq!(buf[i], (i as u8) & 0x0f);
        }
    }

    #[test]
    fn test_400g_line_rate_pps() {
        let pps = line_rate_pps_400g(64);
        // 400e9 / (84 * 8) = 400e9 / 672 ≈ 595.2e6
        assert!((pps - 595_238_095.0).abs() < 1000.0);
    }

    #[test]
    fn test_optimal_batch_size() {
        assert_eq!(optimal_batch_size(64), 128);
        assert_eq!(optimal_batch_size(128), 64);
        assert_eq!(optimal_batch_size(1518), 64);
        assert_eq!(optimal_batch_size(256), 32);
    }

    #[test]
    fn test_udp_checksum() {
        let payload = vec![0x01, 0x02, 0x03, 0x04];
        let checksum = calc_udp_checksum(0xC0A80102, 0xC0A80103, 12, &payload);
        assert_ne!(checksum, 0);
    }
}
