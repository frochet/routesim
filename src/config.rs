use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::path::Path;
use crate::mixnodes::mixnode::Mixnode;

/// A config is a set of hashmaps containing mixes
#[derive(Default)]
pub struct Config {
    layers: [HashMap<u32, Mixnode>; 3],
    unselected: HashMap<u32, Mixnode>
}

impl Config {
    pub fn new() -> Self {
        Config {
            ..Default::default()
        }
    }

    #[allow(dead_code)]
    pub fn update(&self) {
    }

    pub fn layers(&self) -> &[HashMap<u32, Mixnode>] {
        &self.layers
    }
    pub fn unselected(&self) -> &HashMap<u32, Mixnode> {
        &self.unselected
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
            let mix: Mixnode = line.parse().expect(&format!("Unable to parse into a Mixnode -- Is your data correct? {}", line));
            match mix.layer {
                0 | 1 | 2 => config.layers[mix.layer as usize].insert(mix.mixid, mix),
                _ => config.unselected.insert(mix.mixid, mix),
            };
        }
        else {
            println!("Something went wrong while reading");
        }
    }
    config
}

#[test]
fn load_test_config() {
    let config = load("testfiles/1000_137_Random_BP_layout.csv");
    let mix = config.layers()[0].get(&42).unwrap();
    //42│20.430784458454426│False│0
    assert_eq!(mix.is_malicious, false);
    assert_eq!(mix.weight,20.430784458454426);
}




