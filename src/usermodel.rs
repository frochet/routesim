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
    ///// update the client according to the current timing and the network
    ///// topology
    //fn update(&mut self);
}

// + things potentially common to any user model
pub struct UserModelInfo<'a> {
    userid: u32,
    /// Mixnet topology
    topos: &'a [TopologyConfig],
    /// Guards information
    guards: Vec<&'a Mixnode>,
    /// The guard we're currently using
    selected_guard: &'a Mixnode,

    rng: ThreadRng,

    curr_idx: usize,
}

impl<'a> UserModelInfo<'a> {
    pub fn new(userid: u32, topos: &'a [TopologyConfig]) -> Self {
        let mut rng = rand::thread_rng();
        let guards: Vec<&'a Mixnode> = topos[0].sample_guards(GUARDS_LAYER, &mut rng).collect();
        let selected_guard: &'a Mixnode = guards[0];
        UserModelInfo {
            userid,
            topos,
            guards,
            selected_guard,
            rng,
            curr_idx: 0,
        }
    }

    #[inline]
    pub fn get_guards(&self) -> &[&Mixnode] {
        self.guards.as_ref()
    }

    /// If the selected guard is not online, it should be within the unselected mixpool.
    /// so, if the guard is online, it should not be in the unselected pool
    #[inline]
    fn is_selected_guard_online(&self, topoidx: usize) -> bool {
        self.topos[topoidx]
            .unselected()
            .get(&self.selected_guard.mixid)
            .is_none()
    }
    /// check whether at least
    fn is_some_guard_online(&self) -> bool {
        true
    }
    /// Potentially changes this user guards
    #[inline]
    pub fn update(&mut self, message_timing: u64, epoch: u32) {
        let idx = (message_timing / epoch as u64) as usize;
        if idx > self.curr_idx {
            // okaay there's a potential update to do.
            self.curr_idx = idx as usize;
            //let topo = self.topos[self.curr_idx].are_guards_online()
        }
    }
}
