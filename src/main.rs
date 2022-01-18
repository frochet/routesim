mod config;
mod mixnodes;
mod routesim;
mod simplemodel;
mod usermodel;

use clap::{AppSettings, Clap};
use routesim::Runable;
use simplemodel::*;

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(
        short,
        long,
        required = true,
        about = "Network config containing mixes"
    )]
    filename: String,
    #[clap(short, long, default_value = "1", about = "Number of simulated days")]
    days: u32,
    #[clap(
        short,
        long,
        about = "User model for the simulation",
        default_value = "simple"
    )]
    usermod: String,
    #[clap(long, default_value = "5000", about = "Number of users to simulate")]
    users: u32,

    #[clap(
        short,
        long,
        default_value = "86401",
        about = "Validity period for a given topologies"
    )]
    epoch: u32,
    #[clap(short, long, about = "Do we aim to print to console?")]
    to_console: bool,
}

fn main() {
    let opts: Opts = Opts::parse();

    let netconf = config::load(opts.filename);

    let mut topologies = vec![];
    topologies.push(netconf);
    let n = topologies.len();

    let mut runner = Runable::new(opts.users, topologies, opts.days, opts.epoch);
    runner.with_guards();

    if opts.to_console {
        runner.with_console();
    }

    // check whether the parameters days; config and epoch make sense
    // panic otherwise.
    if opts.epoch * n as u32 <= opts.days * 24 * 60 * 60 {
        panic!("Make sure you have enough configuration files, and that the epoch and days value make sense!")
    }

    match &opts.usermod[..] {
        "simple" => runner.run::<SimpleModel>(),
        _ => panic!("We don't have that usermodel: {}", &opts.usermod[..]),
    };
}
