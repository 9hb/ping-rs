//
// DESCRIPTION: ping logic
//
use crate::cli::Args;
use crate::utils::{ prelozit_adresu, vytvorit_icmp_packet, formatovat_cas };
use socket2::{ Domain, Protocol, Socket, Type };
use std::net::{ IpAddr, SocketAddrV4, SocketAddrV6 };
use std::time::{ Duration, Instant };
use tokio::time::sleep;

pub struct PingStatistiky {
    pub odeslano: u32,
    pub prijato: u32,
    pub ztraceno: u32,
    pub min_cas: Option<f64>,
    pub max_cas: Option<f64>,
    pub celkovy_cas: f64,
}

impl PingStatistiky {
    pub fn nova() -> Self {
        Self {
            odeslano: 0,
            prijato: 0,
            ztraceno: 0,
            min_cas: None,
            max_cas: None,
            celkovy_cas: 0.0,
        }
    }

    pub fn pridat_odpoved(&mut self, cas_ms: f64) {
        self.prijato += 1;
        self.celkovy_cas += cas_ms;

        match self.min_cas {
            None => {
                self.min_cas = Some(cas_ms);
            }
            Some(min) if cas_ms < min => {
                self.min_cas = Some(cas_ms);
            }
            _ => {}
        }

        match self.max_cas {
            None => {
                self.max_cas = Some(cas_ms);
            }
            Some(max) if cas_ms > max => {
                self.max_cas = Some(cas_ms);
            }
            _ => {}
        }
    }

    pub fn pridat_ztraceny(&mut self) {
        self.ztraceno += 1;
    }

    pub fn prumer(&self) -> Option<f64> {
        if self.prijato > 0 { Some(self.celkovy_cas / (self.prijato as f64)) } else { None }
    }

    pub fn procento_ztraty(&self) -> f64 {
        if self.odeslano > 0 {
            ((self.ztraceno as f64) / (self.odeslano as f64)) * 100.0
        } else {
            0.0
        }
    }
}

pub async fn spustit_ping(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let cilova_ip = prelozit_adresu(&args.target_addr)?;

    println!("• PING {} ({})", args.target_addr, cilova_ip);
    println!(" → sending {} bytes of data:", args.data_size);

    let mut statistiky = PingStatistiky::nova();

    for sekvencni_cislo in 1..=args.amount {
        statistiky.odeslano += 1;

        match ping_jednou(&cilova_ip, sekvencni_cislo as u16, args.data_size, args.timeout).await {
            Ok(cas_ms) => {
                statistiky.pridat_odpoved(cas_ms);
                println!(
                    "  ← response from {}: bytes={} time={} seq={}",
                    cilova_ip,
                    args.data_size,
                    formatovat_cas(cas_ms),
                    sekvencni_cislo
                );
            }
            Err(e) => {
                statistiky.pridat_ztraceny();
                println!("✖ time limit for seq={}: {}", sekvencni_cislo, e);
            }
        }

        // Pockej pred dalsim pingem (krome posledniho)
        if sekvencni_cislo < args.amount {
            sleep(Duration::from_millis(args.interval)).await;
        }
    }

    zobrazit_statistiky(&args.target_addr, &statistiky);
    Ok(())
}

async fn ping_jednou(
    cilova_ip: &IpAddr,
    sekvencni_cislo: u16,
    data_size: usize,
    timeout_secs: u64
) -> Result<f64, Box<dyn std::error::Error>> {
    // Vytvorit raw socket
    let domain = match cilova_ip {
        IpAddr::V4(_) => Domain::IPV4,
        IpAddr::V6(_) => Domain::IPV6,
    };

    let socket = Socket::new(domain, Type::RAW, Some(Protocol::ICMPV4))?;
    socket.set_read_timeout(Some(Duration::from_secs(timeout_secs)))?;
    socket.set_write_timeout(Some(Duration::from_secs(timeout_secs)))?;

    // Vytvorit ICMP packet
    let packet = vytvorit_icmp_packet(data_size, sekvencni_cislo);

    // Odeslat packet
    let start_time = Instant::now();
    let dest_addr = match &cilova_ip {
        IpAddr::V4(ipv4) => SocketAddrV4::new(*ipv4, 0).into(),
        IpAddr::V6(ipv6) => SocketAddrV6::new(*ipv6, 0, 0, 0).into(),
    };

    socket.send_to(&packet, &dest_addr)?;

    // Cekej na odpoved
    let mut buffer = vec![std::mem::MaybeUninit::new(0u8); 1024];
    match socket.recv_from(&mut buffer) {
        Ok((velikost, _)) => {
            let elapsed = start_time.elapsed();

            // Konvertovat MaybeUninit na inicializovana data
            let mut inicializovany_buffer = vec![0u8; velikost];
            for i in 0..velikost {
                inicializovany_buffer[i] = unsafe { buffer[i].assume_init() };
            }

            // Jednoducha kontrola ze je to ICMP Echo Reply
            if velikost >= 28 {
                // IP header (20) + ICMP header (8)
                let icmp_offset = 20; // Preskocit IP header
                if inicializovany_buffer[icmp_offset] == 0 {
                    // ICMP typ 0 = Echo Reply
                    return Ok(elapsed.as_secs_f64() * 1000.0);
                }
            }
        }
        Err(e) => {
            if elapsed_timeout(start_time, timeout_secs) {
                return Err("✖ time limit exceeded".into());
            }
            return Err(e.into());
        }
    }

    if elapsed_timeout(start_time, timeout_secs) {
        return Err("✖ time limit exceeded".into());
    }

    Ok(0.0) // Pokud nedoslo k odpovedi, vratime 0.0 (ztraceno)
}

fn elapsed_timeout(start_time: Instant, timeout_secs: u64) -> bool {
    start_time.elapsed() >= Duration::from_secs(timeout_secs)
}

fn zobrazit_statistiky(target_addr: &str, stats: &PingStatistiky) {
    println!("\n╭─[ {} ping statistics ]", target_addr);
    println!(
        "│  ← {} sent, {} received, {} lost ({:.1}% loss)",
        stats.odeslano,
        stats.prijato,
        stats.ztraceno,
        stats.procento_ztraty()
    );

    if let Some(prumer) = stats.prumer() {
        let min = stats.min_cas.unwrap_or(0.0);
        let max = stats.max_cas.unwrap_or(0.0);

        println!(
            "│  ← min={}  |  avg={}  |  max={}\n╰─────────────────────────────────────────────",
            formatovat_cas(min),
            formatovat_cas(prumer),
            formatovat_cas(max)
        );
    }
}
