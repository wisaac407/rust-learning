/// Manual packet parsing structures for IP and TCP headers
///
/// This module provides structures and parsing logic for working with
/// raw IP and TCP packets. All fields are in host byte order after parsing.
use std::fmt;

// ============================================================================
// IP Header Parsing
// ============================================================================

/// IPv4 Header Structure (minimum 20 bytes)
///
/// Reference: RFC 791
/// ```
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |Version|  IHL  |Type of Service|          Total Length         |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |         Identification        |Flags|      Fragment Offset    |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |  Time to Live |    Protocol   |         Header Checksum       |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                       Source Address                          |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                    Destination Address                        |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
#[derive(Debug, Clone)]
pub struct IpHeader {
    pub version: u8,          // IP version (should be 4)
    pub ihl: u8,              // Internet Header Length (in 32-bit words, usually 5)
    pub header_len: u8,       // Header length in bytes (ihl * 4)
    pub tos: u8,              // Type of Service
    pub total_len: u16,       // Total length of packet (header + data)
    pub identification: u16,  // Fragment identification
    pub flags: u8,            // Flags (3 bits: Reserved, DF, MF)
    pub fragment_offset: u16, // Fragment offset (13 bits)
    pub ttl: u8,              // Time To Live
    pub protocol: u8,         // Protocol (6=TCP, 17=UDP, 1=ICMP)
    pub checksum: u16,        // Header checksum
    pub src_addr: [u8; 4],    // Source IP address
    pub dst_addr: [u8; 4],    // Destination IP address
}

impl IpHeader {
    /// Parse an IP header from raw bytes
    ///
    /// Returns an IpHeader struct with all fields in host byte order
    pub fn parse(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 20 {
            return Err("Packet too short for IP header");
        }

        // First byte contains version and IHL
        let version = data[0] >> 4;
        let ihl = data[0] & 0x0F;
        let header_len = ihl * 4;

        if version != 4 {
            return Err("Not IPv4");
        }

        if data.len() < header_len as usize {
            return Err("Packet shorter than indicated header length");
        }

        Ok(IpHeader {
            version,
            ihl,
            header_len,
            tos: data[1],
            total_len: u16::from_be_bytes([data[2], data[3]]),
            identification: u16::from_be_bytes([data[4], data[5]]),
            flags: (data[6] >> 5) & 0x07,
            fragment_offset: u16::from_be_bytes([data[6] & 0x1F, data[7]]),
            ttl: data[8],
            protocol: data[9],
            checksum: u16::from_be_bytes([data[10], data[11]]),
            src_addr: [data[12], data[13], data[14], data[15]],
            dst_addr: [data[16], data[17], data[18], data[19]],
        })
    }

    /// Build an IP header into a byte buffer
    ///
    /// Checksum will be calculated automatically
    pub fn build(&self, buffer: &mut [u8]) -> Result<usize, &'static str> {
        if buffer.len() < 20 {
            return Err("Buffer too small for IP header");
        }

        // Version and IHL
        buffer[0] = (self.version << 4) | self.ihl;
        buffer[1] = self.tos;

        // Total length
        buffer[2..4].copy_from_slice(&self.total_len.to_be_bytes());

        // Identification
        buffer[4..6].copy_from_slice(&self.identification.to_be_bytes());

        // Flags and fragment offset
        let flags_frag = ((self.flags as u16) << 13) | (self.fragment_offset & 0x1FFF);
        buffer[6..8].copy_from_slice(&flags_frag.to_be_bytes());

        // TTL and Protocol
        buffer[8] = self.ttl;
        buffer[9] = self.protocol;

        // Checksum (zero it first, then calculate)
        buffer[10] = 0;
        buffer[11] = 0;

        // Addresses
        buffer[12..16].copy_from_slice(&self.src_addr);
        buffer[16..20].copy_from_slice(&self.dst_addr);

        // Calculate checksum
        let checksum = calculate_ip_checksum(&buffer[..20]);
        buffer[10..12].copy_from_slice(&checksum.to_be_bytes());

        Ok(20)
    }

    /// Validate the IP header checksum
    pub fn validate_checksum(&self, data: &[u8]) -> bool {
        calculate_ip_checksum(&data[..self.header_len as usize]) == 0
    }
}

