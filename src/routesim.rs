use crate::config::Config;
use crate::usermodel::UserModel;
use rayon::prelude::*;
use rand::prelude::*;
/// Contains information required for running the simulation
#[derive(Default)]
pub struct Runable {
    /// The number of users we want to simulate
    users: u32,
    /// The Network config
    config: Config,
    /// The number of virtual days for running the experiment
    days: u32,
}

impl Runable {
    pub fn new(users: u32, config: Config, days: u32) -> Self {
        Runable {
            config: config,
            users: users,
            days: days,
        }
    }

    fn format_message_timing(timing: u64) -> String {
        let mut datestr: String = "day ".into();
        let mut timing = timing;
        let days_val: u64 = timing / (60*60*24);
        timing -= days_val * 60*60*24;
        let hours_val: u64 = timing / (60*60);
        timing -= hours_val *60*60;
        let mins_val: u64 = timing/60;
        timing -= mins_val * 60;
        datestr.push_str(&format!("{}, {}:{}:{}", days_val, hours_val, mins_val, timing));
        datestr
    }
    fn days_to_timestamp(&self) -> u64 {
        u64::from(self.days)*24*60*60
    }
    /// Run the simulation -- this function should output
    /// route taken for each user each time the user requires to send
    /// a message, which depends of the user model through time.
    pub fn run<T>(&self)
    where
        T: UserModel +
           Iterator<Item = u64>,
    {
        (0..self.users).into_par_iter().for_each(|user| {
            let mut usermodel = T::new();
            let mut rng = thread_rng();
            usermodel.set_limit(self.days_to_timestamp());
            for message_timing in usermodel {
                let path = self.config.sample_path(&mut rng);
                let strdate = Runable::format_message_timing(message_timing);
                // write out the path for this message_timing
                let is_malicious = self.config.is_path_malicious(&mut path.clone());
                println!("{}, {}, {}{}", strdate, user, path.fold(String::new(), |p, hop|
                                                                   p+&hop.mixid.to_string()+","),
                                                                   is_malicious);
            }
        })

    }
}

#[test]
fn test_date_formatting() {
    let mut timing = 60*11;
    let mut strdate = Runable::format_message_timing(timing);
    assert_eq!(strdate, "day 0, 0:11:0");
    timing = timing+1;
    strdate = Runable::format_message_timing(timing);
    assert_eq!(strdate, "day 0, 0:11:1");
    timing = timing + 25*60*60;
    strdate = Runable::format_message_timing(timing);
    assert_eq!(strdate, "day 1, 1:11:1");
}

