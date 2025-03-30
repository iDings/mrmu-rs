use anyhow::Context;
use libc::{sockaddr_ll, sockaddr_storage, socklen_t};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

pub const ETH_P_RMU: libc::c_int = 0x9101;

fn ifindex_of(name: &str) -> anyhow::Result<i32> {
    if name.len() > libc::IFNAMSIZ {
        return Err(anyhow::anyhow!("ifname invalid"));
    }
    let mut buf = [0u8; libc::IFNAMSIZ];
    buf[..name.len()].copy_from_slice(name.as_bytes());
    let idx = unsafe { libc::if_nametoindex(buf.as_ptr() as *const libc::c_char) };
    if idx == 0 {
        // return Err(anyhow::anyhow!(std::error::Error:last_os_error()));
        return Err(anyhow::anyhow!("nametoindex fail"));
    }

    Ok(idx as i32)
}

// @todo
pub fn create_rmu_sock(interface: &str) -> anyhow::Result<smol::Async<Socket>> {
    let sk = Socket::new(
        Domain::from(libc::AF_PACKET),
        Type::RAW,
        Some(Protocol::from(ETH_P_RMU.to_be())),
    )
    .context("create raw sock fail")?;

    unsafe {
        // @bugs: zeroed not zero all bytes?
        let mut ss: sockaddr_storage = std::mem::zeroed();
        let ssl = &mut *std::mem::transmute::<*mut sockaddr_storage, *mut sockaddr_ll>(&mut ss);
        ssl.sll_family = libc::AF_PACKET as u16;
        ssl.sll_protocol = (ETH_P_RMU as u16).to_be();
        ssl.sll_ifindex = ifindex_of(interface)?;
        ssl.sll_pkttype = 0;
        ssl.sll_hatype = 0;
        ssl.sll_halen = 0;
        ssl.sll_addr = [0; 8];

        let len = std::mem::size_of::<sockaddr_ll>() as socklen_t;
        let sockaddr = SockAddr::new(ss, len);
        sk.bind(&sockaddr).unwrap();
    }

    Ok(smol::Async::new(sk)?)
}