/// Calculate IP header checksum
///
/// The checksum is calculated over the entire IP header.
/// When validating, the result should be 0 if the checksum is correct.
pub fn calculate_ip_checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;

    // Sum all 16-bit words
    for i in (0..data.len()).step_by(2) {
        let word = if i + 1 < data.len() {
            u16::from_be_bytes([data[i], data[i + 1]])
        } else {
            // Odd length, pad with zero
            u16::from_be_bytes([data[i], 0])
        };
        sum += word as u32;
    }

    // Fold 32-bit sum to 16 bits
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    // One's complement
    !sum as u16
}

// ============================================================================
// TCP Header Parsing
// ============================================================================

/// TCP Header Structure (minimum 20 bytes)
///
/// Reference: RFC 793
/// ```
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |          Source Port          |       Destination Port        |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                        Sequence Number                        |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                    Acknowledgment Number                      |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |  Data |           |U|A|P|R|S|F|                               |
/// | Offset| Reserved  |R|C|S|S|Y|I|            Window             |
/// |       |           |G|K|H|T|N|N|                               |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |           Checksum            |         Urgent Pointer        |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
#[derive(Debug, Clone)]
pub struct TcpHeader {
    pub src_port: u16,       // Source port
    pub dst_port: u16,       // Destination port
    pub seq_num: u32,        // Sequence number
    pub ack_num: u32,        // Acknowledgment number
    pub data_offset: u8,     // Header length in bytes (data_offset_raw * 4)
    pub data_offset_raw: u8, // Header length in 32-bit words (usually 5)
    pub flags: u8,           // Control flags (URG, ACK, PSH, RST, SYN, FIN)
    pub window: u16,         // Window size
    pub checksum: u16,       // Checksum
    pub urgent_ptr: u16,     // Urgent pointer
}

impl TcpHeader {
    /// Parse a TCP header from raw bytes
    ///
    /// Returns a TcpHeader struct with all fields in host byte order
    pub fn parse(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 20 {
            return Err("Packet too short for TCP header");
        }

        let data_offset_raw = data[12] >> 4;
        let data_offset = data_offset_raw * 4;

        if data.len() < data_offset as usize {
            return Err("Packet shorter than indicated TCP header length");
        }

        Ok(TcpHeader {
            src_port: u16::from_be_bytes([data[0], data[1]]),
            dst_port: u16::from_be_bytes([data[2], data[3]]),
            seq_num: u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
            ack_num: u32::from_be_bytes([data[8], data[9], data[10], data[11]]),
            data_offset_raw,
            data_offset,
            flags: data[13] & 0x3F, // Lower 6 bits are flags
            window: u16::from_be_bytes([data[14], data[15]]),
            checksum: u16::from_be_bytes([data[16], data[17]]),
            urgent_ptr: u16::from_be_bytes([data[18], data[19]]),
        })
    }

    /// Build a TCP header into a byte buffer
    ///
    /// Note: This does NOT calculate the checksum (requires IP pseudo-header)
    /// Call calculate_tcp_checksum() separately
    pub fn build(&self, buffer: &mut [u8]) -> Result<usize, &'static str> {
        if buffer.len() < 20 {
            return Err("Buffer too small for TCP header");
        }

        // Ports
        buffer[0..2].copy_from_slice(&self.src_port.to_be_bytes());
        buffer[2..4].copy_from_slice(&self.dst_port.to_be_bytes());

        // Sequence and ACK numbers
        buffer[4..8].copy_from_slice(&self.seq_num.to_be_bytes());
        buffer[8..12].copy_from_slice(&self.ack_num.to_be_bytes());

        // Data offset and reserved bits
        buffer[12] = (self.data_offset_raw << 4) | 0x00;

        // Flags
        buffer[13] = self.flags & 0x3F;

        // Window
        buffer[14..16].copy_from_slice(&self.window.to_be_bytes());

        // Checksum (caller must fill this in)
        buffer[16..18].copy_from_slice(&self.checksum.to_be_bytes());

        // Urgent pointer
        buffer[18..20].copy_from_slice(&self.urgent_ptr.to_be_bytes());

        Ok(self.data_offset as usize)
    }

    // Convenience methods for checking flags
    pub fn syn(&self) -> bool {
        self.flags & 0x02 != 0
    }
    pub fn ack(&self) -> bool {
        self.flags & 0x10 != 0
    }
    pub fn fin(&self) -> bool {
        self.flags & 0x01 != 0
    }
    pub fn rst(&self) -> bool {
        self.flags & 0x04 != 0
    }
    pub fn psh(&self) -> bool {
        self.flags & 0x08 != 0
    }
    pub fn urg(&self) -> bool {
        self.flags & 0x20 != 0
    }
}

