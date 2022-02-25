use crate::config::TopologyConfig;
use crate::config::PATH_LENGTH;
use crate::histogram::Histogram;
use crate::mailbox::MailBox;
use crate::mixnodes::mixnode::Mixnode;
use crate::usermodel::*;
use chrono::NaiveDateTime;
use crossbeam_channel::unbounded;
use crossbeam_channel::Sender;
use rand::distributions::Uniform;
use rand::prelude::*;
use rayon::prelude::*;
use std::vec::IntoIter;

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
    /// Timestamps histogram
    timestamps_h: Option<Histogram>,
    /// Sizes histogram
    sizes_h: Option<Histogram>,
    /// The number of contact each sample has
    contacts: u32,
}

impl Runable {
    /// Creates a new simulation to run.
    pub fn new(
        users: u32,
        configs: Vec<TopologyConfig>,
        days: u32,
        epoch: u32,
        contacts: u32,
    ) -> Self {
        Runable {
            configs,
            users,
            days,
            epoch,
            contacts,
            ..Default::default()
        }
    }
    /// Do we enable guards for this simulation?
    pub fn with_guards(&mut self) -> &mut Self {
        self.use_guards = true;
        self
    }
    /// Do we print results to consol?
    pub fn with_console(&mut self) -> &mut Self {
        self.to_console = true;
        self
    }
    /// Do we use a timestamp histogram?
    pub fn with_timestamps_hist(&mut self, h: Histogram) -> &mut Self {
        self.timestamps_h = Some(h);
        self
    }

    pub fn with_sizes_hist(&mut self, h: Histogram) -> &mut Self {
        self.sizes_h = Some(h);
        self
    }
    /// Get a random path from the right mixnet configuration.
    ///
    /// A guard is optionally given. Guards are a set of mixnet nodes chosen to change as few as
    /// possible
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
    /// Returns true if they are, false otherwise.
    pub fn is_path_malicious(&self, path: &[&Mixnode], mailbox: Option<&MailBox>) -> bool {
        let mut mal_mix = 0;
        for i in 0..PATH_LENGTH {
            if path[i as usize].is_malicious {
                mal_mix += 1;
            }
        }
        if let Some(extendedhop) = mailbox {
            if extendedhop.is_malicious {
                mal_mix += 1
            }
            mal_mix == (PATH_LENGTH + 1)
        } else {
            mal_mix == PATH_LENGTH
        }
    }

