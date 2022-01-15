mod config;
mod mixnodes;
mod routesim;
mod usermodel;
mod simplemodel;

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
    #[clap(short, long, default_value = "10", about = "Number of simulated days")]
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

    #[clap(long, default_value = "86400", about = "Validity period for a given topologies")]
    epoch: u32,
}

fn main() {
    let opts: Opts = Opts::parse();

    let netconf = config::load(opts.filename);

    let mut topologies = vec![];
    topologies.push(netconf);

    let mut runner = Runable::new(opts.users, topologies, opts.days, opts.epoch);
    runner.with_guards();

    match &opts.usermod[..] {
        "simple" => runner.run::<SimpleModel>(),
        _ => println!("We don't have that usermodel"),
    };
}
