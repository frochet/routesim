use crate::config::PAYLOAD_SIZE;
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
use std::hash::{Hash, Hasher};
use siphasher::sip128::{Hasher128, SipHasher};


pub struct SimpleEmailModel<'a, T> {
    _tot_users: u32,

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

    hasher: SipHasher,
}

impl<'a, T> UserModel<'a, T> for SimpleEmailModel<'a, T>
where
    T: UserRequestIterator + Clone + Ord + PartialOrd + Eq + PartialEq,
{
    fn new(_tot_users: u32, epoch: u32, uinfo: UserModelInfo<'a, T>) -> Self {
        let rng = SmallRng::from_entropy();
        let hasher = SipHasher::new();
        SimpleEmailModel {
            _tot_users,
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
        //self.init_list();
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

    fn get_contacts(&self) -> Option<&[u32]> {
        Some(&self.uinfo.contacts_list)
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
        match self.uinfo.send_request(req.clone()) {
            Ok(()) => (),
            Err(e) => panic!("Sending a request failed! {}", e),
        };

        Some(req)
    }

    #[inline]
    fn fetch_next(&mut self) -> Option<<SimpleEmailModel<'a, T> as Iterator>::Item> {
        // that may happen if we build an empty list of requests because we're over the limit
        // already
        if self.current_req.is_none() {
            return None;
        }
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

impl<'a, T> Iterator for SimpleEmailModel<'a, T>
where
    T: UserRequestIterator + Clone + Eq + Ord + PartialEq + PartialOrd,
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
    pub request_size: isize,
    /// peers
    pub peers: (u32, u32),
    /// requestid
    pub requestid: u128,
    /// current topology used when this object is created
    pub topos_idx: u16,
}

impl std::hash::Hash for UserRequest {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.request_time.hash(state);
        self.request_size.hash(state);
        self.peers.hash(state);
    }
}

impl UserRequestIterator for UserRequest {
    type RequestTime = u64;
    type RequestSize = isize;

    fn new<H: Hasher + Hasher128>(
        state: &mut H,
        request_time: u64,
        request_size: isize,
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
        r.requestid = state.finish128().as_u128();
        r
    }

    fn get_peers(&self) -> (u32, u32) {
        self.peers
    }

    fn get_requestid(&self) -> u128 {
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
        // we can have a bin "0" for the size; we sent one packet in that case
        if self.request_size >= 0 {
            self.request_size -= PAYLOAD_SIZE as isize;
            Some(self.request_time)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use crate::routesim::Runable;
    use serde_json::Result;

    fn build_timestamp_hist() -> Result<Histogram> {
        let jdata = r#"
        {
            "nbr_sampling": 10,
            "bin_size": 5,
            "data": [
                1,
                1,
                1,
                3,
                3,
                3,
                4,
                4,
                14,
                14,
                3200,
                3201,
                4000
            ]
        }"#;
        let histogram = Histogram::from_json(jdata, 5)?;
        Ok(histogram)
    }

    fn build_size_hist() -> Result<Histogram> {
        let jdata = r#"
        {
            "nbr_sampling": 0,
            "bin_size": 200,
            "data": [
                30,
                42,
                150,
                800,
                810,
                830,
                4400,
                4450,
                7200
            ]
        }"#;
        let histogram = Histogram::from_json(jdata, 5)?;
        Ok(histogram)
    }

    #[test]
    fn test_simple_email() {
        let max = 4000;
        let config = config::load("testfiles/single_layout/1000_137_Random_BP_layout.csv", 1);
        let t_sampler = build_timestamp_hist().unwrap();
        let s_sampler = build_size_hist().unwrap();
        let mut topologies = vec![];
        topologies.push(config.clone());
        topologies.push(config);
        let mut runner = Runable::new(10, topologies, 1, 43200, 3);
        let limit = runner.days_to_timestamp() - 1;
        assert_eq!(limit, 86400 - 1);
        runner
            .with_timestamps_hist(t_sampler)
            .with_sizes_hist(s_sampler);
        let mut usermodels = runner.init::<SimpleEmailModel<UserRequest>, UserRequest>();

        let usermodel = usermodels.get_mut(0).unwrap();
        usermodel.set_limit(limit);
        let contacts: Vec<u32> = usermodel
            .get_contacts()
            .unwrap()
            .iter()
            .map(|c| *c)
            .collect();
        assert_eq!(contacts.len(), 3);
        if let Some((message_timing, _guard, _mailbox, _requestid)) = usermodel.next() {
            assert!(message_timing <= max);
        } else {
            panic!("We should have messages!");
        }

        let mut last_timing: u64 = 0;
        for (message_timing, _guard, _mailbox, _requestid) in usermodel {
            assert!(message_timing >= last_timing);
            last_timing = message_timing;
        }
        assert!(last_timing >= 80000);
    }
}
