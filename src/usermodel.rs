/**
* Simple trait definition for any concrete user model
*/
use crate::config::TopologyConfig;
use crate::config::{GUARDS_LAYER, GUARDS_SAMPLE_SIZE, GUARDS_SAMPLE_SIZE_EXTEND};
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
    #[allow(dead_code)]
    userid: u32,
    /// Mixnet topology
    topos: &'a [TopologyConfig],
    /// Guards information
    guards: Option<Vec<&'a Mixnode>>,
    /// The guard we're currently using
    selected_guard: Option<&'a Mixnode>,

    rng: ThreadRng,

    curr_idx: usize,
}

impl<'a> UserModelInfo<'a> {
    pub fn new(userid: u32, topos: &'a [TopologyConfig], use_guards: bool) -> Self {
        let mut rng = rand::thread_rng();
        let mut guards: Option<Vec<&'a Mixnode>> = None;
        let mut selected_guard: Option<&'a Mixnode> = None;
        if use_guards {
            guards = Some(
                topos[0]
                    .sample_guards(GUARDS_LAYER, GUARDS_SAMPLE_SIZE, &mut rng)
                    .collect(),
            );
            selected_guard = Some(guards.as_ref().unwrap()[0]);
        }
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
    pub fn get_selected_guard(&self) -> Option<&'a Mixnode> {
        self.selected_guard
    }

    #[allow(dead_code)]
    #[inline]
    pub fn get_guards(&self) -> Option<&[&Mixnode]> {
        match &self.guards {
            Some(guardlist) => Some(guardlist.as_ref()),
            None => None,
        }
    }

    /// If the selected guard is not online, it should be within the unselected mixpool.
    /// so, if the guard is online, it should not be in the unselected pool
    #[inline]
    fn is_guard_online(&self, topoidx: usize, mixid: u32) -> bool {
        self.topos[topoidx].unselected().get(&mixid).is_none()
    }
    /// Potentially changes this user guards
    #[inline]
    pub fn update(&mut self, message_timing: u64, epoch: u32) {
        let idx = (message_timing / epoch as u64) as usize;
        if idx > self.curr_idx && self.guards.is_some() {
            // okaay there's a potential update to do.
            self.curr_idx = idx as usize;
            // if our selected guards is still online, do nothing
            let mut guard_iter = self
                .guards
                .as_ref()
                .unwrap()
                .iter()
                .skip_while(|guard| !self.is_guard_online(self.curr_idx, guard.mixid))
                .take(1);
            match guard_iter.next() {
                Some(guard) => self.selected_guard = Some(guard),
                // We need to extend the guard list
                None => {
                    // this should be the idx to take a selected guard after we extend
                    // the guard list
                    match &mut self.guards {
                        Some(guards) => {
                            let guard_idx = guards.len();
                            guards.extend(self.topos[self.curr_idx].sample_guards(
                                GUARDS_LAYER,
                                GUARDS_SAMPLE_SIZE_EXTEND,
                                &mut self.rng,
                            ));
                            // some checks
                            if guards.len() <= guard_idx {
                                panic!(
                                    "Did the guard len got properly extend? len: {}",
                                    guards.len()
                                );
                            }
                            // remember the selected guard
                            self.selected_guard = Some(guards[guard_idx]);
                        }
                        _ => panic!("guards aren't expected to be None"),
                    }
                }
            }
        }
    }
}
