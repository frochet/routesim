/**
 * A simple user model -- It samples messages within a [5, 15min] interval
 *
 * Currently does not send the message to any simulated user in particular, and it is one message
 * at a time.
 */

use crate::usermodel::UserModel;
use rand::distributions::{Distribution, Uniform};
use rand::prelude::*;

pub struct SimpleModel {
    /// timestamp of current time, starting at 0.
    current_time: u64,
    /// the max value of a timing message
    limit: u64,
    rng: ThreadRng,
    die: Uniform<u64>,
}

/// This simple model uniformly samples a new message to send in the next [300 ... 900] second
/// interval
impl UserModel for SimpleModel {
    fn new() -> Self {
        SimpleModel {
            rng: rand::thread_rng(),
            die: Uniform::from(300..900),
            current_time: 0,
            limit: 0,
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
}

/// TODO XXX implement this as a macro
impl Iterator for SimpleModel {
    // "%days, %hh,%mm,%ss
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        // Draw the next message timing from the distribution we use
        match self.current_time {
            currt if currt < self.limit => Some(self.get_next_message_timing()),
            _ => None,
        }
    }
}
