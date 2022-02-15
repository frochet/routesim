/**
* Simple trait definition for any concrete user model
*/
use crate::config::TopologyConfig;
use crate::config::{GUARDS_LAYER, GUARDS_SAMPLE_SIZE, GUARDS_SAMPLE_SIZE_EXTEND};
use crate::histogram::Histogram;
use crate::mailbox::MailBox;
use crate::mixnodes::mixnode::Mixnode;
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use rand::distributions::Uniform;
use rand::prelude::*;
use rustc_hash::FxHashMap as HashMap;
use std::hash::Hasher;

#[derive(PartialEq)]
pub enum AnonModelKind {
    ClientOnly,
    BothPeers,
}

pub trait UserModel<'a, T>:
    Iterator<Item = (u64, Option<&'a Mixnode>, Option<&'a MailBox>, Option<u64>)>
{
    fn new(tot_users: u32, epoch: u32, uinfo: UserModelInfo<'a, T>) -> Self;
    /// Sample the next message timing for this
    /// user model
    fn get_current_time(&self) -> u64;
    fn get_guard_for(&self, topo_idx: usize) -> Option<&'a Mixnode>;
    fn get_userid(&self) -> u32;
    fn get_limit(&self) -> u64;
    fn get_next_message_timing(&mut self) -> u64;
    fn get_request(&self) -> Option<T> {
        None
    }
    fn get_mailbox(&self, _topo_idx: usize) -> Option<&'a MailBox> {
        None
    }
    fn set_contacts(&mut self, _contacts: u32, _die: &Uniform<u32>) {}
    fn get_contacts(&self) -> Option<&[u32]> {None}
    fn set_limit(&mut self, limit: u64);
    fn model_kind(&self) -> AnonModelKind;
    fn with_receiver(&mut self, r: Receiver<T>) -> &mut Self;
    fn with_timestamp_sampler(&mut self, _timestamp_sampler: &'a Histogram) -> &mut Self {
        self
    }
    fn with_size_sampler(&mut self, _size_sampler: &'a Histogram) -> &mut Self {
        self
    }
    fn add_sender(&mut self, _user: u32, _s: Sender<T>) {}
    fn drop_senders(&mut self) {}

    ///// update the client according to the current timing and the network
    ///// topology
    fn update(&mut self, message_timing: u64);
}

/// This iterator is aimed to be from one user to another to make them fetch
/// the data sent to them.
///
/// Should have a total message size and takes a bandwidth in input.
///
/// The iterator should start from the request's timing + some delay, and make sure
/// it does not go over the limit
pub trait UserRequestIterator: Iterator<Item = u64> {
    type RequestTime: Ord + PartialOrd + Eq + PartialEq;
    type RequestSize;

    fn new<H: Hasher>(
        state: &mut H,
        request_time: u64,
        request_size: isize,
        peers: (u32, u32),
        topo_idx: u16,
    ) -> Self;

    fn get_peers(&self) -> (u32, u32);

    fn get_request_size(&self) -> Self::RequestSize;

    fn get_request_time(&self) -> Self::RequestTime;

    fn get_topos_idx(&self) -> u16;

    fn get_requestid(&self) -> u64;

    fn next_with_bandwidth(&mut self, bandwidth: Option<u32>) -> Option<u64>;
}

// XXX &Mixnode or mixid? in this case it is ok to hold &Mixnode reference to a vec within the
// config. Past the initialization, we will never change the config, and it would appear immutable
// everywhere, making usage of a reference to such a vector safe.  However, is that still a good
// pattern? We could simply record the mixid instead of the reference -- that would however cost a
// lookup to retrieve the &Mixnode when we need it.

pub struct UserModelInfo<'a, T> {
    #[allow(dead_code)]
    userid: u32,
    /// Mixnet topology
    topos: &'a [TopologyConfig],
    /// Guards information
    guards: Option<Vec<&'a Mixnode>>,
    /// The guard we're currently using
    selected_guard: Option<&'a Mixnode>,
    /// To tell user i about some data they need to fetch
    senders: HashMap<u32, Sender<T>>,
    /// To receive a request to fetch some data -- potentially from any other user; used in
    /// asynchronous scenarios where both ends require anonymity
    receiver: Option<Receiver<T>>,
    /// epoch length in seconds
    epoch: u32,
    /// contacts,
    pub contacts_list: Vec<u32>,

    curr_idx: usize,
}

impl<'a, T> UserModelInfo<'a, T> {
    pub fn new(userid: u32, topos: &'a [TopologyConfig], epoch: u32, use_guards: bool) -> Self {
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
            senders: HashMap::default(),
            receiver: None,
            epoch,
            contacts_list: Vec::new(),
            curr_idx: 0,
        }
    }

    pub fn get_mailbox(&self, topo_idx: usize) -> &'a MailBox {
        match self.topos.get(topo_idx) {
            Some(topo) => topo.get_mailbox(self.userid).unwrap(),
            None => panic!("BUG: No configs at idx {}", topo_idx),
        }
    }

    pub fn get_userid(&self) -> u32 {
        self.userid
    }

    pub fn with_receiver(&mut self, r: Receiver<T>) -> &mut Self {
        self.receiver = Some(r);
        self
    }

    #[inline]
    pub fn get_guard_for(&self, topo_idx: usize) -> Option<&'a Mixnode> {
        match self.guards.as_ref() {
            Some(v_guards) => {
                match v_guards
                    .iter()
                    .skip_while(|guard| !self.is_guard_online(topo_idx, guard.mixid))
                    .take(1)
                    .next()
                {
                    Some(guard) => Some(*guard),
                    None => {
                        // No guard online
                        None
                    }
                }
            }
            None => None,
        }
    }

    pub fn get_request(&self) -> Option<T> {
        if let Some(recv) = &self.receiver {
            match recv.try_recv() {
                Ok(req) => Some(req),
                Err(TryRecvError::Empty) => None,
                Err(e) => {
                    panic!("We received an error that shouldn't happen: {}", e);
                }
            }
        } else {
            None
        }
    }

    pub fn add_sender(&mut self, user: u32, sender: Sender<T>) {
        self.senders.insert(user, sender);
    }

    pub fn drop_senders(&mut self) {
        drop(&self.senders)
    }

    pub fn send_request(&self, req: T) -> Result<(), crossbeam_channel::SendError<T>>
    where
        T: UserRequestIterator,
    {
        match self.senders.get(&req.get_peers().1) {
            None => panic!(
                "BUG: User {} is Missing sender {}; senders: {:?}",
                req.get_peers().0,
                req.get_peers().1,
                self.senders
            ),
            Some(sender) => sender.send(req),
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
    pub fn update<R: Rng + ?Sized>(&mut self, message_timing: u64, rng: &mut R) {
        let idx = (message_timing / self.epoch as u64) as usize;
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
                // We have an online guard. So be it.
                Some(guard) => self.selected_guard = Some(guard),
                // We have no online guard. We need to extend the guard list
                None => {
                    // this should be the idx to take a selected guard after we extend
                    // the guard list
                    match &mut self.guards {
                        Some(guards) => {
                            let guard_idx = guards.len();
                            guards.extend(self.topos[self.curr_idx].sample_guards(
                                GUARDS_LAYER,
                                GUARDS_SAMPLE_SIZE_EXTEND,
                                rng,
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
