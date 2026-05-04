# TCP Stack Implementation Guide

## Project Overview

This project implements a TCP/IP stack from scratch in Rust. The goal is to deeply understand how TCP works by building it yourself, including:

- Manual parsing of IP and TCP headers
- TCP state machine implementation
- Connection establishment (3-way handshake)
- Reliable data transfer
- Connection teardown

## Architecture

```
┌─────────────────────────────────┐
│   Test Application              │
│   (connect to 10.0.0.1:8080)   │
└────────────┬────────────────────┘
             │ (TCP/IP packets)
             ▼
┌─────────────────────────────────┐
│   Kernel Network Stack          │
│   (routes to TUN device)        │
└────────────┬────────────────────┘
             │ (IP packets)
             ▼
┌─────────────────────────────────┐
│   TUN Device (tun0)             │
│   IP: 10.0.0.1/24               │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│   Your TCP Implementation       │
│   • Read IP packets from TUN    │
│   • Parse IP/TCP headers        │
│   • Process TCP logic           │
│   • Write responses to TUN      │
└─────────────────────────────────┘
```

## How TUN Devices Work

### What is a TUN Device?

A TUN (network TUNnel) device is a **virtual network interface** that operates at **Layer 3 (IP layer)**:

- When you read from a TUN device, you get complete IP packets
- When you write to a TUN device, you send complete IP packets
- The kernel routes IP traffic to/from the TUN device like any other interface
- Your program is the "other end" of the tunnel

### TUN vs TAP

- **TUN (Layer 3)**: Operates with IP packets
  - You receive: `[IP Header][TCP Header][Data]`
  - Simpler, focused on TCP/IP implementation
- **TAP (Layer 2)**: Operates with Ethernet frames
  - You receive: `[Ethernet Header][IP Header][TCP Header][Data]`
  - More complex, but teaches more about networking

For TCP learning, **TUN is ideal**.

## Setting Up TUN Device for Localhost Testing

### Creating the TUN Device

The starter code creates a TUN device programmatically with these steps:

1. **Open TUN device** - Request kernel to create a virtual interface
2. **Assign IP address** - Configure it with `10.0.0.1/24`
3. **Bring interface up** - Activate the interface
4. **Configure routing** - Traffic to `10.0.0.1` flows through your TUN device

### Network Configuration

```
TUN Interface: tun0
IP Address: 10.0.0.1
Netmask: 255.255.255.0 (/24)
Network: 10.0.0.0/24
```

### How Localhost Testing Works

1. **Your TCP stack runs** and creates `tun0` with IP `10.0.0.1`
2. **Test client connects** to `10.0.0.1:8080`
3. **Kernel routes the packet** to your TUN device (since it owns that IP)
4. **Your program reads the packet** from TUN device
5. **Your TCP stack processes it** and writes response back to TUN
6. **Kernel delivers response** to the test client

This is all **localhost** - no real network hardware involved!

## IP Packet Structure

### IPv4 Header (20 bytes minimum)

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|Version|  IHL  |Type of Service|          Total Length         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|         Identification        |Flags|      Fragment Offset    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|  Time to Live |    Protocol   |         Header Checksum       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       Source Address                          |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    Destination Address                        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    Options (if IHL > 5)                       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

### Key Fields

- **Version**: 4 (for IPv4)
- **IHL**: Internet Header Length (number of 32-bit words in header, usually 5)
- **Total Length**: Length of entire packet (header + data)
- **Protocol**: 6 for TCP, 17 for UDP, 1 for ICMP
- **Source/Dest Address**: 32-bit IP addresses

## TCP Segment Structure

### TCP Header (20 bytes minimum)

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|          Source Port          |       Destination Port        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        Sequence Number                        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    Acknowledgment Number                      |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|  Data |           |U|A|P|R|S|F|                               |
| Offset| Reserved  |R|C|S|S|Y|I|            Window             |
|       |           |G|K|H|T|N|N|                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|           Checksum            |         Urgent Pointer        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    Options (if Data Offset > 5)               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                             data                              |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

### Key Fields

- **Source/Dest Port**: 16-bit port numbers
- **Sequence Number**: Position of data in stream (or ISN for SYN)
- **Acknowledgment Number**: Next expected byte (if ACK flag set)
- **Data Offset**: TCP header length in 32-bit words (usually 5)
- **Flags**:
  - **SYN**: Synchronize, initiates connection
  - **ACK**: Acknowledgment field is valid
  - **FIN**: Finish, closes connection
  - **RST**: Reset connection
  - **PSH**: Push data immediately
  - **URG**: Urgent pointer is valid
- **Window**: Flow control, bytes sender can receive
- **Checksum**: Error detection for header + data

## TCP State Machine

Your TCP implementation will need a state machine for each connection:

```
                              +---------+
                              |  CLOSED |
                              +---------+
                                   |
                         passive open|  active open
                          (listen()) |  (connect())
                                   |
                              +---------+
                     +--------|  LISTEN |
                     |        +---------+
         rcv SYN     |             |
         snd SYN,ACK |             | snd SYN
                     |             |
                +----------+   +----------+
                |SYN_RCVD  |   |SYN_SENT  |
                +----------+   +----------+
                     |             |
         rcv ACK     |             | rcv SYN,ACK
                     |             | snd ACK
                     |             |
                     |    +-----------+
                     +--->|ESTABLISHED|<----+
                          +-----------+     |
                                |           |
                     close()    |           |
                     snd FIN    |           |
                                |           |
                          +----------+      |
                          |FIN_WAIT_1|      |
                          +----------+      |
                                |           |
                 rcv ACK of FIN |           |
                                |           |
                          +----------+      |
                          |FIN_WAIT_2|      |
                          +----------+      |
                                |           |
                      rcv FIN   |           |
                      snd ACK   |           |
                                |           |
                          +---------+       |
                          |TIME_WAIT|       |
                          +---------+       |
                                |           |
                       2MSL timer |         |
                                |           |
                          +---------+       |
                          | CLOSED  |<------+
                          +---------+
```

### Core States

1. **CLOSED**: No connection
2. **LISTEN**: Server waiting for SYN
3. **SYN_SENT**: Client sent SYN, waiting for SYN+ACK
4. **SYN_RECEIVED**: Server received SYN, sent SYN+ACK, waiting for ACK
5. **ESTABLISHED**: Connection open, data transfer
6. **FIN_WAIT_1**: Sent FIN, waiting for ACK
7. **FIN_WAIT_2**: Received ACK of FIN, waiting for peer's FIN
8. **TIME_WAIT**: Waiting to ensure remote received our ACK
9. **CLOSE_WAIT**: Received FIN, waiting for local close
10. **LAST_ACK**: Sent FIN after receiving FIN, waiting for ACK

## TCP Connection: 3-Way Handshake

### Connection Establishment

```
Client                                Server
  |                                     |
  |  SYN (seq=x)                        |
  |------------------------------------>|
  |                                     | [LISTEN -> SYN_RECEIVED]
  |           SYN+ACK (seq=y, ack=x+1) |
  |<------------------------------------|
  |                                     |
  |  ACK (seq=x+1, ack=y+1)            |
  |------------------------------------>|
  |                                     | [SYN_RECEIVED -> ESTABLISHED]
  |                                     |
  |         [ESTABLISHED]               |
```

### Step-by-Step

1. **Client → Server: SYN**
   - Client picks random Initial Sequence Number (ISN) `x`
   - Sends TCP segment with SYN flag, seq=`x`
   - Client enters SYN_SENT state

2. **Server → Client: SYN+ACK**
   - Server picks its own ISN `y`
   - Sends TCP segment with SYN+ACK flags, seq=`y`, ack=`x+1`
   - Server enters SYN_RECEIVED state

3. **Client → Server: ACK**
   - Client sends ACK, seq=`x+1`, ack=`y+1`
   - Client enters ESTABLISHED state
   - Server receives ACK, enters ESTABLISHED state

## Implementation Phases

### Phase 1: TUN Device Setup (✓ Starter Code Provided)

- [x] Create TUN device
- [x] Configure IP address
- [x] Read/write raw packets
- [x] Basic IP header parsing structures

### Phase 2: IP Layer Parsing

- [ ] Parse IP header completely
- [ ] Validate IP checksum
- [ ] Extract TCP payload
- [ ] Build IP header for responses

### Phase 3: TCP Header Parsing

- [ ] Parse TCP header fields
- [ ] Validate TCP checksum (includes IP pseudo-header!)
- [ ] Extract TCP flags
- [ ] Extract TCP options (MSS, window scale, etc.)

### Phase 4: Connection Management

- [ ] Implement TCP state machine
- [ ] Handle SYN (LISTEN → SYN_RECEIVED)
- [ ] Handle SYN+ACK (SYN_SENT → ESTABLISHED)
- [ ] Handle ACK (SYN_RECEIVED → ESTABLISHED)
- [ ] Track connections (4-tuple: src_ip, src_port, dst_ip, dst_port)

### Phase 5: Basic Data Transfer

- [ ] Receive data in ESTABLISHED state
- [ ] Send ACKs for received data
- [ ] Send data to application
- [ ] Handle receive buffer

### Phase 6: Connection Teardown

- [ ] Handle FIN (ESTABLISHED → CLOSE_WAIT)
- [ ] Send FIN on close (ESTABLISHED → FIN_WAIT_1)
- [ ] Complete 4-way handshake
- [ ] TIME_WAIT state (2MSL timer)

### Phase 7: Reliability (Advanced)

- [ ] Retransmission on timeout
- [ ] Duplicate ACK detection
- [ ] Out-of-order segment buffering
- [ ] Sliding window management

### Phase 8: Flow Control (Advanced)

- [ ] Window size management
- [ ] Persist timer
- [ ] Window scaling option

### Phase 9: Congestion Control (Advanced)

- [ ] Slow start
- [ ] Congestion avoidance
- [ ] Fast retransmit / fast recovery

## Testing Your Implementation

### Step 1: Run Your TCP Stack

```bash
cd tcp
sudo cargo run
```

