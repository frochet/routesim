use crate::mixnodes::mixnode::Mixnode;
/**
 * A simple user model -- It samples messages within a [5, 15min] interval
 *
 * Currently does not send the message to any simulated user in particular, and it is one message
 * at a time.
 */
use crate::usermodel::{AnonModelKind, UserModel, UserModelInfo};
use crossbeam_channel::{Receiver, Sender};
use rand::distributions::{Distribution, Uniform};
use rand::rngs::SmallRng;
use rand::SeedableRng;

const INTERVAL_MAX: u64 = 900;

const INTERVAL_MIN: u64 = 300;

pub struct SimpleSynchronousModel<'a, T> {
    /// timestamp of current time, starting at 0.
    current_time: u64,
    /// the max value of a timing message
    limit: u64,
    rng: SmallRng,
    die: Uniform<u64>,
    uinfo: UserModelInfo<'a, T>,
}

/// This simple model uniformly samples a new message to send in the next [300 ... 900] second
/// interval
impl<'a, T> UserModel<'a, T> for SimpleSynchronousModel<'a, T> {
    fn new(tot_users: u32, uinfo: UserModelInfo<'a, T>) -> Self {
        // initialize the client with guards

        let rng = SmallRng::from_entropy();
        SimpleSynchronousModel {
            rng,
            die: Uniform::from(INTERVAL_MIN..INTERVAL_MAX),
            current_time: 0,
            limit: 0,
            uinfo,
        }
    }
    /// We simply increase the current time with the sampled value
    fn get_next_message_timing(&mut self) -> u64 {
        self.current_time += self.die.sample(&mut self.rng);
        self.current_time
    }

    fn get_current_time(&self) -> u64 {
        self.current_time
    }

    fn set_limit(&mut self, limit: u64) {
        self.limit = limit;
    }

    fn model_kind(&self) -> AnonModelKind {
        AnonModelKind::ClientOnly
    }
    /// does not use channels
    fn with_receiver(&mut self, _r: Receiver<T>) -> &mut Self {
        self
    }
    /// does not use channels
    fn add_sender(&mut self, user: u32, s: Sender<T>) {}

    ///// Update any client information (e.g., guards), relative to the current timing
    fn update(&mut self, message_timing: u64) {
        self.uinfo.update(message_timing, &mut self.rng);
    }
}

impl<'a, T> Iterator for SimpleSynchronousModel<'a, T> {
    // "%days, %hh,%mm,%ss
    type Item = (u64, Option<&'a Mixnode>);

    fn next(&mut self) -> Option<Self::Item> {
        // update user information
        // Draw the next message timing from the distribution we use
        let next_timing = self.get_next_message_timing();
        match next_timing {
            currt if currt < self.limit => {
                self.update(currt);
                Some((currt, self.uinfo.get_selected_guard()))
            }
            _ => None,
        }
    }
}
