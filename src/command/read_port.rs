use std::io::ErrorKind;
use std::io::{self, Read, Write};
use std::str::FromStr;

use anyhow::anyhow;
use clap::Args;
use mac_address::mac_address_by_name;
use mac_address::MacAddress;
use strum::IntoEnumIterator;

use crate::message;
use crate::message::header::RequestHeader;
use crate::message::register::{RegOpRequest, RegOpResponse};
use crate::message::register::{RegisterRequest, RegisterResponse};
use crate::message::MessageOperation;
use crate::message_builder::MessageBuilder;
use crate::packet_sock;
use crate::reginfo::PhysicalControl;
use crate::reginfo::{u16_get_bits, PortRegister, PortSTatus};

use super::CommandOperation;

#[derive(Args, Debug)]
pub struct ReadPortRegCmd {
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
    portid: u8,

    #[arg(short, long, value_enum)]
    register: PortRegister,

    #[arg(short, long, num_args = 1, value_delimiter = ',')]
    fields: Option<Vec<String>>,

    /// Print raw register value too
    #[arg(long)]
    #[arg(default_value_t = false)]
    print_reg: bool,
}

fn read_port_register(val: u16, reg: PortRegister, fields: &Option<Vec<String>>) {
    match reg {
        PortRegister::PortStatus => {
            println!("PortStatus:");
            if fields.is_none() {
                for field in PortSTatus::iter() {
                    println!(" {} {}", field, u16_get_bits(val, field));
                }
            } else {
                for field in fields.as_ref().unwrap() {
                    let field = PortSTatus::from_str(field);
                    if field.is_err() {
                        continue;
                    }

                    let field = field.unwrap();
                    println!(" {} {}", field, u16_get_bits(val, field));
                }
            }
        }

        PortRegister::PhysicalControl => {
            println!("PhysicalControl:");
            if fields.is_none() {
                for field in PhysicalControl::iter() {
                    println!(" {} {}", field, u16_get_bits(val, field));
                }
            } else {
                for field in fields.as_ref().unwrap() {
                    let field = PhysicalControl::from_str(field);
                    if field.is_err() {
                        continue;
                    }

                    let field = field.unwrap();
                    println!(" {} {}", field, u16_get_bits(val, field));
                }
            }
        }
        _ => todo!("reg: {:?}", reg),
    }
}

async fn proccmd(cmd: &ReadPortRegCmd) -> anyhow::Result<()> {
    let sock = packet_sock::create_rmu_sock(&cmd.interface)?;
    let smac = mac_address_by_name(&cmd.interface)?.unwrap().bytes();
    let dmac = cmd.mac.parse::<MacAddress>()?.bytes();

    let mut req = Into::<MessageBuilder<RegisterRequest>>::into(
        MessageBuilder::<RequestHeader>::new()
            .destination_address(&dmac)
            .source_address(&smac)
            .device_id(cmd.devid),
    )
    .add_regop(RegOpRequest::Read {
        addr: cmd.portid,
        reg: cmd.register as u8,
    })
    .build()?;

    let mut wbuf = message::prealloc_buffer(&req);
    let _ = req.marshal(&mut wbuf[..])?;
    sock.write_with(|mut s| s.write(&wbuf[..])).await?;

    // @todo: timeout
    let mut rbuf = [0; 1514];
    let sz = sock.read_with(|mut s| s.read(&mut rbuf)).await?;
    if sz == 0 {
        return Err(io::Error::from(ErrorKind::UnexpectedEof).into());
    }

    let resp = message::unmarshal::<RegisterResponse>(&rbuf[..sz])?;
    let regvec = resp.regops.as_ref();
    let val = match regvec[0] {
        RegOpResponse::Read { data, .. } => data,
        _ => return Err(anyhow!("read {:?} fail", cmd.register)),
    };

    read_port_register(val, cmd.register, &cmd.fields);
    Ok(())
}

impl CommandOperation for ReadPortRegCmd {
    fn process(&self) -> anyhow::Result<()> {
        smol::block_on(proccmd(self))
    }
}
