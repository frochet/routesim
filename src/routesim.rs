use config::Config;

/// Contains information required for running the simulation
pub struct Runable {
    /// The number of users we want to simulate
    users: u32,
    /// The Network config
    config: Config,
    /// The number of virtual days for running the experiment
    days: u32,
    /// todo
    //usermod: UserModel,
}

impl Runable {
    pub fn new() -> Self {
        Runable { ..Default::default() }
    }
    /// Run the simulation -- this function should output
    /// route taken for each user each time the user requires to send
    /// a message, which depends of the user model through time.
    pub fn run(&self) {

    }
}