/// Calculate TCP checksum
///
/// TCP checksum includes:
/// 1. IP pseudo-header (source IP, dest IP, protocol, TCP length)
/// 2. TCP header
/// 3. TCP data
///
/// The checksum field in the TCP header should be zero when calculating.
pub fn calculate_tcp_checksum(src_ip: &[u8; 4], dst_ip: &[u8; 4], tcp_segment: &[u8]) -> u16 {
    let mut sum: u32 = 0;

    // Pseudo-header
    // Source IP (4 bytes)
    sum += u16::from_be_bytes([src_ip[0], src_ip[1]]) as u32;
    sum += u16::from_be_bytes([src_ip[2], src_ip[3]]) as u32;

    // Destination IP (4 bytes)
    sum += u16::from_be_bytes([dst_ip[0], dst_ip[1]]) as u32;
    sum += u16::from_be_bytes([dst_ip[2], dst_ip[3]]) as u32;

    // Zero + Protocol (1 byte zero, 1 byte protocol=6 for TCP)
    sum += 6u32; // Protocol 6 = TCP

    // TCP length (2 bytes)
    sum += tcp_segment.len() as u32;

    // TCP segment (header + data)
    for i in (0..tcp_segment.len()).step_by(2) {
        let word = if i + 1 < tcp_segment.len() {
            u16::from_be_bytes([tcp_segment[i], tcp_segment[i + 1]])
        } else {
            // Odd length, pad with zero
            u16::from_be_bytes([tcp_segment[i], 0])
        };
        sum += word as u32;
    }

    // Fold 32-bit sum to 16 bits
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    // One's complement
    !sum as u16
}

// ============================================================================
// TCP Flags Constants
// ============================================================================

pub const TCP_FIN: u8 = 0x01;
pub const TCP_SYN: u8 = 0x02;
pub const TCP_RST: u8 = 0x04;
pub const TCP_PSH: u8 = 0x08;
pub const TCP_ACK: u8 = 0x10;
pub const TCP_URG: u8 = 0x20;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_checksum() {
        // Simple IP header with known checksum
        let header = [
            0x45, 0x00, 0x00, 0x3c, // Version, IHL, TOS, Total Length
            0x1c, 0x46, 0x40, 0x00, // ID, Flags, Fragment Offset
            0x40, 0x06, 0x00, 0x00, // TTL, Protocol, Checksum (zeroed)
            0xac, 0x10, 0x0a, 0x63, // Source IP: 172.16.10.99
            0xac, 0x10, 0x0a, 0x0c, // Dest IP: 172.16.10.12
        ];

        let checksum = calculate_ip_checksum(&header);
        assert_ne!(checksum, 0, "Checksum should not be zero");
    }

    #[test]
    fn test_ip_parse() {
        let packet = [
            0x45, 0x00, 0x00, 0x3c, // Version=4, IHL=5, TOS=0, Len=60
            0x1c, 0x46, 0x40, 0x00, // ID, Flags, Offset
            0x40, 0x06, 0xb1, 0xe6, // TTL=64, Proto=6 (TCP), Checksum
            0xac, 0x10, 0x0a, 0x63, // Src: 172.16.10.99
            0xac, 0x10, 0x0a, 0x0c, // Dst: 172.16.10.12
        ];

        let ip = IpHeader::parse(&packet).unwrap();
        assert_eq!(ip.version, 4);
        assert_eq!(ip.protocol, 6); // TCP
        assert_eq!(ip.src_addr, [172, 16, 10, 99]);
        assert_eq!(ip.dst_addr, [172, 16, 10, 12]);
    }

    #[test]
    fn test_tcp_parse() {
        let tcp_data = [
            0x04, 0xd2, 0x00, 0x50, // Src port=1234, Dst port=80
            0x00, 0x00, 0x00, 0x01, // Seq=1
            0x00, 0x00, 0x00, 0x00, // Ack=0
            0x50, 0x02, 0x20, 0x00, // Offset=5, Flags=SYN, Window=8192
            0x00, 0x00, 0x00, 0x00, // Checksum, Urgent
        ];

        let tcp = TcpHeader::parse(&tcp_data).unwrap();
        assert_eq!(tcp.src_port, 1234);
        assert_eq!(tcp.dst_port, 80);
        assert_eq!(tcp.seq_num, 1);
        assert!(tcp.syn());
        assert!(!tcp.ack());
    }
}
