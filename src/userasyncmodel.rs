use crate::histogram::Histogram;
use crate::mailbox::MailBox;
/**
 * This is expected to contain a generic model for asynchronous message sending and fetching
 *
 * E.g., sending email-like data within a mixnet; or chat messages
 */
use crate::mixnodes::mixnode::Mixnode;
use crate::usermodel::*;
use crossbeam_channel::{Receiver, Sender};
use rand::distributions::{Distribution, Uniform};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct SimpleEmailModel<'a, T> {
    tot_users: u32,

    current_time: u64,
    /// list of requests for the current period.
    req_list: Vec<T>,
    /// request being considered now.
    current_req: Option<T>,

    uinfo: UserModelInfo<'a, T>,

    timestamp_sampler: Option<&'a Histogram>,

    size_sampler: Option<&'a Histogram>,

    contact_sampler: Option<Uniform<u32>>,

    limit: u64,

    epoch: u32,

    rng: SmallRng,

    hasher: DefaultHasher,
}

impl<'a, T> UserModel<'a, T> for SimpleEmailModel<'a, T>
where
    T: UserRequestIterator + Clone + Ord + PartialOrd + Eq + PartialEq,
{
    fn new(tot_users: u32, epoch: u32, uinfo: UserModelInfo<'a, T>) -> Self {
        let rng = SmallRng::from_entropy();
        let hasher = DefaultHasher::new();
        SimpleEmailModel {
            tot_users,
            current_time: 0,
            current_req: None,
            req_list: Vec::new(),
            uinfo,
            timestamp_sampler: None,
            size_sampler: None,
            contact_sampler: None,
            limit: 0,
            epoch,
            rng,
            hasher,
        }
    }

    fn with_timestamp_sampler(&mut self, timestamp_sampler: &'a Histogram) -> &mut Self {
        self.timestamp_sampler = Some(timestamp_sampler);
        self.init_list();
        self
    }

    fn with_size_sampler(&mut self, size_sampler: &'a Histogram) -> &mut Self {
        self.size_sampler = Some(size_sampler);
        self
    }

    #[inline]
    fn get_userid(&self) -> u32 {
        self.uinfo.get_userid()
    }
    #[inline]
    fn get_guard_for(&self, topo_idx: usize) -> Option<&'a Mixnode> {
        self.uinfo.get_guard_for(topo_idx)
    }
    #[inline]
    fn get_mailbox(&self, topo_idx: usize) -> Option<&'a MailBox> {
        Some(self.uinfo.get_mailbox(topo_idx))
    }

    #[inline]
    fn get_request(&self) -> Option<T> {
        self.uinfo.get_request()
    }

    #[inline]
    fn get_next_message_timing(&mut self) -> u64 {
        self.current_time += 1000;
        self.current_time
    }

    #[inline]
    fn get_current_time(&self) -> u64 {
        self.current_time
    }

    fn set_limit(&mut self, limit: u64) {
        self.limit = limit
    }

    fn set_contacts(&mut self, contacts: u32, die: &Uniform<u32>) {
        self.contact_sampler = Some(Uniform::from(0..contacts));
        let mut count = contacts;
        while count != 0 {
            let peer = die.sample(&mut self.rng);
            if peer != self.get_userid() && !self.uinfo.contacts_list.contains(&peer) {
                self.uinfo.contacts_list.push(peer);
                count -= 1;
            }
        }
    }
    #[inline]
    fn get_limit(&self) -> u64 {
        self.limit
    }
    fn with_receiver(&mut self, r: Receiver<T>) -> &mut Self {
        self.uinfo.with_receiver(r);
        self
    }
    #[inline]
    fn add_sender(&mut self, user: u32, s: Sender<T>) {
        self.uinfo.add_sender(user, s)
    }
    #[inline]
    fn drop_senders(&mut self) {
        self.uinfo.drop_senders()
    }
    #[inline]
    fn model_kind(&self) -> AnonModelKind {
        AnonModelKind::BothPeers
    }
    #[inline]
    fn update(&mut self, message_timing: u64) {
        self.uinfo.update(message_timing, &mut self.rng);
    }
}

