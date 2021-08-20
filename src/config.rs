use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::path::Path;
use crate::mixnodes::mixnode::Mixnode;
use std::vec::IntoIter;
use rand::prelude::*;
use rand::distributions::WeightedIndex;

/// A config is a set of hashmaps containing mixes
#[derive(Default)]
pub struct Config {
    layers: [Vec<Mixnode>; 3],
    wc_layers: [Box<Option<WeightedIndex<f64>>>; 3],
    unselected: HashMap<u32, Mixnode>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            wc_layers: [Box::new(None), Box::new(None), Box::new(None)],
            ..Default::default()
        }
    }

    #[allow(dead_code)]
    pub fn update(&self) {
    }

    pub fn layers(&self) -> &[Vec<Mixnode>] {
        &self.layers
    }
    pub fn unselected(&self) -> &HashMap<u32, Mixnode> {
        &self.unselected
    }
    pub fn sample_path(&self, rng: &mut ThreadRng) -> IntoIter<u32> {
        let mut path = vec![0 as u32; 3];
        // TODO sample the path
        // and returns an owned iterator
        for i in 0..3 {
            if let Some(wc) = &*self.wc_layers[i] {
                path[i] = self.layers[i][wc.sample(rng)].mixid;
            }
        }
        path.into_iter()
    }
}

pub fn load<P>(filename: P) -> Config
where
    P: AsRef<Path>,
{
    let file = File::open(filename).expect("Unable to open the file");
    let mut config: Config = Config::new();
    //skip header
    for line_r in BufReader::new(file).lines().skip(1) {
        if let Ok(line) = line_r {
            let mix: Mixnode = line.parse().expect(&format!("Unable to parse into a Mixnode -- Is
                                                            your data correct? {}", line));
            match mix.layer {
                0 | 1 | 2 => config.layers[mix.layer as usize].push(mix),
                _ => {config.unselected.insert(mix.mixid, mix);},
            };
        }
        else {
            println!("Something went wrong while reading");
        }
    }
    for i in 0..3 {
        config.wc_layers[i] = Box::new(Some(WeightedIndex::new(config.layers[i].iter().map(|item| item.weight)).unwrap()));
    }
    config
}

#[test]
fn load_test_config() {
    let config = load("testfiles/1000_137_Random_BP_layout.csv");
    let mix = &config.layers()[0][42];
    //42│20.430784458454426│False│0
    assert_eq!(mix.is_malicious, false);
    assert_eq!(mix.weight,1.8480844586590168);
}
#[test]
fn test_sample_path() {
    let config = load("testfiles/1000_137_Random_BP_layout.csv");
    let mut rng = thread_rng();
    let path = config.sample_path(&mut rng);
    let path = path.collect::<Vec<_>>();
    assert_eq!(path.len(), 3);
}
