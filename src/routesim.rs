use crate::config::TopologyConfig;
use crate::config::PATH_LENGTH;
use crate::mixnodes::mixnode::Mixnode;
use crate::usermodel::*;
use crossbeam_channel::unbounded;
use rand::prelude::*;
use rayon::prelude::*;
use std::vec::IntoIter;

const DAY: u64 = 60 * 60 * 24;
const HOUR: u64 = 60 * 60;

/// Contains information required for running the simulation
#[derive(Default)]
pub struct Runable {
    /// The number of users we want to simulate
    users: u32,
    /// The Network config
    configs: Vec<TopologyConfig>,
    /// The number of virtual days for running the experiment
    days: u32,
    /// Does this simulation use the guard principle?
    use_guards: bool,
    /// each topology lifetime -- we assume this to be unique (e.g., 1 day)
    epoch: u32,
    /// print to console --- default: false
    to_console: bool,
}

impl Runable {
    pub fn new(users: u32, configs: Vec<TopologyConfig>, days: u32, epoch: u32) -> Self {
        Runable {
            configs,
            users,
            days,
            epoch,
            ..Default::default()
        }
    }

    pub fn with_guards(&mut self) -> &mut Self {
        self.use_guards = true;
        self
    }

    pub fn with_console(&mut self) -> &mut Self {
        self.to_console = true;
        self
    }

    #[inline]
    pub fn sample_path<'a>(
        &'a self,
        message_timing: u64,
        rng: &mut ThreadRng,
        guard: Option<&'a Mixnode>,
    ) -> IntoIter<&'a Mixnode> {
        self.configs[(message_timing / self.epoch as u64) as usize].sample_path(rng, guard)
    }

    /// Check whether the three mixnode in path are compromised.
    /// return true if they are, false otherwise.
    pub fn is_path_malicious(&self, path: &[&Mixnode]) -> bool {
        let mut mal_mix = 0;
        for i in 0..PATH_LENGTH {
            if path[i as usize].is_malicious {
                mal_mix += 1;
            }
        }
        mal_mix == PATH_LENGTH
    }

    fn format_message_timing(timing: u64) -> String {
        let mut datestr: String = "day ".into();
        let mut timing = timing;
        let days_val: u64 = timing / DAY;
        timing -= days_val * DAY;
        let hours_val: u64 = timing / HOUR;
        timing -= hours_val * HOUR;
        let mins_val: u64 = timing / 60;
        timing -= mins_val * 60;
        datestr.push_str(&format!(
            "{}, {}:{}:{}",
            days_val, hours_val, mins_val, timing
        ));
        datestr
    }

    #[inline]
    fn days_to_timestamp(&self) -> u64 {
        u64::from(self.days) * 24 * 60 * 60
    }

    #[inline]
    fn log_stdout(&self, user: u32, strdate: &str, path: IntoIter<&Mixnode>, is_malicious: bool) {
        if self.to_console {
            println!(
                "{strdate} {user} {} {is_malicious};",
                path.fold(String::new(), |p, hop| p + &hop.mixid.to_string() + ","),
            );
        } else {
            // does not flush for each path (i.e., println should be one system call per call. This
            // should not).
            print!(
                "{strdate} {user} {} {is_malicious};",
                path.fold(String::new(), |p, hop| p + &hop.mixid.to_string() + ","),
            );
        }
    }

    pub fn init_sync<'a, T, U>(&'a self) -> Vec<T>
    where
        T: UserModel<'a, U>,
    {
        let usermodels: Vec<_> = (0..self.users)
            .map(|user| {
                T::new(
                    self.users,
                    self.epoch,
                    UserModelInfo::new(user, &self.configs, self.epoch, self.use_guards),
                )
            })
            .collect();
        usermodels
    }

    pub fn init<'a, T, U>(&'a self) -> Vec<T>
    where
        T: UserModel<'a, U>,
        U: UserRequestIterator,
    {
        // create first all model info
        // add the mpc channels
        let mut usermodels: Vec<_> = (0..self.users)
            .map(|user| {
                T::new(
                    self.users,
                    self.epoch,
                    UserModelInfo::new(user, &self.configs, self.epoch, self.use_guards),
                )
            })
            .collect();
        for i in 0..self.users {
            // let's create one receiver per user, and give
            // one sender to every other users
            let (s, r) = unbounded();
            usermodels[i as usize].with_receiver(r);
            for j in 0..self.users {
                if j != i {
                    usermodels[j as usize].add_sender(i, s.clone())
                }
            }
            usermodels[i as usize].add_sender(i, s);
        }
        usermodels
    }

    /// Run the simulation -- this function should output
    /// route taken for each user each time the user requires to send
    /// a message, which depends of the user model through time.
    pub fn run<'a, T, U>(&'a self, mut usermodels: Vec<T>)
    where
        T: UserModel<'a, U> + Send,
        U: UserRequestIterator
    {
        // for_each should block until they all completed
        (0..self.users)
            .into_par_iter()
            .zip(&mut usermodels)
            .for_each(|(user, mut usermodel)| {
                let mut rng = thread_rng();
                // move this in the init part?
                usermodel.set_limit(self.days_to_timestamp());
                //let userinfo = &mut userinfos[user as usize];
                for (message_timing, guard) in &mut usermodel {
                    // do we need to update userinfo relative to the current timing?
                    let path = self.sample_path(message_timing, &mut rng, guard);
                    let strdate = Runable::format_message_timing(message_timing);
                    // write out the path for this message_timing
                    let is_malicious = self.is_path_malicious(path.as_slice());
                    self.log_stdout(user, &strdate, path, is_malicious);
                }
                // Drop the senders -- i.e., the receiver should not block when 
                // all messages are read
                if let AnonModelKind::BothPeers = usermodel.model_kind() {
                    usermodel.drop_senders();
                }
            });
        // now check the UserRequests, if any
        usermodels
            .into_par_iter()
            .filter(|usermodel| usermodel.model_kind() == AnonModelKind::BothPeers)
            .for_each(|usermodel| {
                let mut rng = thread_rng();
                //XXX should we parallel iter on the channel recv()?
                while let Some(request) = usermodel.get_request() {
                    // XXX From the request information, fetch the right guard
                    let guard: Option<&'a Mixnode> = usermodel.get_guard_for(request.get_topos_idx() as usize);
                    let user = usermodel.get_userid();
                    for message_timing in request.filter(|t| t < &usermodel.get_limit()) {
                        let path = self.sample_path(message_timing, &mut rng, guard);
                        let strdate = Runable::format_message_timing(message_timing);
                        // write out the path for this message_timing
                        let is_malicious = self.is_path_malicious(path.as_slice());
                        self.log_stdout(user, &strdate, path, is_malicious);
                    }
                }
            })
    }
}

#[test]
fn test_date_formatting() {
    let mut timing = 60 * 11;
    let mut strdate = Runable::format_message_timing(timing);
    assert_eq!(strdate, "day 0, 0:11:0");
    timing = timing + 1;
    strdate = Runable::format_message_timing(timing);
    assert_eq!(strdate, "day 0, 0:11:1");
    timing = timing + 25 * 60 * 60;
    strdate = Runable::format_message_timing(timing);
    assert_eq!(strdate, "day 1, 1:11:1");
}