impl<'a, T> SimpleEmailModel<'a, T>
where
    T: UserRequestIterator + Clone + PartialOrd + Ord + Eq + PartialEq,
{
    fn build_reqlist(&mut self) -> Option<T> {
        let contact: u32 =
            self.uinfo.contacts_list[self.contact_sampler.unwrap().sample(&mut self.rng) as usize];
        // req_timestamp is computed from the current period + the sampled value.
        let req_timestamp = self.timestamp_sampler.unwrap().sample(&mut self.rng) as u64 + self.current_time;
        // if we select over the simulation limit; we stop.
        if req_timestamp > self.limit {
            return None;
        }
        let topo_idx: u16 = (req_timestamp / self.epoch as u64) as u16;
        let req = T::new(
            &mut self.hasher,
            req_timestamp,
            self.size_sampler.unwrap().sample(&mut self.rng),
            (self.uinfo.get_userid(), contact),
            topo_idx,
        );
        match self.uinfo.send_request(req.clone()) {
            Ok(()) => (),
            Err(e) => panic!("Sending a request failed! {}", e),
        };

        Some(req)
    }

    #[inline]
    fn fetch_next(&mut self) -> Option<<SimpleEmailModel<'a,T> as Iterator>::Item> {
        let mailbox = self.get_mailbox(self.current_req.as_ref().unwrap().get_topos_idx() as usize);
        let req = self.current_req.as_mut().unwrap();
        let reqid = req.get_requestid();
        match req.next() {
            Some(timestamp) if timestamp < self.limit => {
                self.update(timestamp);
                Some((
                        timestamp,
                        self.uinfo.get_selected_guard(),
                        mailbox,
                        Some(reqid),
                ))
            },
            // we're over the limit
            Some(_) => None,
            None => None,
        }
    }

    fn init_list(&mut self) {
        // draw requests from the timestamps distribution
        for _ in 0..self.timestamp_sampler.unwrap().nbr_sampling {
            if let Some(req) = self.build_reqlist() {
                self.req_list.push(req);
            }
        }
        self.req_list.sort_by(|r1, r2| r1.get_request_time().cmp(&r2.get_request_time()));
        self.current_req = self.req_list.pop();
        self.current_time += self.timestamp_sampler.unwrap().period;
    }


}

impl<'a, T> Iterator for SimpleEmailModel<'a, T>
where
    T: UserRequestIterator + Clone + Eq + Ord + PartialEq + PartialOrd,
{
    type Item = (u64, Option<&'a Mixnode>, Option<&'a MailBox>, Option<u64>);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.fetch_next();
        match next {
            Some(item) => Some(item),
            None => {
                if self.req_list.is_empty() && self.current_time < self.limit {
                    self.init_list();
                    self.fetch_next()
                }
                else {
                    self.current_req = self.req_list.pop();
                    self.fetch_next()
                }
            }
        }
    }
}

#[derive(Hash)]
struct RequestId {
    request_time: u64,
    request_size: usize,
    peers: (u32, u32),
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct UserRequest {
    /// time of the initial request
    pub request_time: u64,
    /// nbr packets
    pub request_size: usize,
    /// peers
    pub peers: (u32, u32),
    /// requestid
    pub requestid: u64,
    /// current topology used when this object is created
    pub topos_idx: u16,
}

impl Hash for UserRequest {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.request_time.hash(state);
        self.request_size.hash(state);
        self.peers.hash(state);
    }
}

impl UserRequestIterator for UserRequest {
    type RequestTime = u64;
    type RequestSize = usize;

    fn new<H: Hasher>(
        state: &mut H,
        request_time: u64,
        request_size: usize,
        peers: (u32, u32),
        topos_idx: u16,
    ) -> Self {
        let mut r = UserRequest {
            request_time,
            request_size,
            peers,
            requestid: 0,
            topos_idx,
        };
        r.hash(state);
        r.requestid = state.finish();
        r
    }

    fn get_peers(&self) -> (u32, u32) {
        self.peers
    }

    fn get_requestid(&self) -> u64 {
        self.requestid
    }

    fn get_request_size(&self) -> Self::RequestSize {
        self.request_size
    }

    fn get_request_time(&self) -> Self::RequestTime {
        self.request_time
    }

    fn get_topos_idx(&self) -> u16 {
        self.topos_idx
    }

    fn next_with_bandwidth(&mut self, _bandwidth: Option<u32>) -> Option<u64> {
        // XXX consider handling the bandwidth 
        Some(self.request_time)
    }
}

impl Iterator for UserRequest {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.request_size > 0 {
            self.request_size -= 1;
            Some(self.request_time)
        } else {
            None
        }
    }
}