    /// Format the message's sending time as a naive time
    #[inline]
    fn format_message_timing(timing: u64) -> String {
        let dt = NaiveDateTime::from_timestamp(timing as i64, 0);
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    #[inline]
    pub fn days_to_timestamp(&self) -> u64 {
        u64::from(self.days) * 24 * 60 * 60
    }

    #[inline]
    fn log_stdout(
        &self,
        user: u32,
        strdate: &str,
        path: IntoIter<&Mixnode>,
        is_malicious: bool,
        mailbox: Option<&MailBox>,
        requestid: Option<u128>,
        line_count: &mut u32,
    ) {
        let mut log: String;
        if let Some(rid) = requestid {
            log = format!(
                "{strdate} {user} {rid} {}",
                path.fold(String::new(), |p, hop| p + &hop.mixid.to_string() + ",")
            );
        } else {
            log = format!(
                "{strdate} {user} {}",
                path.fold(String::new(), |p, hop| p + &hop.mixid.to_string() + ",")
            );
        }
        if let Some(mailbox) = mailbox {
            let mixid = mailbox.mixid;
            log.push_str(&format!("{mixid}"));
        }
        if self.to_console {
            log.push_str(&format!(" {is_malicious};"));
            println!("{log}");
        } else {
            // does not flush for each path (i.e., println should be one system call per call. This
            // should not).
            if *line_count == 10000 {
                log.push_str(&format!(" {is_malicious}\n"));
                *line_count = 0;
            }
            else {
                log.push_str(&format!(" {is_malicious};"));
            }
            print!("{log}");
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
        let die = Uniform::from(0..self.users);
        let mut usermodels: Vec<_> = (0..self.users)
            .map(|user| {
                let mut model = T::new(
                    self.users,
                    self.epoch,
                    UserModelInfo::new(user, &self.configs, self.epoch, self.use_guards),
                );
                // add a reference to the histogram for sampling
                model.set_contacts(self.contacts, &die);
                model.with_size_sampler(&self.sizes_h.as_ref().unwrap());
                model.with_timestamp_sampler(&self.timestamps_h.as_ref().unwrap());
                model
            })
            .collect();
        let mut senders: Vec<Sender<U>> = Vec::with_capacity(self.users as usize);
        for i in 0..self.users {
            // let's create one receiver per user, and give
            // one sender to every other users
            let (s, r) = unbounded();
            senders.push(s);
            usermodels[i as usize].with_receiver(r);
        }
        for i in 0..self.users {
            let contacts: Vec<u32> = usermodels[i as usize]
                .get_contacts()
                .unwrap()
                .iter()
                .map(|c| *c)
                .collect();
            contacts.iter().for_each(|j| {
                usermodels[i as usize].add_sender(*j, senders[*j as usize].clone());
            });
            usermodels[i as usize].add_sender(i, senders[i as usize].clone());
        }
        usermodels
    }

    /// Run the simulation -- this function should output
    /// route taken for each user each time the user requires to send
    /// a message, which depends of the user model through time.
    pub fn run<'a, T, U>(&'a self, mut usermodels: Vec<T>)
    where
        T: UserModel<'a, U> + Send,
        U: UserRequestIterator,
    {
        // for_each should block until they all completed
        (0..self.users)
            .into_par_iter()
            .zip(&mut usermodels)
            .for_each(|(user, mut usermodel)| {
                let mut rng = thread_rng();
                let mut line_count: u32 = 0;
                // move this in the init part?
                usermodel.set_limit(self.days_to_timestamp());
                //let userinfo = &mut userinfos[user as usize];
                for (message_timing, guard, mailbox, requestid) in &mut usermodel {
                    // do we need to update userinfo relative to the current timing?
                    let path = self.sample_path(message_timing, &mut rng, guard);
                    let strdate = Runable::format_message_timing(message_timing);
                    // write out the path for this message_timing
                    let is_malicious = self.is_path_malicious(path.as_slice(), mailbox);
                    line_count += 1;
                    self.log_stdout(user, &strdate, path, is_malicious, mailbox, requestid, &mut line_count);
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
                let mut line_count: u32 = 0;
                //XXX should we parallel iter on the channel recv()?
                while let Some(request) = usermodel.get_request() {
                    // XXX From the request information, fetch the right guard
                    let guard: Option<&'a Mixnode> =
                        usermodel.get_guard_for(request.get_topos_idx() as usize);
                    // fetch the message over the sendbox
                    let mailbox = usermodel.get_mailbox(request.get_topos_idx() as usize);
                    let user = usermodel.get_userid();
                    let requestid = request.get_requestid();
                    for message_timing in request.filter(|t| t < &usermodel.get_limit()) {
                        let path = self.sample_path(message_timing, &mut rng, guard);
                        let strdate = Runable::format_message_timing(message_timing);
                        // write out the path for this message_timing
                        let is_malicious = self.is_path_malicious(path.as_slice(), mailbox);
                        line_count += 1;
                        self.log_stdout(
                            user,
                            &strdate,
                            path,
                            is_malicious,
                            mailbox,
                            Some(requestid),
                            &mut line_count,
                        );
                    }
                }
            })
    }
}

#[test]
fn test_date_formatting() {
    let mut timing = 60 * 11;
    let mut strdate = Runable::format_message_timing(timing);
    assert_eq!(strdate, "1970-01-01 00:11:00");
    timing = timing + 1;
    strdate = Runable::format_message_timing(timing);
    assert_eq!(strdate, "1970-01-01 00:11:01");
    timing = timing + 25 * 60 * 60;
    strdate = Runable::format_message_timing(timing);
    assert_eq!(strdate, "1970-01-02 01:11:01");
}
