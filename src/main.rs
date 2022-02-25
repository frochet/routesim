//! Simulating behavioural activity within a Mixnet
//!
//! This tool evaluates the probability of deanonymization through time,
//! assuming some level of adversarial activity among the mixes.
//!
//! We expect to take in input various information to evaluate users' behaviour
//! resistance to deanonymization. Among them, we have: the Mixnet
//! [`topologies`](config::TopologyConfig), an
//! [`histogram`](histogram::Histogram) defining how often the simulated user
//! is interacting with the network (i.e., sending data), and a histogram that
//! gives the distrution of sizes. Messages are packaged within payloads of
//! size [`PAYLOAD_LENGTH`](config::PAYLOAD_LENGTH) and virtually "sent" within
//! the mixnet.
//!
//! The simulator appliess a Monte Carlo method to draw paths and outputs path
//! information for each message sent by each sample. As a matter of example,
//! the "simple" model outputs lines such as:
//! 
//! ```  
//! 1970-01-01 00:44:31 2538 570,260,1007, false
//! ``` 
//!
//! containing the date, the sample id, the path (mix ids) and whether the
//! route is fully compromised or not (i.e., whether the user selected
//! PATH_LENGTH malicious mixes).
//!

mod config;
mod histogram;
mod mailbox;
mod mixnodes;
mod routesim;
mod simplemodel;
mod userasyncmodel;
mod usermodel;

use clap::Parser;
use config::TopologyConfig;
use histogram::Histogram;
use rayon::prelude::*;
use routesim::Runable;
use simplemodel::*;
use std::fs;
use userasyncmodel::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Opts {
    #[clap(
        short,
        long,
        required = true,
        parse(from_os_str),
        help = "Directory containing Network configs containing mixes (and nothing else!)"
    )]
    in_dir: std::path::PathBuf,
    #[clap(
        long,
        parse(from_os_str),
        default_value = "testfiles/timestamps.json",
        help = "timestamps data used to build a histogram"
    )]
    timestamps_h: std::path::PathBuf,
    #[clap(
        long,
        parse(from_os_str),
        default_value = "testfiles/sizes.json",
        help = "Message sizes data used to build a histogram"
    )]
    sizes_h: std::path::PathBuf,
    #[clap(long, default_value = "1", help = "Number of simulated days")]
    days: u32,
    #[clap(
        short,
        long,
        help = "User model for the simulation",
        default_value = "simple"
    )]
    usermod: String,
    #[clap(long, default_value = "5000", help = "Number of users to simulate")]
    users: u32,
    #[clap(
        short,
        long,
        default_value = "86401",
        help = "Validity period for a given topologies"
    )]
    epoch: u32,
    #[clap(
        short,
        long,
        help = "Do we aim to print to console? Printing to console would display
           one route per line"
    )]
    to_console: bool,
    #[clap(short, help = "Do we disable guards?")]
    disable_guards: bool,
    #[clap(short, help = "The number of contacts", default_value = "10")]
    contacts: u32,
}

fn read_entries(path: impl AsRef<std::path::Path>) -> std::io::Result<Vec<std::path::PathBuf>> {
    let paths = fs::read_dir(path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    Ok(paths)
}

fn main() {
    let opts: Opts = Opts::parse();

    let filenames = read_entries(&opts.in_dir).expect("Something went wrong reading paths");
    let mut topologies: Vec<TopologyConfig> = filenames
        .into_par_iter()
        .map(|filename| config::load(filename, opts.users))
        .collect();
    // We need sorting the topologies for accessing the right one depending
    // on the current epoch
    topologies.sort_by(|a, b| a.epoch.cmp(&b.epoch));
    let n = topologies.len();
    
    let mut epoch = opts.epoch;
    // check whether the parameters days; config and epoch make sense
    if epoch * n as u32 <= opts.days * 24 * 60 * 60 {
        eprintln!("Make sure you have enough configuration files, and that the epoch and days value make sense!");
        epoch = 86400*opts.days + 1;
        eprintln!("Setting epoch to {epoch}. Maybe you want to change that");
    }
    // we should sample users in a valid range
    if opts.contacts > opts.users {
        panic!("The number of contacts cannot be higher than the number of samples (users)");
    }

    let mut runner = Runable::new(opts.users, topologies, opts.days, epoch, opts.contacts);

    if !opts.disable_guards {
        runner.with_guards();
    }
    if opts.to_console {
        runner.with_console();
    }

    match &opts.usermod[..] {
        "simple" => {

            // XXX todo! makes sync models accept histograms 
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
            // run the simulation then exit main.
            runner.run(usermodels);
        }
        _ => panic!("We don't have that usermodel: {}", &opts.usermod[..]),
    };
}
