#![allow(dead_code)]
#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

mod command;
mod message;
mod message_builder;
mod message_code;
mod packet_sock;
mod reginfo;

use clap::Parser;

use command::{CommandOperation, Commands};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.cmd.process()
}
