use super::dice_string_parser::{self, GraphBuildingError};
use std::{
    collections::HashMap,
    ops::{Add, Mul},
};
pub type Value = i64;
pub type Prob = fraction::BigFraction;
pub type AggrValue = fraction::BigFraction;
type Distribution = Box<dyn Iterator<Item = (Value, Prob)>>;
pub type DistributionHashMap = HashMap<Value, Prob>;

#[derive(Debug, PartialEq, Eq)]
pub enum Factor {
    Constant(Value),
    FairDie { min: Value, max: Value },
    SumCompound(Vec<Factor>),
    ProductCompound(Vec<Factor>),
    MaxCompound(Vec<Factor>),
    MinCompound(Vec<Factor>),
    SampleSumCompound(Box<Factor>, Box<Factor>),
}

pub struct FactorStats {
    pub min: Value,
    pub max: Value,
    pub median: Value,
    pub mode: Value,
    pub mean: AggrValue,
    pub sd: AggrValue,
    pub distribution: Vec<(Value, Prob)>,
}

impl Factor {
    pub fn stats(&self) -> FactorStats {
        let dist_vec: Vec<(Value, Prob)> = self.distribution_iter().collect();

        FactorStats::from_distribution(dist_vec)
    }

    fn distribution_hashmap(&self) -> DistributionHashMap {
        match self {
            Factor::Constant(v) => {
                let mut m = DistributionHashMap::new();
                m.insert(*v, Prob::new(1u64, 1u64));
                m
            }
            Factor::FairDie { min, max } => {
                assert!(max >= min);
                let min: i64 = *min;
                let max: i64 = *max;
                let prob: Prob = Prob::new(1u64, (max - min + 1) as u64);
                let mut m = DistributionHashMap::new();
                for v in min..=max {
                    m.insert(v, prob.clone());
                }
                m
            }

            Factor::SumCompound(vec) => {
                let hashmaps = vec
                    .iter()
                    .map(|e| e.distribution_hashmap())
                    .collect::<Vec<DistributionHashMap>>();
                convolute_hashmaps(&hashmaps, |a, b| a + b)
            }
            Factor::ProductCompound(vec) => {
                let hashmaps = vec
                    .iter()
                    .map(|e| e.distribution_hashmap())
                    .collect::<Vec<DistributionHashMap>>();
                convolute_hashmaps(&hashmaps, |a, b| a * b)
            }
            Factor::MaxCompound(vec) => {
                let hashmaps = vec
                    .iter()
                    .map(|e| e.distribution_hashmap())
                    .collect::<Vec<DistributionHashMap>>();
                convolute_hashmaps(&hashmaps, std::cmp::max)
            }
            Factor::MinCompound(vec) => {
                let hashmaps = vec
                    .iter()
                    .map(|e| e.distribution_hashmap())
                    .collect::<Vec<DistributionHashMap>>();
                convolute_hashmaps(&hashmaps, std::cmp::min)
            }
            Factor::SampleSumCompound(f1, f2) => sample_sum_convolute_two_hashmaps(
                f1.distribution_hashmap(),
                f2.distribution_hashmap(),
            ),
        }
    }

    fn distribution_iter(&self) -> Distribution {
        let mut distribution_vec = self
            .distribution_hashmap()
            .into_iter()
            .collect::<Vec<(Value, Prob)>>();
        distribution_vec.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        Box::new(distribution_vec.into_iter())
    }

    // fn distribution_vec(&self) -> Vec<(Value, Prob)> {
    //     self.distribution_iter().collect()
    // }

    pub fn from_string(input: &str) -> Result<Self, GraphBuildingError> {
        dice_string_parser::string_to_factor(input)
    }
}

fn convolute_two_hashmaps(
    h1: &HashMap<Value, Prob>,
    h2: &HashMap<Value, Prob>,
    operation: fn(Value, Value) -> Value,
) -> DistributionHashMap {
    let mut m = DistributionHashMap::new();
    for (v1, p1) in h1.iter() {
        // println!("loop1 v1:{} p1:{}", v1, p1);
        for (v2, p2) in h2.iter() {
            // println!("loop2 v2:{} p2:{}", v2, p2);
            let v = operation(*v1, *v2);
            let p = p1 * p2;
            if let std::collections::hash_map::Entry::Vacant(e) = m.entry(v) {
                e.insert(p);
            } else {
                *m.get_mut(&v).unwrap() += p;
            }
        }
    }
    m
}

