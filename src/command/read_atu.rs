use std::io::ErrorKind;
use std::io::{self, Read, Write};

use bit_ops::bitops_u16;
use clap::Args;
use mac_address::mac_address_by_name;
use mac_address::MacAddress;

use crate::message;
use crate::message::header::RequestHeader;
use crate::message::register::{RegOpRequest, RegOpRequestList, RegOpResponse};
use crate::message::register::{RegisterRequest, RegisterResponse};
use crate::message::MessageOperation;
use crate::message_builder::MessageBuilder;
use crate::packet_sock;

use super::CommandOperation;

#[derive(Args, Debug)]
pub struct ReadAtuCmd {
    #[arg(short, long)]
    interface: String,

    #[arg(short, long, default_value_t = 100)]
    timeout_ms: u32,

    #[arg(short, long)]
    #[arg(default_value_t = String::from("01:50:43:00:00:03"))]
    mac: String,

    #[arg(short, long, value_parser=clap_num::maybe_hex::<u8>)]
    devid: u8,

    #[arg(short, long)]
    fid: u8,

    /// Print raw register value too
    #[arg(long)]
    #[arg(default_value_t = false)]
    print_reg: bool,
}

fn build_prepare_requests(fid: u8) -> RegOpRequestList {
    let mut oplist = RegOpRequestList::new();

    oplist.add_regop(RegOpRequest::WaitOnBit0 {
        addr: 0x1B,
        reg: 0x0B,
        bit: 15,
    });
    oplist.add_regop(RegOpRequest::Write {
        addr: 0x1B,
        reg: 0x0D,
        data: 0xFFFF,
    });
    oplist.add_regop(RegOpRequest::Write {
        addr: 0x1B,
        reg: 0x0E,
        data: 0xFFFF,
    });
    oplist.add_regop(RegOpRequest::Write {
        addr: 0x1B,
        reg: 0x0F,
        data: 0xFFF,
    });

    oplist.add_regop(RegOpRequest::Write {
        addr: 0x1B,
        reg: 0x01,
        data: fid as u16,
    });

    oplist
}

fn build_requests() -> RegOpRequestList {
    let mut oplist = RegOpRequestList::new();
    oplist.add_regop(RegOpRequest::Write {
        addr: 0x1B,
        reg: 0x0B,
        data: 0xC000,
    });
    oplist.add_regop(RegOpRequest::WaitOnBit0 {
        addr: 0x1B,
        reg: 0x0B,
        bit: 15,
    });
    // ATU_OP
    oplist.add_regop(RegOpRequest::Read {
        addr: 0x1B,
        reg: 0x0B,
    });
    // ATU_FID
    oplist.add_regop(RegOpRequest::Read {
        addr: 0x1B,
        reg: 0x01,
    });
    // ATU_DATA
    oplist.add_regop(RegOpRequest::Read {
        addr: 0x1B,
        reg: 0x0C,
    });

    // MAC
    oplist.add_regop(RegOpRequest::Read {
        addr: 0x1B,
        reg: 0x0D,
    });
    oplist.add_regop(RegOpRequest::Read {
        addr: 0x1B,
        reg: 0x0E,
    });
    oplist.add_regop(RegOpRequest::Read {
        addr: 0x1B,
        reg: 0x0F,
    });

    oplist
}

async fn proccmd(cmd: &ReadAtuCmd) -> anyhow::Result<()> {
    let sock = packet_sock::create_rmu_sock(&cmd.interface)?;
    let smac = mac_address_by_name(&cmd.interface)?.unwrap().bytes();
    let dmac = cmd.mac.parse::<MacAddress>()?.bytes();

    let mut requ = Into::<MessageBuilder<RegisterRequest>>::into(
        MessageBuilder::<RequestHeader>::new()
            .destination_address(&dmac)
            .source_address(&smac)
            .device_id(cmd.devid),
    )
    .regops(build_prepare_requests(cmd.fid))
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

    let mut seqno = 1;
    // @fixup: why first read is entry_state is 0
    let mut exit_count = 0;

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
        let rvec = resp.regops.as_ref();

        let atu_data = if let RegOpResponse::Read { data, .. } = rvec[4] {
            data
        } else {
            return Err(anyhow::anyhow!("read atu_data fail"));
        };
        let entry_state = bitops_u16::get_bits(atu_data, 4, 0);
        if entry_state == 0 {
            if exit_count == 1 {
                break;
            }

            exit_count += 1;
            continue;
        }
        let portvec = bitops_u16::get_bits(atu_data, 10, 4);

        let atu_op = if let RegOpResponse::Read { data, .. } = rvec[2] {
            data
        } else {
            return Err(anyhow::anyhow!("read atu_op fail"));
        };
        let mac_qpri = bitops_u16::get_bits(atu_op, 3, 8);
        let mac_fpri = bitops_u16::get_bits(atu_op, 3, 0);

        let atu_fid = if let RegOpResponse::Read { data, .. } = rvec[3] {
            data
        } else {
            return Err(anyhow::anyhow!("read atu_fid fail"));
        };
        let _fid = bitops_u16::get_bits(atu_fid, 12, 0);

        let mac01 = match rvec[5] {
            RegOpResponse::Read { data, .. } => data.to_be_bytes(),
            _ => return Err(anyhow::anyhow!("read atu_mac01 fail")),
        };
        let mac23 = match rvec[6] {
            RegOpResponse::Read { data, .. } => data.to_be_bytes(),
            _ => return Err(anyhow::anyhow!("read atu_mac23 fail")),
        };
        let mac45 = match rvec[7] {
            RegOpResponse::Read { data, .. } => data.to_be_bytes(),
            _ => return Err(anyhow::anyhow!("read atu_mac45 fail")),
        };

        println!(
            "mac(H):{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X} \
             entry_state(H):{:X} portvec(B):{:010b} qpri:{} fpri:{}",
            mac01[0],
            mac01[1],
            mac23[0],
            mac23[1],
            mac45[0],
            mac45[1],
            entry_state,
            portvec,
            mac_qpri,
            mac_fpri,
        );

        if cmd.print_reg {
            println!(
                " |- atu_op:{:04X} atu_data:{:04X} atu_fid:{:04X}",
                atu_op, atu_data, atu_fid
            );
        }
    }

    Ok(())
}

impl CommandOperation for ReadAtuCmd {
    fn process(&self) -> anyhow::Result<()> {
        smol::block_on(proccmd(self))
    }
}
