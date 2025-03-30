use std::io::{Read, Write};
use std::{io::ErrorKind, time::Duration};

use clap::Args;
use mac_address::mac_address_by_name;
use smol::{future::FutureExt, Timer};

use super::CommandOperation;
use super::RMU_MULTICAST_ADDR;
use crate::message::getid::GetIdRequest;
use crate::message::getid::GetIdResponse;
use crate::message::header::RequestHeader;
use crate::message::MessageOperation;
use crate::message_builder::MessageBuilder;
use crate::packet_sock;

/// Scan all marvell switch devices through multicast
#[derive(Args, Debug)]
pub struct ScanCmd {
    #[arg(short, long)]
    interface: String,

    #[arg(short, long, default_value_t = 1000)]
    timeout_ms: u32,
}

#[derive(Debug)]
struct DevInfo {
    mac: [u8; 6],
    devid: u8,
    prodno: u16,
}

impl std::fmt::Display for DevInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let _ = write!(
            f,
            "mac: {:<02X}:{:<02X}:{:<02X}:{:<02X}:{:<02X}:{:<02X} devid:0x{:<02X} prodno:0x{:<04X}",
            self.mac[0],
            self.mac[1],
            self.mac[2],
            self.mac[3],
            self.mac[4],
            self.mac[5],
            self.devid,
            self.prodno,
        );

        Ok(())
    }
}

impl CommandOperation for ScanCmd {
    fn process(&self) -> anyhow::Result<()> {
        smol::block_on(async {
            let sock = packet_sock::create_rmu_sock(&self.interface)?;
            let smac = mac_address_by_name(&self.interface)?.unwrap().bytes();

            let mut seqno = 0;
            let mut devs: Vec<DevInfo> = Vec::new();
            let mut updown = true;

            for devid in 0x00..=0x1F {
                if updown {
                    eprint!("\rStarting Scan |");
                } else {
                    eprint!("\rStarting Scan -");
                }
                updown = !updown;

                let mut reqmsg = Into::<MessageBuilder<GetIdRequest>>::into(
                    MessageBuilder::<RequestHeader>::new()
                        .destination_address(&RMU_MULTICAST_ADDR)
                        .source_address(&smac)
                        .device_id(devid)
                        .sequence_number(seqno),
                )
                .build()?;

                let mut wbuf = [0u8; 60];
                let _ = reqmsg.marshal(&mut wbuf)?;
                sock.write_with(|mut s| s.write(&wbuf[..])).await?;

                // loop if multi device response
                loop {
                    let mut rbuf = [0; 1514];
                    let res = sock
                        .read_with(|mut s| s.read(&mut rbuf))
                        .or(async {
                            Timer::after(Duration::from_millis(100)).await;
                            Err(ErrorKind::TimedOut.into())
                        })
                        .await;

                    match res {
                        Ok(sz) if sz > 0 => {
                            let respmsg = GetIdResponse::unmarshal(&rbuf[..sz])?;
                            devs.push(DevInfo {
                                mac: respmsg.header().source_address(),
                                devid: respmsg.header().device_id(),
                                prodno: respmsg.header().product_number(),
                            });
                        }
                        Ok(_) => {}
                        Err(e) => {
                            if e.kind() != ErrorKind::TimedOut {
                                eprintln!("Error: {e}");
                            }

                            break;
                        }
                    }
                }

                seqno += 1;
            }

            println!("\nScan done: {} devices", devs.len());
            devs.iter().for_each(|dev| println!(" {}", dev));

            Ok(())
        })
    }
}
