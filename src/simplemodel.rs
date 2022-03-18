use crate::mailbox::MailBox;
use crate::mixnodes::mixnode::Mixnode;
/**
 * A simple user model -- It samples messages within a [5, 15min] interval
 *
 * Currently does not send the message to any simulated user in particular, and it is one message
 * at a time.
 */
use crate::usermodel::{
    AnonModelKind, RequestHandler, UserModel, UserModelInfo, UserRequestIterator,
};
use crossbeam_channel::Receiver;
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
    req_list: Vec<T>,
}

/// This simple model uniformly samples a new message to send in the next [300 ... 900] second
/// interval
impl<'a, T> UserModel<'a> for SimpleSynchronousModel<'a, T>
where
    T: UserRequestIterator + Clone + Ord + PartialOrd + Eq + PartialEq,
{
    type URequest = T;

    fn new(_tot_users: u32, _epoch: u32, uinfo: UserModelInfo<'a, T>) -> Self {
        // initialize the client with guards
        let rng = SmallRng::from_entropy();
        SimpleSynchronousModel {
            rng,
            die: Uniform::from(INTERVAL_MIN..INTERVAL_MAX),
            current_time: 0,
            limit: 0,
            uinfo,
            req_list: Vec::new(),
        }
    }
    fn get_reqlist(&self) -> &Vec<Self::URequest> {
        &self.req_list
    }

    fn get_reqlist_mut(&mut self) -> &mut Vec<Self::URequest> {
        &mut self.req_list
    }

    fn get_guard_for(&self, topo_idx: usize) -> Option<&'a Mixnode> {
        self.uinfo.get_guard_for(topo_idx)
    }

    fn get_userid(&self) -> u32 {
        self.uinfo.get_userid()
    }
    fn get_current_time(&self) -> u64 {
        self.current_time
    }

    fn set_limit(&mut self, limit: u64) {
        self.limit = limit;
    }

    fn get_limit(&self) -> u64 {
        self.limit
    }

    fn model_kind(&self) -> AnonModelKind {
        AnonModelKind::ClientOnly
    }
    /// does not use channels
    fn with_receiver(&mut self, _r: Receiver<T>) -> &mut Self {
        self
    }
    ///// Update any client information (e.g., guards), relative to the current timing
    fn update(&mut self, message_timing: u64) {
        self.uinfo.update(message_timing, &mut self.rng);
    }
    fn build_req(&mut self) -> Option<T> {
        None
    }
}

impl<T> SimpleSynchronousModel<'_, T> {
    /// We simply increase the current time with the sampled value
    fn get_next_message_timing(&mut self) -> u64 {
        self.current_time += self.die.sample(&mut self.rng);
        self.current_time
    }
}

impl<'a, T> RequestHandler for SimpleSynchronousModel<'a, T>
where
    T: UserRequestIterator + Clone + PartialOrd + Ord + PartialEq + Eq,
{
    type Out = (u64, Option<&'a Mixnode>, Option<&'a MailBox>, Option<u128>);

    #[inline]
    fn fetch_next(&mut self) -> Option<Self::Out> {
        let next_timing = self.get_next_message_timing();
        match next_timing {
            currt if currt < self.limit => {
                self.update(currt);
                Some((currt, self.uinfo.get_selected_guard(), None, None))
            }
            _ => None,
        }
    }
}