You'll see output indicating the TUN device was created and your stack is running.

### Step 2: Verify TUN Interface

In another terminal:

```bash
# View network interfaces
ifconfig tun0

# Should show:
# tun0: flags=...
#       inet 10.0.0.1 netmask 0xffffff00
```

### Step 3: Monitor Traffic

```bash
# Watch packets on TUN device
sudo tcpdump -i tun0 -n -v

# Or with more detail
sudo tcpdump -i tun0 -n -X
```

### Step 4: Send Test Traffic

```bash
# Try to connect (will fail initially, but sends SYN)
nc 10.0.0.1 8080

# Or use telnet
telnet 10.0.0.1 8080

# Send ping (ICMP, not TCP, but useful for debugging)
ping 10.0.0.1
```

### Step 5: Build a Test Client

Create a simple Rust client:

```rust
use std::net::TcpStream;
use std::io::{Write, Read};

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("10.0.0.1:8080")?;
    stream.write_all(b"Hello from client")?;

    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer)?;
    println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));

    Ok(())
}
```

**Note**: This uses the OS's TCP stack, so it will attempt a proper 3-way handshake. Your stack must respond correctly!

## Debugging Tips

### Print Incoming Packets

```rust
println!("=== Received Packet ===");
println!("IP: {} -> {}", ip_header.src_addr, ip_header.dst_addr);
println!("TCP: {} -> {}", tcp_header.src_port, tcp_header.dst_port);
println!("Flags: SYN={} ACK={} FIN={}",
    tcp_header.syn(), tcp_header.ack(), tcp_header.fin());
println!("Seq: {}, Ack: {}", tcp_header.seq, tcp_header.ack_num);
```

### Common Issues

1. **No packets arriving**
   - Check TUN device is up: `ifconfig tun0`
   - Check routing: `ip route get 10.0.0.1`
   - May need to run with `sudo`

2. **Checksum errors**
   - TCP checksum includes IP pseudo-header
   - Must use network byte order (big-endian)
   - Don't forget to zero checksum field before calculating

3. **Connection hangs**
   - Check state transitions
   - Verify sequence/ack numbers are correct
   - Use `tcpdump` to see what kernel expects vs. what you send

4. **Packets not reaching application**
   - Kernel may be sending RST packets
   - Check firewall rules: `sudo pfctl -s rules` (macOS) or `iptables -L` (Linux)
   - You may need to disable kernel's TCP stack for your test IP

### Useful Commands

```bash
# View all network interfaces
ifconfig

# View routing table
netstat -rn      # macOS
ip route         # Linux

# View active TCP connections
netstat -an | grep ESTABLISHED

# Capture packets on TUN device
sudo tcpdump -i tun0 -w capture.pcap
# Analyze in Wireshark later

# Check if port is listening
lsof -i :8080
```

## Checksum Calculation

### IP Checksum

The IP checksum is calculated over the IP header only:

```rust
fn calculate_ip_checksum(header: &[u8]) -> u16 {
    let mut sum: u32 = 0;

    // Sum 16-bit words
    for i in (0..header.len()).step_by(2) {
        let word = u16::from_be_bytes([header[i], header[i + 1]]);
        sum += word as u32;
    }

    // Fold 32-bit sum to 16 bits
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    // One's complement
    !sum as u16
}
```

### TCP Checksum

TCP checksum is more complex - it includes a **pseudo-header** from the IP layer:

```
Pseudo-Header:
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       Source Address                          |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    Destination Address                        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|     Zero      |    Protocol   |         TCP Length            |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

Then checksum over: `[pseudo-header][TCP header][TCP data]`

## Resources

### RFCs (Specifications)

- **RFC 791**: Internet Protocol (IP)
- **RFC 793**: Transmission Control Protocol (TCP)
- **RFC 1122**: Requirements for Internet Hosts
- **RFC 5681**: TCP Congestion Control
- **RFC 6298**: Computing TCP's Retransmission Timer

### Learning Resources

- [TCP/IP Illustrated, Volume 1](https://en.wikipedia.org/wiki/TCP/IP_Illustrated) by W. Richard Stevens
- [Beej's Guide to Network Programming](https://beej.us/guide/bgnet/)
- [Linux kernel TCP implementation](https://github.com/torvalds/linux/tree/master/net/ipv4) (reference)

### Tools

- **Wireshark**: Packet analyzer with excellent TCP analysis
- **tcpdump**: Command-line packet capture
- **netcat (nc)**: Simple TCP/UDP client/server
- **socat**: Advanced network tool

## Next Steps

1. **Run the starter code** - See packets arriving from kernel
2. **Implement IP parsing** - Extract source/dest, protocol, payload
3. **Implement TCP parsing** - Extract ports, flags, seq/ack numbers
4. **Build TCP state machine** - Track connection states
5. **Handle SYN** - Respond to connection attempts
6. **Complete handshake** - Establish connections
7. **Data transfer** - Send/receive actual data
8. **Connection teardown** - Clean close with FIN

Remember: **Start simple!** Even just parsing and printing packets is a great first milestone.

Good luck building your TCP stack! 🚀
