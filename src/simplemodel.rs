use crate::config::TopologyConfig;
/**
 * A simple user model -- It samples messages within a [5, 15min] interval
 *
 * Currently does not send the message to any simulated user in particular, and it is one message
 * at a time.
 */
use crate::usermodel::UserModel;
use rand::distributions::{Distribution, Uniform};
use rand::prelude::*;

const INTERVAL_MAX: u64 = 900;

const INTERVAL_MIN: u64 = 300;

pub struct SimpleModel<'a> {
    /// timestamp of current time, starting at 0.
    current_time: u64,
    /// the max value of a timing message
    limit: u64,
    rng: ThreadRng,
    die: Uniform<u64>,
    // Mixnet topology -- should contain all topologies studied in our time period
    #[allow(dead_code)]
    topos: &'a [TopologyConfig],
}

/// This simple model uniformly samples a new message to send in the next [300 ... 900] second
/// interval
impl<'a> UserModel<'a> for SimpleModel<'a> {
    fn new(topos: &'a [TopologyConfig]) -> Self {
        // initialize the client with guards

        let rng = rand::thread_rng();
        SimpleModel {
            rng,
            die: Uniform::from(INTERVAL_MIN..INTERVAL_MAX),
            current_time: 0,
            limit: 0,
            topos,
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

    ///// Update any client information (e.g., guards), relative to the current timing
    //fn update(&mut self) {}
}

impl Iterator for SimpleModel<'_> {
    // "%days, %hh,%mm,%ss
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        // update user information
        //self.update();
        // Draw the next message timing from the distribution we use
        let next_timing = self.get_next_message_timing();
        match next_timing {
            currt if currt < self.limit => Some(currt),
            _ => None,
        }
    }
}
