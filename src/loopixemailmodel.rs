//! Contains an implementation of how an email-like data transfer would work on loopix.
//!
//! We assume for the worst-case analysis that user gateways are compromised.

use crate::mailbox::MailBox;
use crate::histogram::Histogram;
use crate::mixnodes::mixnode::Mixnode;
use crate::usermodel::*;
use crossbeam_channel::Receiver;
use rand::distributions::{Distribution, Uniform};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use siphasher::sip128::SipHasher;

pub struct LoopixEmailModel<'a, T> {

    current_time: u64,

    /// list of requests for the current period
    req_list: Vec<T>,
    /// request being considered now
    current_req: Option<T>,
    /// generic information that each model can share
    uinfo: UserModelInfo<'a, T>,
    /// Distribution of email timings
    timestamp_sampler: Option<&'a Histogram>,
    /// Distribution of sizes
    size_sampler: Option<&'a Histogram>,
    /// Whose's contact to talk
    contact_sampler: Option<Uniform<u32>>,
    /// timestamp limit for this model to sample²
    limit: u64,
    /// lifetime of a given topology
    epoch: u32,
    rng: SmallRng,
    hasher: SipHasher,
}


impl<'a, T> UserModel<'a, T> for LoopixEmailModel<'a, T>
where
    T: UserRequestIterator + Ord + PartialOrd + Eq + PartialEq,
{

    fn new(_tot_users: u32, epoch: u32, uinfo: UserModelInfo<'a, T>) -> Self {
        let rng = SmallRng::from_entropy();
        let hasher = SipHasher::new();
        LoopixEmailModel {
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
    fn get_request(&self) -> Option<T> {
        self.uinfo.get_request()
    }

    #[inline]
    fn get_current_time(&self) -> u64 {
        self.current_time
    }
    #[inline]
    fn get_limit(&self) -> u64 {
        self.limit
    }

    fn set_limit(&mut self, limit: u64) {
        self.limit = limit
    }
    
    /// does not use channels
    fn with_receiver(&mut self, _r: Receiver<T>) -> &mut Self {
        self
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
    fn model_kind(&self) -> AnonModelKind {
        AnonModelKind::ClientOnly
    }
    #[inline]
    fn update(&mut self, message_timing: u64) {
        self.uinfo.update(message_timing, &mut self.rng);
    }
}

impl<'a, T> LoopixEmailModel<'a, T>
where
    T: UserRequestIterator + PartialEq + Eq + PartialOrd + Ord,
{

    fn build_req(&mut self) -> Option<T> {
        let contact: u32 =
            self.uinfo.contacts_list[self.contact_sampler.unwrap().sample(&mut self.rng) as usize];
        // req_timestamp is computed from the current period + the sampled value.
        let req_timestamp =
            self.timestamp_sampler.unwrap().sample(&mut self.rng) as u64 + self.current_time;
        // if we select over the simulation limit; we stop.
        if req_timestamp >= self.limit {
            return None;
        }
        let topo_idx: u16 = (req_timestamp / self.epoch as u64) as u16;
        let req = T::new(
            &mut self.hasher,
            req_timestamp,
            self.size_sampler.unwrap().sample(&mut self.rng) as isize,
            (self.uinfo.get_userid(), contact),
            topo_idx,
        );
        Some(req)
    }

    #[inline]
    fn fetch_next(&mut self) -> Option<<LoopixEmailModel<'a, T> as Iterator>::Item> {
        if self.current_req.is_none() {
            return None;
        }
        let req = self.current_req.as_mut().unwrap();
        let reqid = req.get_requestid();
        match req.next() {
            Some(timestamp) if timestamp < self.limit => {
                self.update(timestamp);
                Some((
                    timestamp,
                    self.uinfo.get_selected_guard(),
                    None,
                    Some(reqid),
                ))
            }
            // we're over the limit
            Some(_) => None,
            None => None,
        }
    }
    fn init_list(&mut self) {
        // draw requests from the timestamps distribution
        let t_sampler = self.timestamp_sampler.unwrap();
        for _ in 0..t_sampler.nbr_sampling {
            if let Some(req) = self.build_req() {
                self.req_list.push(req);
            }
        }
        // should sort with the biggest request_time first.
        self.req_list
            .sort_by(|r1, r2| r2.get_request_time().cmp(&r1.get_request_time()));
        // pop the last element (i.e, the smallest request_time)
        self.current_req = self.req_list.pop();
        self.current_time += t_sampler.period + 1;
    }
}

impl<'a, T> Iterator for LoopixEmailModel<'a, T>
where
    T: UserRequestIterator + Eq + Ord + PartialEq + PartialOrd,
{
    type Item = (u64, Option<&'a Mixnode>, Option<&'a MailBox>, Option<u128>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.req_list.is_empty() {
            self.init_list();
        }
        let next = self.fetch_next();
        match next {
            Some(item) => Some(item),
            // Three possible cases:
            // 1) we consumed all requests, which mean
            // we can re-fill the list providing that the simulation shouldn't halt
            // 2) the list is not empty, and we're not over the limit. Let's pop and consume
            // the next request
            // 3) the is not empty but we're over the limit. In that case, fetch_next() is
            //    expected to return None
            // So eventually we can handle the three cases with a if {} else {}
            None => {
                if self.req_list.is_empty() && self.current_time < self.limit {
                    self.init_list();
                    self.fetch_next()
                } else {
                    self.current_req = self.req_list.pop();
                    self.fetch_next()
                }
            }
        }
    }
}
