use std::io::{Read, Write};
use std::{io::ErrorKind, time::Duration};

use clap::Args;
use mac_address::mac_address_by_name;
use mac_address::MacAddress;
use smol::future::FutureExt;
use smol::Timer;
use socket2::Socket;

use super::CommandOperation;
use crate::message::customer_info_read::CustomerInfoReadRequest;
use crate::message::customer_info_read::CustomerInfoReadResponse;
use crate::message::fw_version::FwVersionRequest;
use crate::message::fw_version::FwVersionResponse;
use crate::message::header::RequestHeader;
use crate::message::{self, MessageOperation};
use crate::message_builder::MessageBuilder;
use crate::packet_sock;

/// Fetch customer-info and fw-version
#[derive(Args, Debug)]
pub struct SoftwareInfoCmd {
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

fn wire_buffer_size<T: MessageOperation>(message: &T) -> usize {
    if message.wire_size() < 60 {
        return 60;
    }
    message.wire_size()
}

async fn fw_version_get(
    cmd: &SoftwareInfoCmd,
    sock: &smol::Async<Socket>,
    smac: &[u8; 6],
    dmac: &[u8; 6],
) -> anyhow::Result<Vec<FwVersionResponse>> {
    let mut responses = Vec::new();

    let mut reqmsg = Into::<MessageBuilder<FwVersionRequest>>::into(
        MessageBuilder::<RequestHeader>::new()
            .destination_address(&dmac)
            .source_address(&smac)
            .device_id(cmd.devid),
    )
    .build()?;

    let mut wbuf = [0u8; 60];
    let _ = reqmsg.marshal(&mut wbuf)?;
    sock.write_with(|mut s| s.write(&wbuf[..])).await?;

    loop {
        let mut rbuf = [0; 1514];
        let res = sock
            .read_with(|mut s| s.read(&mut rbuf))
            .or(async {
                Timer::after(Duration::from_millis(cmd.timeout_ms.into())).await;
                Err(ErrorKind::TimedOut.into())
            })
            .await;

        match res {
            Ok(sz) if sz > 0 => {
                let msg = message::unmarshal::<FwVersionResponse>(&rbuf[..sz])?;
                responses.push(msg);
            }
            Ok(_) => continue,
            Err(e) if e.kind() == ErrorKind::TimedOut => break,
            Err(e) => return Err(e.into()),
        }
    }

    Ok(responses)
}

async fn customer_info_read(
    cmd: &SoftwareInfoCmd,
    sock: &smol::Async<Socket>,
    smac: &[u8; 6],
    dmac: &[u8; 6],
) -> anyhow::Result<Vec<CustomerInfoReadResponse>> {
    let mut responses = Vec::new();

    let mut reqmsg = Into::<MessageBuilder<CustomerInfoReadRequest>>::into(
        MessageBuilder::<RequestHeader>::new()
            .destination_address(&dmac)
            .source_address(&smac)
            .device_id(cmd.devid),
    )
    .build()?;

    // @todo: buffer size handle
    let mut wbuf = [0u8; 60];
    let _ = reqmsg.marshal(&mut wbuf)?;
    sock.write_with(|mut s| s.write(&wbuf[..])).await?;

    loop {
        let mut rbuf = [0; 1514];
        let res = sock
            .read_with(|mut s| s.read(&mut rbuf))
            .or(async {
                Timer::after(Duration::from_millis(cmd.timeout_ms.into())).await;
                Err(ErrorKind::TimedOut.into())
            })
            .await;

        match res {
            Ok(sz) if sz > 0 => {
                let msg = message::unmarshal::<CustomerInfoReadResponse>(&rbuf[..sz])?;
                responses.push(msg);
            }
            Ok(_) => continue,
            Err(e) if e.kind() == ErrorKind::TimedOut => break,
            Err(e) => return Err(e.into()),
        }
    }

    Ok(responses)
}

impl CommandOperation for SoftwareInfoCmd {
    fn process(&self) -> anyhow::Result<()> {
        let sock = packet_sock::create_rmu_sock(&self.interface)?;
        let smac = mac_address_by_name(&self.interface)?.unwrap().bytes();
        let dmac = self.mac.parse::<MacAddress>()?.bytes();

        smol::block_on(async {
            let cusinfos = customer_info_read(self, &sock, &smac, &dmac).await?;
            let fwvers = fw_version_get(self, &sock, &smac, &dmac).await?;

            if cusinfos.is_empty() {
                return Err(anyhow::anyhow!(
                    "Not device: mac:{} devid:{:02X}",
                    self.mac,
                    self.devid
                ));
            }

            cusinfos.iter().for_each(|cusinfo| {
                let mac: MacAddress = cusinfo.header().source_address().into();
                let devid = cusinfo.header().device_id();

                print!(
                    "Mac:{} Id:0x{:02X} Prodno:0x{:04X}\n info: {}\n",
                    mac,
                    devid,
                    cusinfo.header().product_number(),
                    cusinfo.info.to_string_lossy(),
                );

                fwvers.iter().for_each(|fwver| {
                    let m: MacAddress = fwver.header().source_address().into();
                    let id = fwver.header().device_id();
                    if m == mac && devid == id {
                        print!(" build: {}\n", fwver.build_string());
                        return;
                    }
                });
            });

            Ok(())
        })
    }
}
