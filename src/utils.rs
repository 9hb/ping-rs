//
// DESCRIPTION: utility functions for the app
//
use std::net::{ IpAddr, ToSocketAddrs };

pub fn prelozit_adresu(hostname: &str) -> Result<IpAddr, Box<dyn std::error::Error>> {
    // Pokusit se parsovat jako IP adresu primo
    if let Ok(ip) = hostname.parse::<IpAddr>() {
        return Ok(ip);
    }

    // Jinak resolvovat hostname
    let adresa_s_portem = format!("{}:0", hostname);
    let mut adresy = adresa_s_portem.to_socket_addrs()?;

    match adresy.next() {
        Some(socket_addr) => Ok(socket_addr.ip()),
        None => Err(format!("✖ cannot resolve hostname: {}", hostname).into()),
    }
}

pub fn vytvorit_icmp_packet(velikost: usize, sekvencni_cislo: u16) -> Vec<u8> {
    let mut packet = vec![0u8; velikost + 8]; // 8 bajtu pro ICMP header

    // ICMP typ: Echo Request (8)
    packet[0] = 8;
    // ICMP kod: 0
    packet[1] = 0;
    // Checksum: nastavi se pozdeji
    packet[2] = 0;
    packet[3] = 0;
    // Identifier
    packet[4] = 0;
    packet[5] = 1;
    // Sekvencni cislo
    packet[6] = (sekvencni_cislo >> 8) as u8;
    packet[7] = (sekvencni_cislo & 0xff) as u8;

    // Data - jednoduchy vzor
    for i in 8..packet.len() {
        packet[i] = ((i - 8) % 256) as u8;
    }

    // Vypocitat checksum
    let checksum = vypocitat_checksum(&packet);
    packet[2] = (checksum >> 8) as u8;
    packet[3] = (checksum & 0xff) as u8;

    packet
}

fn vypocitat_checksum(data: &[u8]) -> u16 {
    let mut suma = 0u32;
    let mut i = 0;

    // Projit data po 16-bit slovech
    while i + 1 < data.len() {
        if i == 2 {
            // Preskocit checksum pole
            i += 2;
            continue;
        }
        let slovo = ((data[i] as u32) << 8) + (data[i + 1] as u32);
        suma += slovo;
        i += 2;
    }

    // Pridat posledni bajt pokud je lichy pocet
    if i < data.len() && i != 2 {
        suma += (data[i] as u32) << 8;
    }

    // Soucet carry bitu
    while suma >> 16 > 0 {
        suma = (suma & 0xffff) + (suma >> 16);
    }

    // Komplement
    !suma as u16
}

pub fn formatovat_cas(ms: f64) -> String {
    if ms < 1.0 {
        format!("{:.3}ms", ms)
    } else if ms < 1000.0 {
        format!("{:.1}ms", ms)
    } else {
        format!("{:.1}s", ms / 1000.0)
    }
}
