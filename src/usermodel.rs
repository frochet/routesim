/**
* Simple trait definition for any concrete user model
*/
use crate::config::TopologyConfig;
use crate::config::GUARDS_LAYER;
use crate::mixnodes::mixnode::Mixnode;
use rand::prelude::*;

pub trait UserModel<'a> {
    fn new(topo: &'a [TopologyConfig]) -> Self;
    /// Sample the next message timing for this
    /// user model
    fn get_current_time(&self) -> u64;
    fn set_limit(&mut self, limit: u64);
    fn get_next_message_timing(&mut self) -> u64;
    /// update the client according to the current timing and the network
    /// topology
    fn update(&mut self);
}

// + things potentially common to any user model
//
//

pub struct UserModelInfo<'a> {
    userid: u32,
    // Mixnet topology
    topos: &'a [TopologyConfig],
    // Guards information
    guards: Vec<&'a Mixnode>,

    rng: ThreadRng,
}

impl<'a> UserModelInfo<'a> {
    pub fn new(userid: u32, topos: &'a [TopologyConfig]) -> Self {
        let mut rng = rand::thread_rng();
        let guards = topos[0].sample_guards(GUARDS_LAYER, &mut rng).collect();
        UserModelInfo {
            userid,
            topos,
            guards,
            rng,
        }
    }

    pub fn get_guards(&self) -> &[&Mixnode] {
        self.guards.as_ref()
    }
    /// Potentially changes this user guards
    pub fn update(&mut self, message_timing: u64, epoch: u32) {
        //let topo = self.topos[(message_timing/self.epoch as u64) as usize].are_guards_online()
    }
}