fn convolute_hashmaps(
    hashmaps: &Vec<DistributionHashMap>,
    operation: fn(Value, Value) -> Value,
) -> DistributionHashMap {
    // let mut m = HashMap::<Value, Prob>::new();
    let len = hashmaps.len();
    if len == 0 {
        panic!("cannot convolute hashmaps from a zero element vector");
    }
    let mut convoluted_h = hashmaps[0].clone();
    for h in hashmaps.iter().skip(1) {
        convoluted_h = convolute_two_hashmaps(&convoluted_h, h, operation);
    }
    convoluted_h
}

fn sample_sum_convolute_two_hashmaps(
    count_factor: DistributionHashMap,
    sample_factor: DistributionHashMap,
) -> DistributionHashMap {
    let mut total_hashmap = DistributionHashMap::new();
    for (count, count_p) in count_factor.iter() {
        let mut count_hashmap: DistributionHashMap = match count.cmp(&0) {
            std::cmp::Ordering::Less => {
                panic!("cannot use count_factor {}", count);
            }
            std::cmp::Ordering::Equal => {
                let mut h = DistributionHashMap::new();
                h.insert(0, Prob::new(1u64, 1u64));
                h
            }
            std::cmp::Ordering::Greater => {
                let count: usize = *count as usize;
                let sample_vec: Vec<DistributionHashMap> = std::iter::repeat(&sample_factor)
                    .take(count)
                    .cloned()
                    .collect();
                convolute_hashmaps(&sample_vec, |a, b| a + b)
            }
        };
        count_hashmap.iter_mut().for_each(|e| {
            *e.1 *= count_p.clone();
        });
        merge_hashmaps(&mut total_hashmap, &count_hashmap);
    }
    total_hashmap
}

impl Mul for Box<Factor> {
    type Output = Box<Factor>;

    fn mul(self, rhs: Self) -> Self::Output {
        Box::new(Factor::ProductCompound(vec![*self, *rhs]))
    }
}

impl Add for Box<Factor> {
    type Output = Box<Factor>;

    fn add(self, rhs: Self) -> Self::Output {
        Box::new(Factor::SumCompound(vec![*self, *rhs]))
    }
}

impl FactorStats {
    /// assumes distribution is sorted by Value in ascending order
    fn from_distribution(distribution: Vec<(Value, Prob)>) -> FactorStats {
        let max: Value = distribution.last().map(|e| e.0).unwrap();
        let min: Value = distribution.first().map(|e| e.0).unwrap();

        // match distribution.first() {
        //     None => None,
        //     Some(e) => Some(e.0),
        // }     .unwrap();

        let mut mean: AggrValue = AggrValue::from(0);

        let mut total_probability: Prob = Prob::new(0u64, 1u64);
        let median_prob: Prob = Prob::new(1u64, 2u64);
        // todo median
        let mut median: Option<Value> = None;
        let mut mode: Option<(Value, Prob)> = None;
        for (val, prob) in distribution.iter().cloned() {
            mean += prob.clone() * Prob::from(val);
            total_probability += prob.clone();
            match median {
                Some(_) => {}
                None => {
                    if total_probability >= median_prob {
                        median = Some(val);
                    }
                }
            }
            match &mode {
                Some((_, p)) => {
                    if prob > *p {
                        mode = Some((val, prob));
                    }
                }
                None => {
                    mode = Some((val, prob));
                }
            }
        }

        let mut sd: AggrValue = AggrValue::from(0);
        for (val, prob) in distribution.iter().cloned() {
            let val = AggrValue::from(val);
            let val_minus_mean = &val - &mean;
            let square = (&val_minus_mean) * (&val_minus_mean);
            sd += square * prob
        }

        let median = median.unwrap();
        let mode = mode.unwrap().0;

        FactorStats {
            mean,
            sd,
            mode,
            min,
            max,
            median,
            distribution,
        }
    }
}

pub fn merge_hashmaps(first: &mut DistributionHashMap, second: &DistributionHashMap) {
    for (k, v) in second.iter() {
        if first.contains_key(k) {
            *first.get_mut(k).unwrap() += v;
        } else {
            first.insert(*k, v.clone());
        }
    }
}
