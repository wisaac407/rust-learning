use std::io::Read;
use tun::Device;

mod packet;
use packet::{IpHeader, TcpHeader};

fn main() {
    println!("=== TCP Stack Implementation ===\n");

    // Create TUN device with configuration
    println!("Creating TUN device...");

    let mut config = tun::Configuration::default();
    config
        .name("utun9") // macOS uses utun interfaces
        .address((10, 0, 0, 1))
        .destination((10, 0, 0, 1))
        .netmask((255, 255, 255, 0))
        .up();

    // Note: packet_information is handled automatically by the crate

    let mut iface = match tun::create(&config) {
        Ok(iface) => {
            println!("✓ TUN device created successfully");
            if let Ok(name) = iface.name() {
                println!("  Interface name: {}", name);
            }
            iface
        }
        Err(e) => {
            eprintln!("✗ Failed to create TUN device: {}", e);
            eprintln!("\nNote: You may need to run with sudo:");
            eprintln!("  sudo cargo run");
            eprintln!("\nOn macOS, make sure you have the necessary permissions.");
            return;
        }
    };

    // The interface should now be configured automatically
    println!("\n✓ Network interface configured:");
    println!("  Address: 10.0.0.1/24");
    if let Ok(name) = iface.name() {
        println!("  Interface: {}", name);
    }

    println!("\n=== Listening for packets ===");
    println!("Try connecting to 10.0.0.1:8080 from another terminal:");
    println!("  nc 10.0.0.1 8080");
    println!("  telnet 10.0.0.1 8080");
    println!("  ping 10.0.0.1");
    println!("\nOr run tcpdump to see traffic:");
    println!("  sudo tcpdump -i tun0 -n -v\n");

    // Main packet processing loop
    let mut buffer = [0u8; 1500]; // MTU is typically 1500 bytes

    loop {
        match iface.read(&mut buffer) {
            Ok(nbytes) => {
                println!("\n>>> Received {} bytes", nbytes);

                // On macOS with packet_information, skip first 4 bytes
                let packet_start = if cfg!(target_os = "macos") { 4 } else { 0 };

                // Parse IP header
                match IpHeader::parse(&buffer[packet_start..nbytes]) {
                    Ok(ip_header) => {
                        print_ip_header(&ip_header);

                        // Check if it's TCP
                        if ip_header.protocol == 6 {
                            // Parse TCP header
                            let tcp_payload_start = ip_header.header_len as usize;
                            let tcp_payload = &buffer[tcp_payload_start..nbytes];

                            match TcpHeader::parse(tcp_payload) {
                                Ok(tcp_header) => {
                                    print_tcp_header(&tcp_header);

                                    // TODO: Implement TCP state machine and response logic here
                                    println!("\n[Your TCP implementation will handle this packet]");
                                }
                                Err(e) => {
                                    eprintln!("Failed to parse TCP header: {}", e);
                                }
                            }
                        } else {
                            println!("Non-TCP packet (protocol: {})", ip_header.protocol);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse IP header: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from TUN device: {}", e);
            }
        }
    }
}

fn print_ip_header(header: &IpHeader) {
    println!("  IP Header:");
    println!("    Version: {}", header.version);
    println!("    Header Length: {} bytes", header.header_len);
    println!("    Total Length: {} bytes", header.total_len);
    println!(
        "    Protocol: {} ({})",
        header.protocol,
        match header.protocol {
            1 => "ICMP",
            6 => "TCP",
            17 => "UDP",
            _ => "Other",
        }
    );
    println!(
        "    Source: {}.{}.{}.{}",
        header.src_addr[0], header.src_addr[1], header.src_addr[2], header.src_addr[3]
    );
    println!(
        "    Destination: {}.{}.{}.{}",
        header.dst_addr[0], header.dst_addr[1], header.dst_addr[2], header.dst_addr[3]
    );
}

fn print_tcp_header(header: &TcpHeader) {
    println!("  TCP Header:");
    println!("    Source Port: {}", header.src_port);
    println!("    Destination Port: {}", header.dst_port);
    println!("    Sequence Number: {}", header.seq_num);
    println!("    Acknowledgment Number: {}", header.ack_num);
    println!("    Data Offset: {} bytes", header.data_offset);
    println!(
        "    Flags: {}{}{}{}{}{}",
        if header.flags & 0x20 != 0 { "URG " } else { "" },
        if header.flags & 0x10 != 0 { "ACK " } else { "" },
        if header.flags & 0x08 != 0 { "PSH " } else { "" },
        if header.flags & 0x04 != 0 { "RST " } else { "" },
        if header.flags & 0x02 != 0 { "SYN " } else { "" },
        if header.flags & 0x01 != 0 { "FIN " } else { "" },
    );
    println!("    Flags Hex: {:#x}", header.flags);
    println!("    Window: {}", header.window);
    println!("    Checksum: 0x{:04x}", header.checksum);
}
