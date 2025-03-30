use std::io::{Read, Write};
use std::{io::ErrorKind, time::Duration};

use clap::Args;
use mac_address::mac_address_by_name;
use mac_address::MacAddress;
use smol::future::FutureExt;
use smol::Timer;

use super::CommandOperation;
use crate::message;
use crate::message::customer_info_read::{CustomerInfoReadRequest, CustomerInfoReadResponse};
use crate::message::header::RequestHeader;
use crate::message::MessageOperation;
use crate::message_builder::MessageBuilder;
use crate::packet_sock;

/// Do MSG_RMU_REG_CUSTOMER_INFO_READ request
#[derive(Args, Debug)]
pub struct CustomerInfoReadCmd {
    #[arg(short, long)]
    interface: String,

    #[arg(short, long, default_value_t = 100)]
    timeout_ms: u32,

    #[arg(short, long)]
    #[arg(default_value_t = String::from("01:50:43:00:00:03"))]
    mac: String,

    #[arg(short, long, value_parser=clap_num::maybe_hex::<u8>)]
    devid: u8,
}

impl CommandOperation for CustomerInfoReadCmd {
    fn process(&self) -> anyhow::Result<()> {
        let sock = packet_sock::create_rmu_sock(&self.interface)?;
        let smac = mac_address_by_name(&self.interface)?.unwrap().bytes();
        let dmac = self.mac.parse::<MacAddress>()?.bytes();

        smol::block_on(async {
            let mut reqmsg = Into::<MessageBuilder<CustomerInfoReadRequest>>::into(
                MessageBuilder::<RequestHeader>::new()
                    .destination_address(&dmac)
                    .source_address(&smac)
                    .device_id(self.devid),
            )
            .build()?;

            // @todo: buffer size handle
            let mut wbuf = [0u8; 60];
            let _ = message::marshal(&mut reqmsg, &mut wbuf)?;
            sock.write_with(|mut s| s.write(&wbuf[..])).await?;

            loop {
                let mut rbuf = [0; 1514];
                let res = sock
                    .read_with(|mut s| s.read(&mut rbuf))
                    .or(async {
                        Timer::after(Duration::from_millis(self.timeout_ms.into())).await;
                        Err(ErrorKind::TimedOut.into())
                    })
                    .await;

                match res {
                    Ok(sz) if sz > 0 => {
                        let msg: CustomerInfoReadResponse = message::unmarshal(&rbuf[..sz])?;
                        let mac: MacAddress = msg.header().source_address().into();
                        println!(
                            "mac:{mac} devid:0x{:02X} info:{}",
                            msg.header().device_id(),
                            msg.info.to_string_lossy(),
                        );
                    }
                    Ok(_) => continue,
                    Err(e) if e.kind() == ErrorKind::TimedOut => break,
                    Err(e) => return Err(e.into()),
                }
            }

            Ok(())
        })
    }
}
