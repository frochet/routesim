use crate::mixnodes::mixnode::Mixnode;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::vec::IntoIter;

const PATH_LENGTH: i8 = 3;
/// in byte
const PAYLOAD_SIZE: usize = 2048;

/// A config is a set of mixes for each layer
/// and a hashmap for unselected mixes.
#[derive(Default)]
pub struct Config {
    /// The path length
    layers: [Vec<Mixnode>; PATH_LENGTH as usize],
    wc_layers: [Box<Option<WeightedIndex<f64>>>; PATH_LENGTH as usize],
    unselected: HashMap<u32, Mixnode>,
    /// Payload size of a mixnet message
    #[allow(dead_code)]
    payload_size: usize,
}

impl Config {
    pub fn new() -> Self {
        Config {
            wc_layers: [Box::new(None), Box::new(None), Box::new(None)],
            payload_size: PAYLOAD_SIZE,
            ..Default::default()
        }
    }

    /// This function is supposed to be callede every period to update the network topology
    /// Currently we assume that it is stays the same
    ///
    /// In the future, we should support loading ordered config files from the disk, which would be
    /// useful to replay an history of mixnet topologies
    #[allow(dead_code)]
    pub fn update(&mut self) -> &mut Config {
        self
    }
    #[allow(dead_code)]
    pub fn layers(&self) -> &[Vec<Mixnode>] {
        &self.layers
    }
    #[allow(dead_code)]
    pub fn unselected(&self) -> &HashMap<u32, Mixnode> {
        &self.unselected
    }
    /// Sample a route from the network layer configation
    pub fn sample_path(&self, rng: &mut ThreadRng) -> IntoIter<&Mixnode> {
        let mut path = vec![];
        // returns an owned iterator
        for i in 0..PATH_LENGTH {
            if let Some(wc) = &*self.wc_layers[i as usize] {
                path.push(&self.layers[i as usize][wc.sample(rng)]);
            }
        }
        path.into_iter()
    }
    /// Check whether the three mixnode in path are compromised.
    /// return true if they are, false otherwise.
    pub fn is_path_malicious(&self, path: &mut IntoIter<&Mixnode>) -> bool {
        let mut mal_mix = 0;
        for hop in path {
            if hop.is_malicious {
                mal_mix += 1;
            }
        }
        mal_mix == PATH_LENGTH
    }
}

/// Load the network configuration from filename.
///
/// Each line must be
/// mixid [integer], weight [float], is_malicious [bool], layer [-1..2]
pub fn load<P>(filename: P) -> Config
where
    P: AsRef<Path>,
{
    let file = File::open(filename).expect("Unable to open the file");
    let mut config: Config = Config::new();
    //skip header
    for line_r in BufReader::new(file).lines().skip(1) {
        if let Ok(line) = line_r {
            let mix: Mixnode = line.parse().unwrap_or_else(|_| {
                panic!(
                    "Unable to parse into a Mixnode -- Is
                                                            your data correct? {}",
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
    for i in 0..3 {
        config.wc_layers[i] = Box::new(Some(
            WeightedIndex::new(config.layers[i].iter().map(|item| item.weight)).unwrap(),
        ));
    }
    config
}

#[test]
fn load_test_config() {
    let config = load("testfiles/1000_137_Random_BP_layout.csv");
    let mix = &config.layers()[0][42];
    //42│20.430784458454426│False│0
    assert_eq!(mix.is_malicious, false);
    assert_eq!(mix.weight, 1.8480844586590168);
}
#[test]
fn test_sample_path() {
    let config = load("testfiles/1000_137_Random_BP_layout.csv");
    let mut rng = thread_rng();
    let path = config.sample_path(&mut rng);
    let path = path.collect::<Vec<_>>();
    assert_eq!(path.len(), 3);
}
