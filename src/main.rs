mod config;
mod histogram;
mod mailbox;
mod mixnodes;
mod routesim;
mod simplemodel;
mod userasyncmodel;
mod usermodel;

use clap::{AppSettings, Clap};
use routesim::Runable;
use simplemodel::*;
use userasyncmodel::*;

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(
        short,
        long,
        required = true,
        parse(from_os_str),
        about = "Network config containing mixes"
    )]
    filename: std::path::PathBuf,
    #[clap(
        short,
        long,
        parse(from_os_str),
        about = "timestamps data used to build a histogram"
    )]
    timestamps_h: Option<std::path::PathBuf>,
    #[clap(
        short,
        long,
        parse(from_os_str),
        about = "Message sizes data used to build a histogram"
    )]
    sizes_h: Option<std::path::PathBuf>,
    #[clap(long, default_value = "1", about = "Number of simulated days")]
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
    #[clap(
        short,
        long,
        about = "Do we aim to print to console? Printing to console would display
           one route per line"
    )]
    to_console: bool,
    #[clap(short, about = "Do we desable guards?")]
    disable_guards: bool,
}

fn main() {
    let opts: Opts = Opts::parse();

    let netconf = config::load(opts.filename, opts.users);

    let mut topologies = vec![];
    topologies.push(netconf);
    let n = topologies.len();

    let mut runner = Runable::new(opts.users, topologies, opts.days, opts.epoch);

    if !opts.disable_guards {
        runner.with_guards();
    }

    if opts.to_console {
        runner.with_console();
    }

    // check whether the parameters days; config and epoch make sense
    // panic otherwise.
    if opts.epoch * n as u32 <= opts.days * 24 * 60 * 60 {
        panic!("Make sure you have enough configuration files, and that the epoch and days value make sense!")
    }

    match &opts.usermod[..] {
        "simple" => {
            let usermodels = runner.init_sync::<SimpleSynchronousModel<UserRequest>, UserRequest>();
            runner.run(usermodels);
        }
        "email" => {
            let usermodels = runner.init::<SimpleEmailModel<UserRequest>, UserRequest, std::path::PathBuf>(opts.timestamps_h, opts.sizes_h);
            runner.run(usermodels);
        }
        _ => panic!("We don't have that usermodel: {}", &opts.usermod[..]),
    };
}
