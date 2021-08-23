mod config;
mod routesim;
mod mixnodes;
mod usermodel;
use usermodel::*;
use routesim::Runable;
use clap::{AppSettings, Clap};

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {

    #[clap(short, long, required=true, about="Network config containing mixes")]
    filename: String,
    #[clap(short, long, default_value="10", about="Number of simulated days")]
    days: u32,
    #[clap(short, long, about="User model for the simulation", default_value="simple")]
    usermod: String,
    #[clap(long, default_value="5000", about="Number of users to simulate")]
    users: u32,
}


fn main() {
    let opts: Opts = Opts::parse();
    
    let netconf = config::load(opts.filename);

    let runner = Runable::new(opts.users, netconf, opts.days);
    match &opts.usermod[..] {
        "simple" => runner.run::<SimpleModel>(),
        _ => println!("We don't have that usermodel"),
    };
}
