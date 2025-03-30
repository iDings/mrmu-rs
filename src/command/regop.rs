use std::io::{Read, Write};
use std::{io::ErrorKind, time::Duration};

use clap::Args;
use mac_address::mac_address_by_name;
use mac_address::MacAddress;
use smol::future::FutureExt;
use smol::Timer;

use crate::message;
use crate::message::header::RequestHeader;
use crate::message::register::RegOpRequest;
use crate::message::register::RegOpRequestList;
use crate::message::register::RegisterRequest;
use crate::message::register::RegisterResponse;
use crate::message::MessageOperation;
use crate::message_builder::MessageBuilder;
use crate::packet_sock;

use super::CommandOperation;

/// Perform register opeartions
#[derive(Args, Debug)]
pub struct RegOpCmd {
    #[arg(short, long)]
    interface: String,

    #[arg(short, long, default_value_t = 100)]
    timeout_ms: u32,

    #[arg(short, long)]
    #[arg(default_value_t = String::from("01:50:43:00:00:03"))]
    mac: String,

    #[arg(short, long, value_parser=clap_num::maybe_hex::<u8>)]
    devid: u8,

    /// Action list to do register operation
    ///
    /// Support register operations:
    ///   READ:addr=[],reg=[] | WRITE:addr=[],reg=[],data=[] |
    ///   WaitOnBit0:addr=[],reg=[],bit=[] | WaitOnBit1:addr=[],reg=[],bit=[]
    #[arg(short, long, num_args=1..)]
    actions: Vec<String>,
}

fn strtoint<T: num_traits::Num>(val: &str) -> Option<T> {
    if val.starts_with("0x") || val.starts_with("0X") {
        return T::from_str_radix(&val[2..], 16).ok();
    }

    if val.starts_with("0o") || val.starts_with("0o") {
        return T::from_str_radix(&val[2..], 8).ok();
    }

    if val.starts_with("0b") || val.starts_with("0B") {
        return T::from_str_radix(&val[2..], 2).ok();
    }

    T::from_str_radix(&val[..], 10).ok()
}

fn parse_read(param: &str, last_addr: &mut Option<u8>) -> anyhow::Result<RegOpRequest> {
    let mut addr: Option<u8> = None;
    let mut reg: Option<u8> = None;

    let paras: Vec<_> = param.split(',').collect();
    for para in paras {
        let kv: Vec<_> = para.split('=').collect();
        if kv.len() != 2 {
            return Err(anyhow::anyhow!("wrong read keyval pair"));
        }

        let key = kv[0].trim().to_lowercase();
        let val = kv[1].trim().to_lowercase();
        match key.as_str() {
            "addr" => addr = strtoint(&val),
            "reg" => reg = strtoint(&val),
            _ => (),
        }
    }

    addr = match addr {
        Some(x) => {
            last_addr.replace(x);
            Some(x)
        }
        None if last_addr.is_some() => Some(last_addr.unwrap()),
        None => None,
    };

    if addr.is_none() || reg.is_none() {
        return Err(anyhow::anyhow!(
            "{}:{} invalid params: addr@{:?} reg@{:?}",
            file!(),
            line!(),
            addr,
            reg
        ));
    }

    Ok(RegOpRequest::Read {
        addr: addr.unwrap(),
        reg: reg.unwrap(),
    })
}

fn parse_write(param: &str, last_addr: &mut Option<u8>) -> anyhow::Result<RegOpRequest> {
    let mut addr: Option<u8> = None;
    let mut reg: Option<u8> = None;
    let mut data: Option<u16> = None;

    let paras: Vec<_> = param.split(',').collect();
    for para in paras {
        let kv: Vec<_> = para.split('=').collect();
        if kv.len() != 2 {
            return Err(anyhow::anyhow!("wrong read keyval pair"));
        }

        let key = kv[0].trim().to_lowercase();
        let val = kv[1].trim().to_lowercase();
        match key.as_str() {
            "addr" => addr = strtoint(&val),
            "reg" => reg = strtoint(&val),
            "data" => data = strtoint(&val),
            _ => (),
        }
    }

    addr = match addr {
        Some(x) => {
            last_addr.replace(x);
            Some(x)
        }
        None if last_addr.is_some() => Some(last_addr.unwrap()),
        None => None,
    };

    if addr.is_none() || reg.is_none() || data.is_none() {
        return Err(anyhow::anyhow!(
            "{}:{} invalid params: addr@{:?} reg@{:?}",
            file!(),
            line!(),
            addr,
            reg
        ));
    }

    Ok(RegOpRequest::Write {
        addr: addr.unwrap(),
        reg: reg.unwrap(),
        data: data.unwrap(),
    })
}

