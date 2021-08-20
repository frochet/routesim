use super::mixerror::MixError;
use std::error;
use std::str::FromStr;

pub struct Mixnode {
    pub layer: i8,
    pub weight: f64,
    pub mixid: u32,
    pub is_malicious: bool,
}

/// should parse one line of the config line
impl FromStr for Mixnode {
    type Err = Box<dyn error::Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split(',').map(|each| each.trim().to_lowercase()).collect::<Vec<_>>() {
            vec if vec.len() < 4 => Err(Box::new(MixError::new("Missing arguments"))),
            vec if vec.len() >= 4 => {
                let mixid_p = vec[0].parse::<u32>()?;
                let bw_p = vec[1].parse::<f64>()?;
                let is_malicious_p = vec[2].parse::<bool>()?;
                let layer_p = vec[3].parse::<i8>()?;
                Ok(Mixnode{layer:layer_p, weight: bw_p, mixid: mixid_p, is_malicious:
                    is_malicious_p})
            },
            _ => Err(Box::new(MixError::new("The line information are unexpected. It should be: mixid,
                                   weight, is_malicious, layer")))
        }
    }
}

#[test]
#[should_panic(expected="Missing arguments")]
fn test_missing_arguments() {
    let _mix: Mixnode = "10,1".parse().unwrap();
}
#[test]
fn test_correct_arguments() {
    let mix: Mixnode = "10, 200, False, -1".parse().unwrap();
    assert_eq!(mix.layer, -1);
    assert_eq!(mix.weight, 200.0);
    assert_eq!(mix.mixid, 10);
    assert_eq!(mix.is_malicious, false);
}
#[test]
fn test_without_space() {
    let mix: Mixnode = "0,7.222983840621532,False,-1".parse().unwrap();
    assert_eq!(mix.layer, -1);
    assert_eq!(mix.weight, 7.222983840621532);
    assert_eq!(mix.mixid, 0);
    assert_eq!(mix.is_malicious, false);
}

#[test]
fn test_long_strings() {
    let mix: Mixnode = "10, 200, False, -1, 1, 0, 2".parse().unwrap();
    assert_eq!(mix.layer, -1);
    assert_eq!(mix.weight, 200.0);
    assert_eq!(mix.mixid, 10);
    assert_eq!(mix.is_malicious, false);
}
