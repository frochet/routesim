use serde::Deserialize;
use serde_json::Result;
use rand::prelude::*;


#[derive(Deserialize)]
struct HistData {
    nbr_sampling: u32,
    bin_size: usize,
    data: Vec<usize>,
}

pub struct Histogram {
    /// One week by default
    period: u64,
    /// The number of sampling we take over the overall period
    nbr_sampling: u32,

    timestamps: Vec<usize>,
    wi: Box<WeightedAliasIndex<usize>>,
}


impl Histogram {

    /// nbr_sampling should also be described within the json data
    /// {
    ///     "nbr_sampling": int,
    ///     "data": list,
    /// }
    /// data should be a list of (counts, timestamps).
    /// E.g., in the email case, the bin value should be a value every
    /// hour up to 1 week in timestamp (7*24 timestamp), with a width of 60*60.
    /// period will be derived from the last bin value
    pub fn from_json(json_data: &str) -> Result<Histogram> {

        let mut jdata: HistData =  serde_json::from_str(json_data)?;
        jdata.data.sort();
        let period = jdata.data.last().unwrap_or(&(60*60*24*7 as usize)) - jdata.data.first().unwrap_or(&(0 as usize));
        let first = jdata.data.first().unwrap_or(&(0 as usize));
        let last = jdata.data.last().unwrap_or(&(60*60*24*7 as usize));
        let mut bin: i64 = 0;
        let mut count: i64 = 0;
        let (timestamps, weights): (Vec<usize>, Vec<usize>) = jdata.data.iter().map(|timestamp| {
            let curval: i64 = (*timestamp - *first) as i64;
            let curbin: i64 = bin as i64;
            if curval - curbin <= curbin + jdata.bin_size as i64 - curval {
                count += 1;
                //cmp addr
                if timestamp != last {
                    (0, 0)
                }
                else {
                    (bin as usize, count as usize)
                }
            }
            else {
                // what bin are we the closest?
                let this_count = count;
                let this_bin = bin;
                count = 1;
                let mut cur_timestamp: i64 = *timestamp as i64;
                while cur_timestamp % jdata.bin_size as i64 != 0 {
                    cur_timestamp -=1
                }
                let prev_bin: i64 = cur_timestamp as i64;
                if *timestamp as i64 - prev_bin <= prev_bin + (jdata.bin_size as i64) - *timestamp as i64 {
                    bin = prev_bin;
                }
                else {
                    bin = prev_bin + jdata.bin_size as i64;
                }
                (this_bin as usize, this_count as usize)
            }
        }).filter(|x| if let (0, 0) = x {false} else {true}).unzip();

        let wi = Box::new(
            WeightedAliasIndex::new(weights).unwrap());
        Ok(Histogram {
            period: period as u64,
            nbr_sampling: jdata.nbr_sampling,
            timestamps,
            wi,
        })

    }

    pub fn sample<R: Rng+?Sized>(&self, rng: &mut R) -> usize {
        let idx = self.wi.sample(rng);
        match self.timestamps.get(idx) {
            Some(elem) => *elem,
            None=> panic!("Wrong idx: {}", idx),
        }
    }
}

mod tests {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    fn get_histo() -> Result<Histogram> {

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
                14
            ]
        }"#;
        let histogram = Histogram::from_json(jdata)?;
        Ok(histogram)
    }

    #[test]
    fn test_from_json() {
        let histogram = get_histo();
        assert!(histogram.is_ok(), "The histogram wasn't loaded properly");
        assert_eq!(histogram.unwrap().timestamps, vec![0, 5, 15]);
    }

    #[test]
    fn test_sampling() {
        let histogram = get_histo().unwrap();
        let mut rng = SmallRng::from_entropy();
        for _ in 0..100 {
            let sample1 = histogram.sample(&mut rng);
            assert!(sample1 <= 15, "{} is larger than 15", sample1);
        }
    }
}