fn parse_waitbit(
    param: &str,
    bit0: bool,
    last_addr: &mut Option<u8>,
) -> anyhow::Result<RegOpRequest> {
    let mut addr: Option<u8> = None;
    let mut reg: Option<u8> = None;
    let mut bit: Option<u8> = None;

    let paras: Vec<_> = param.split(',').collect();
    for para in paras {
        let kv: Vec<_> = para.split('=').collect();
        if kv.len() != 2 {
            return Err(anyhow::anyhow!("wrong read keyval pair"));
        }

        let key = kv[0].trim().to_lowercase();
        let val = kv[1].trim().to_lowercase();
        match key.as_str() {
            "addr" => addr = strtoint(&val),
            "reg" => reg = strtoint(&val),
            "bit" => bit = strtoint(&val),
            _ => (),
        }
    }

    addr = match addr {
        Some(x) => {
            last_addr.replace(x);
            Some(x)
        }
        None if last_addr.is_some() => Some(last_addr.unwrap()),
        None => None,
    };

    if addr.is_none() || reg.is_none() || bit.is_none() {
        return Err(anyhow::anyhow!(
            "{}:{} invalid params: addr@{:?} reg@{:?}",
            file!(),
            line!(),
            addr,
            reg
        ));
    }

    if bit0 {
        return Ok(RegOpRequest::WaitOnBit0 {
            addr: addr.unwrap(),
            reg: reg.unwrap(),
            bit: bit.unwrap(),
        });
    }

    Ok(RegOpRequest::WaitOnBit1 {
        addr: addr.unwrap(),
        reg: reg.unwrap(),
        bit: bit.unwrap(),
    })
}

fn parse_actions(actions: &Vec<String>) -> anyhow::Result<RegOpRequestList> {
    let mut oplist = RegOpRequestList::new();
    let mut last_addr: Option<u8> = None;

    for action in actions {
        let actpara: Vec<_> = action.split(':').collect();
        if actpara.len() != 2 {
            return Err(anyhow::anyhow!("wrong action format:{}", action));
        }

        match actpara[0].to_lowercase().as_str() {
            "read" => oplist.add_regop(parse_read(actpara[1], &mut last_addr)?),
            "write" => oplist.add_regop(parse_write(actpara[1], &mut last_addr)?),
            "waitbit0" => oplist.add_regop(parse_waitbit(actpara[1], true, &mut last_addr)?),
            "waitbit1" => oplist.add_regop(parse_waitbit(actpara[1], false, &mut last_addr)?),
            _ => return Err(anyhow::anyhow!("invalid op:{}", actpara[0])),
        }
    }

    Ok(oplist)
}

impl CommandOperation for RegOpCmd {
    fn process(&self) -> anyhow::Result<()> {
        let sock = packet_sock::create_rmu_sock(&self.interface)?;
        let smac = mac_address_by_name(&self.interface)?.unwrap().bytes();
        let dmac = self.mac.parse::<MacAddress>()?.bytes();

        let oplist = parse_actions(&self.actions)?;

        smol::block_on(async {
            let mut requ = Into::<MessageBuilder<RegisterRequest>>::into(
                MessageBuilder::<RequestHeader>::new()
                    .destination_address(&dmac)
                    .source_address(&smac)
                    .device_id(self.devid),
            )
            .regops(oplist)
            .build()?;

            // @todo:
            let mut wbuf = Vec::new();
            let len = if requ.wire_size() > 60 {
                requ.wire_size()
            } else {
                60
            };

            wbuf.resize(len, 0);
            let _ = requ.marshal(&mut wbuf[..])?;
            sock.write_with(|mut s| s.write(&wbuf[..])).await?;
            let mut has_response = false;

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
                        has_response = true;
                        let resp: RegisterResponse = message::unmarshal(&rbuf[..sz])?;
                        let mac: MacAddress = resp.header().source_address().into();
                        println!(
                            "Mac:{mac} Devid:0x{:02X}\n {:04X?}",
                            resp.header().device_id(),
                            resp.regops,
                        );
                    }
                    Ok(_) => continue,
                    Err(e) if e.kind() == ErrorKind::TimedOut => {
                        if has_response {
                            break;
                        }

                        return Err(anyhow::anyhow!(
                            "No response: check network or no rmu at mac={},devid=0x{:02X}",
                            self.mac,
                            self.devid
                        ));
                    }
                    Err(e) => return Err(e.into()),
                }
            }

            Ok(())
        })
    }
}
