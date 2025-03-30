use std::io::{self, Read, Write};
use std::io::ErrorKind;

use anyhow::anyhow;
use bit_ops::bitops_u16;
use clap::Args;
use mac_address::mac_address_by_name;
use mac_address::MacAddress;

use crate::message::header::RequestHeader;
use crate::message::register::{RegOpRequest, RegOpRequestList, RegOpResponse};
use crate::message::register::{RegisterRequest, RegisterResponse};
use crate::message::{self, MessageOperation};
use crate::message_builder::MessageBuilder;
use crate::packet_sock;

use super::CommandOperation;

#[derive(Args, Debug)]
pub struct ReadVtuCmd {
    #[arg(short, long)]
    interface: String,

    #[arg(short, long, default_value_t = 100)]
    timeout_ms: u32,

    #[arg(short, long)]
    #[arg(default_value_t = String::from("01:50:43:00:00:03"))]
    mac: String,

    #[arg(short, long, value_parser=clap_num::maybe_hex::<u8>)]
    devid: u8,

    #[arg(long, default_value_t = false)]
    print_reg: bool,
}

fn build_prepare_requests() -> RegOpRequestList {
    let mut oplist = RegOpRequestList::new();

    oplist.add_regop(RegOpRequest::WaitOnBit0 {
        addr: 0x1B,
        reg: 0x05,
        bit: 15,
    });

    oplist.add_regop(RegOpRequest::Write {
        addr: 0x1B,
        reg: 0x06,
        data: 0x2FFF,
    });

    oplist
}

fn build_requests() -> RegOpRequestList {
    let mut oplist = RegOpRequestList::new();

    oplist.add_regop(RegOpRequest::Write {
        addr: 0x1B,
        reg: 0x05,
        data: 0xC000,
    });

    oplist.add_regop(RegOpRequest::WaitOnBit0 {
        addr: 0x1B,
        reg: 0x05,
        bit: 15,
    });

    // VTU FID
    oplist.add_regop(RegOpRequest::Read {
        addr: 0x1B,
        reg: 0x02,
    });
    // VTU_VID
    oplist.add_regop(RegOpRequest::Read {
        addr: 0x1B,
        reg: 0x06,
    });

    oplist.add_regop(RegOpRequest::Read {
        addr: 0x1B,
        reg: 0x07,
    });

    oplist.add_regop(RegOpRequest::Read {
        addr: 0x1B,
        reg: 0x08,
    });
    oplist
}

async fn proccmd(cmd: &ReadVtuCmd) -> anyhow::Result<()> {
    let sock = packet_sock::create_rmu_sock(&cmd.interface)?;
    let smac = mac_address_by_name(&cmd.interface)?.unwrap().bytes();
    let dmac = cmd.mac.parse::<MacAddress>()?.bytes();

    let mut requ = Into::<MessageBuilder<RegisterRequest>>::into(
        MessageBuilder::<RequestHeader>::new()
            .destination_address(&dmac)
            .source_address(&smac)
            .device_id(cmd.devid),
    )
    .regops(build_prepare_requests())
    .build()?;

    let mut wbuf = message::prealloc_buffer(&requ);
    let _ = requ.marshal(&mut wbuf[..])?;
    sock.write_with(|mut s| s.write(&wbuf[..])).await?;

    let mut rbuf = [0; 1514];
    let sz = sock.read_with(|mut s| s.read(&mut rbuf)).await?;
    if sz == 0 {
        return Err(io::Error::from(ErrorKind::UnexpectedEof).into());
    }

    // @todo: how to check successful
    let _resp = message::unmarshal::<RegisterResponse>(&rbuf[..sz])?;
    // println!("{:04X?}", resp.regops);

    let mut first_vid = None;
    let mut seqno = 1;
    loop {
        let mut req = Into::<MessageBuilder<RegisterRequest>>::into(
            MessageBuilder::<RequestHeader>::new()
                .destination_address(&dmac)
                .source_address(&smac)
                .sequence_number(seqno)
                .device_id(cmd.devid),
        )
        .regops(build_requests())
        .build()?;

        seqno += 1;

        let mut wbuf = message::prealloc_buffer(&req);
        let _ = req.marshal(&mut wbuf[..])?;
        sock.write_with(|mut s| s.write(&wbuf[..])).await?;
        let mut rbuf = [0; 1514];
        let sz = sock.read_with(|mut s| s.read(&mut rbuf)).await?;
        if sz == 0 {
            return Err(io::Error::from(ErrorKind::UnexpectedEof).into());
        }

        let resp = message::unmarshal::<RegisterResponse>(&rbuf[..sz])?;
        if req.regops.as_ref().len() != resp.regops.as_ref().len() {
            eprintln!("response with error: {:04x?}", resp.regops);
            return Err(io::Error::from(ErrorKind::InvalidData).into());
        }

        // @todo: index with special named (reg+ops?) as a key
        // for now, index is determined by request
        let opvec = resp.regops.as_ref();

        let vtu_vid = match opvec[3] {
            RegOpResponse::Read { data, .. } => data,
            _ => return Err(anyhow!("read vtu_vid fail")),
        };

        let vid = bitops_u16::get_bits(vtu_vid, 12, 0);
        let valid = bitops_u16::get_bit(vtu_vid, 12);
        let page = bitops_u16::get_bit(vtu_vid, 13);

        if (valid == 0) || (first_vid.is_some_and(|x| x == vid)) {
            break;
        }

        if first_vid.is_none() {
            first_vid = Some(vid);
        }

        let vtu_fid = match opvec[2] {
            RegOpResponse::Read { data, .. } => data,
            _ => return Err(anyhow!("read vtu_fid fail")),
        };
        let fid = bitops_u16::get_bits(vtu_fid, 12, 0);

        println!("fid:{} vid(H):{:04X} page:{}", fid, vid, page);

        if cmd.print_reg {
            let vtu_data_p0p7 = match opvec[4] {
                RegOpResponse::Read { data, .. } => data,
                _ => return Err(anyhow!("read vtu_data_p0_p7 fail")),
            };

            let vtu_data_p8p9 = match opvec[5] {
                RegOpResponse::Read { data, .. } => data,
                _ => return Err(anyhow!("read vtu_data_p8_p9 fail")),
            };

            print!(" |- vtu_fid:{:04X} vtu_vid:{:04X}", vtu_fid, vtu_vid);
            print!(
                " vtu_data_p0p7:{:04X} vtu_data_p8p9:{:04X}",
                vtu_data_p0p7, vtu_data_p8p9
            );
            println!("");
        }
    }

    Ok(())
}

impl CommandOperation for ReadVtuCmd {
    fn process(&self) -> anyhow::Result<()> {
        smol::block_on(proccmd(self))
    }
}
