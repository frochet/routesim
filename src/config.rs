use crate::mailbox::MailBox;
use crate::mixnodes::mixnode::Mixnode;
use array_init::array_init;
use rand::prelude::*;
use rand_distr::weighted_alias::WeightedAliasIndex;
use rustc_hash::FxHashMap as HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::vec::IntoIter;

pub const PATH_LENGTH: i8 = 3;
#[allow(dead_code)]
/// in byte
pub const PAYLOAD_SIZE: usize = 2048;
/// Default sample size for guards -- todo move this in clap
pub const GUARDS_SAMPLE_SIZE: usize = 5;
/// How much do we extend the sample size each time we ran out of guard?
pub const GUARDS_SAMPLE_SIZE_EXTEND: usize = 2;
/// guards are use in layer... [0..n[
pub const GUARDS_LAYER: usize = 1;

/// A config is a set of mixes for each layer
/// and a hashmap for unselected mixes.
#[derive(Default, Clone)]
pub struct TopologyConfig {
    pub filename: String,

    pub epoch: u32,
    /// The path length
    layers: [Vec<Mixnode>; PATH_LENGTH as usize],
    wc_layers: [Box<Option<WeightedAliasIndex<f64>>>; PATH_LENGTH as usize],
    unselected: HashMap<u32, Mixnode>,
    /// This topology is valid until valid_until's value.
    #[allow(dead_code)]
    valid_until: u64,
    /// We assume mailboxes are public information for user ids
    mailboxes: HashMap<u32, MailBox>,
}

impl TopologyConfig {
    pub fn new(filename: String, epoch: u32) -> Self {
        TopologyConfig {
            filename,
            epoch,
            wc_layers: array_init(|_| Box::new(None)),
            ..Default::default()
        }
    }

    pub fn with_mailboxes(&mut self, tot_users: u32) -> &mut Self {
        (0..tot_users).for_each(|user| {
            self.mailboxes
                .insert(user, MailBox::new(&self.layers[0..1]));
        });
        self
    }

    pub fn get_mailbox(&self, userid: u32) -> Option<&MailBox> {
        self.mailboxes.get(&userid)
    }

    #[allow(dead_code)]
    pub fn layers(&self) -> &[Vec<Mixnode>] {
        &self.layers
    }
    #[allow(dead_code)]
    pub fn unselected(&self) -> &HashMap<u32, Mixnode> {
        &self.unselected
    }

    /// sample n guards from layer l
    pub fn sample_guards<'a, R: Rng + ?Sized>(
        &'a self,
        l: usize,
        n_guards: usize,
        rng: &mut R,
    ) -> IntoIter<&'a Mixnode> {
        let mut sample_guards = vec![];
        for _ in 0..n_guards {
            if let Some(wc) = &*self.wc_layers[l] {
                sample_guards.push(&self.layers[l][wc.sample(rng)]);
            }
        }
        sample_guards.into_iter()
    }

    /// Sample a route from the network layer configuration
    #[inline]
    pub fn sample_path<'a, R: Rng + ?Sized>(
        &'a self,
        rng: &mut R,
        guard: Option<&'a Mixnode>,
    ) -> IntoIter<&'a Mixnode> {
        let mut path = vec![];
        // returns an owned iterator
        for i in 0..PATH_LENGTH {
            if let Some(wc) = &*self.wc_layers[i as usize] {
                if i as usize == GUARDS_LAYER && guard.is_some() {
                    path.push(guard.unwrap());
                } else {
                    path.push(&self.layers[i as usize][wc.sample(rng)]);
                }
            }
        }
        path.into_iter()
    }
}

/// Load the network configuration from filename.
///
/// Each line must be
/// mixid [integer], weight [float], is_malicious [bool], layer [-1..2]
pub fn load<P>(filename: P, tot_users: u32) -> TopologyConfig
where
    P: AsRef<Path>,
{
    let file = File::open(&filename).expect("Unable to open the file");
    let filename_string: String = filename.as_ref().to_str().unwrap().to_owned();
    let mut line_reader = BufReader::new(file).lines();
    let mut epoch: u32 = 0;
    if let Some(header) = line_reader.next() {
        let header_val = header.unwrap_or("mixid, bandwidth, malicious, epoch_0".to_string());
        let epoch_val = header_val.split(',').nth(3).unwrap_or("epoch_0");
        let epoch_val = epoch_val.split('_').nth(1);
        match epoch_val {
            Some(some_integer) => epoch = some_integer.parse::<u32>().unwrap(),
            None => (),
        }
    }
    let mut config: TopologyConfig = TopologyConfig::new(filename_string, epoch);

    //skip header
    for line_r in line_reader {
        if let Ok(line) = line_r {
            let mix: Mixnode = line.parse().unwrap_or_else(|_| {
                panic!(
                    "Unable to parse {} into a Mixnode -- Is your data correct? {}",
                    filename.as_ref().display(),
                    line
                )
            });
            match mix.layer {
                // This trick gets arround the unsupported excluded range syntax for now
                0..=PATH_LENGTH if mix.layer < PATH_LENGTH => {
                    config.layers[mix.layer as usize].push(mix)
                }
                _ => {
                    config.unselected.insert(mix.mixid, mix);
                }
            };
        } else {
            println!("Something went wrong while reading");
        }
    }
    for i in 0..PATH_LENGTH {
        config.wc_layers[i as usize] = Box::new(Some(
            WeightedAliasIndex::new(
                config.layers[i as usize]
                    .iter()
                    .map(|item| item.weight)
                    .collect(),
            )
            .unwrap(),
        ));
    }
    config.with_mailboxes(tot_users);
    config
}

#[test]
fn load_test_topology_config() {
    let config = load("testfiles/single_layout/1000_137_Random_BP_layout.csv", 1);
    let mix = &config.layers()[0][42];
    //42│20.430784458454426│False│0
    assert_eq!(mix.is_malicious, false);
    assert_eq!(mix.weight, 1.8480844586590168);
}
#[test]
fn test_sample_path() {
    let config = load("testfiles/single_layout/1000_137_Random_BP_layout.csv", 10);
    let mut rng = thread_rng();
    let path = config.sample_path(&mut rng, None);
    let path = path.collect::<Vec<_>>();
    assert_eq!(path.len(), 3);
}
