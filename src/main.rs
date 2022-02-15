mod config;
mod histogram;
mod mailbox;
mod mixnodes;
mod routesim;
mod simplemodel;
mod userasyncmodel;
mod usermodel;

use clap::{AppSettings, Clap};
use histogram::Histogram;
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
        long,
        parse(from_os_str),
        default_value = "testfiles/timestamps.json",
        about = "timestamps data used to build a histogram"
    )]
    timestamps_h: std::path::PathBuf,
    #[clap(
        long,
        parse(from_os_str),
        default_value = "testfiles/sizes.json",
        about = "Message sizes data used to build a histogram"
    )]
    sizes_h: std::path::PathBuf,
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
    #[clap(short, about = "The number of contacts", default_value = "10")]
    contacts: u32,
}

fn main() {
    let opts: Opts = Opts::parse();

    let netconf = config::load(opts.filename, opts.users);

    let mut topologies = vec![];
    topologies.push(netconf);
    let n = topologies.len();

    let mut runner = Runable::new(opts.users, topologies, opts.days, opts.epoch, opts.contacts);

    if !opts.disable_guards {
        runner.with_guards();
    }
    if opts.to_console {
        runner.with_console();
    }
    // we should sample users in a valid range
    if opts.contacts > opts.users {
        panic!("The number of contacts cannot be higher than the number of samples (users)");
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
            // try to open timstamps_h and sizes_h. Panic if it fails.
            let timestamps_s =
                std::fs::read_to_string(&opts.timestamps_h).expect("Couldn't open the file");
            let timestamps_h: Histogram = Histogram::from_json(&timestamps_s, 60)
                .expect("Something went wrong while processing the json data");
            let sizes_s = std::fs::read_to_string(&opts.sizes_h).expect("Couldn't open the file");
            let sizes_h: Histogram = Histogram::from_json(&sizes_s, 2048)
                .expect("Something went wrong while processing the json data");
            runner
                .with_timestamps_hist(timestamps_h)
                .with_sizes_hist(sizes_h);
            let usermodels = runner.init::<SimpleEmailModel<UserRequest>, UserRequest>();
            runner.run(usermodels);
        }
        _ => panic!("We don't have that usermodel: {}", &opts.usermod[..]),
    };
}
