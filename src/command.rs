mod customer_info_read;
mod fw_version_get;
mod read_atu;
mod read_port;
mod read_vtu;
mod regop;
mod scan;
mod verinfo;
mod version_read;

use customer_info_read::CustomerInfoReadCmd;
use fw_version_get::FwVersionGetCmd;
use read_atu::ReadAtuCmd;
use read_port::ReadPortRegCmd;
use read_vtu::ReadVtuCmd;
use regop::RegOpCmd;
use scan::ScanCmd;
use verinfo::SoftwareInfoCmd;
use version_read::VersionReadCmd;

use clap::Subcommand;

// @todo: impl future
#[derive(Subcommand, Debug)]
pub enum Commands {
    Scan(ScanCmd),
    VersionRead(VersionReadCmd),
    CustomerInfoRead(CustomerInfoReadCmd),
    FwVersionGet(FwVersionGetCmd),
    SoftwareInfo(SoftwareInfoCmd),
    Regop(RegOpCmd),
    ReadAtu(ReadAtuCmd),
    ReadVtu(ReadVtuCmd),
    ReadPort(ReadPortRegCmd),
}

// @todo: future poll api
pub trait CommandOperation {
    fn process(&self) -> anyhow::Result<()>;
}

const RMU_MULTICAST_ADDR: [u8; 6] = [0x01, 0x50, 0x43, 0x00, 0x00, 0x03];

impl CommandOperation for Commands {
    fn process(&self) -> anyhow::Result<()> {
        match self {
            Commands::Scan(m) => m.process(),
            Commands::VersionRead(m) => m.process(),
            Commands::CustomerInfoRead(m) => m.process(),
            Commands::FwVersionGet(m) => m.process(),
            Commands::SoftwareInfo(m) => m.process(),
            Commands::Regop(m) => m.process(),
            Commands::ReadAtu(m) => m.process(),
            Commands::ReadVtu(m) => m.process(),
            Commands::ReadPort(m) => m.process(),
        }
    }
}
