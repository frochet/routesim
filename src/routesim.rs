use config::Config;
use usermodel::UserModel;
use rayon::prelude::*;
use rand::prelude::*;
/// Contains information required for running the simulation
pub struct Runable {
    /// The number of users we want to simulate
    users: u32,
    /// The Network config
    config: Config,
    /// The number of virtual days for running the experiment
    days: u32,

    outputfilename: String,
}

impl Runable {
    pub fn new() -> Self {
        Runable { ..Default::default() }
    }

    fn format_message_timing(timing: u64) -> String {
        "".to_string()
    }
    fn days_to_timestamp(&self) {
        self.days*24*60*60
    }
    /// Run the simulation -- this function should output
    /// route taken for each user each time the user requires to send
    /// a message, which depends of the user model through time.
    pub fn run<T: UserModel+Iterator+Default>(&self) {
        let days_in_timestamp = self.days_in_timestamp();

        (0..self.users).par_iter().for_each(|user| {
            let usermodel = T::new();
            let mut rng = thread_rng();
            usermodel.set_limit(self.days_in_timestamp(self.days));
            for message_timing in self.usermodel.iter() {
                let path = self.config.sample_path(&mut rng);
                // write out the path for this message_timing
            }
        })

    }
}
