use std::{
    collections::HashMap,
    ops::{Add, Mul},
};
pub type Value = i64;
pub type Prob = fraction::Fraction;
type Distribution = Box<dyn Iterator<Item = (Value, Prob)>>;
pub type DistributionHashMap = HashMap<Value, Prob>;

pub enum Factor {
    Constant(Value),
    FairDie { min: Value, max: Value },
    SumCompound(Vec<Box<Factor>>),
    ProductCompound(Vec<Box<Factor>>),
    MaxCompound(Vec<Box<Factor>>),
    MinCompound(Vec<Box<Factor>>),
    SampleSumCompound(Box<Factor>, Box<Factor>),
}

impl Factor {
    pub fn boxed_zero() -> Box<Factor> {
        Box::new(Factor::Constant(0))
    }

    pub fn boxed_one() -> Box<Factor> {
        Box::new(Factor::Constant(1))
    }

    pub fn distribution_vec(&self) -> Vec<(Value, Prob)> {
        self.distribution_iter().collect()
    }

    fn distribution_hashmap(&self) -> DistributionHashMap {
        match self {
            Factor::Constant(v) => {
                let mut m = DistributionHashMap::new();
                m.insert(*v, Prob::new(1u64, 1u64));
                return m;
            }
            Factor::FairDie { min, max } => {
                assert!(max >= min);
                let min: i64 = *min;
                let max: i64 = *max;
                let prob: Prob = Prob::new(1u64, (max - min + 1) as u64);
                let mut m = DistributionHashMap::new();
                for v in min..=max {
                    m.insert(v, prob);
                }
                return m;
            }

            Factor::SumCompound(vec) => {
                let hashmaps = vec
                    .iter()
                    .map(|e| e.distribution_hashmap())
                    .collect::<Vec<DistributionHashMap>>();
                return Factor::convolute_hashmaps(&hashmaps, |a, b| a + b);
            }
            Factor::ProductCompound(vec) => {
                let hashmaps = vec
                    .iter()
                    .map(|e| e.distribution_hashmap())
                    .collect::<Vec<DistributionHashMap>>();
                return Factor::convolute_hashmaps(&hashmaps, |a, b| a * b);
            }
            Factor::MaxCompound(vec) => {
                let hashmaps = vec
                    .iter()
                    .map(|e| e.distribution_hashmap())
                    .collect::<Vec<DistributionHashMap>>();
                return Factor::convolute_hashmaps(&hashmaps, |a, b| std::cmp::max(a, b));
            }
            Factor::MinCompound(vec) => {
                let hashmaps = vec
                    .iter()
                    .map(|e| e.distribution_hashmap())
                    .collect::<Vec<DistributionHashMap>>();
                return Factor::convolute_hashmaps(&hashmaps, |a, b| std::cmp::min(a, b));
            }
            Factor::SampleSumCompound(f1, f2) => {
                return Factor::sample_sum_convolute_two_hashmaps(
                    f1.distribution_hashmap(),
                    f2.distribution_hashmap(),
                );
            }
        }
    }

    pub fn distribution_iter(&self) -> Distribution {
        let mut distribution_vec = self
            .distribution_hashmap()
            .into_iter()
            .collect::<Vec<(Value, Prob)>>();
        distribution_vec.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        return Box::new(distribution_vec.into_iter());
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
                if m.contains_key(&v) {
                    *m.get_mut(&v).unwrap() += p;
                } else {
                    m.insert(v, p);
                }
            }
        }
        return m;
    }

    fn convolute_hashmaps(
        hashmaps: &Vec<DistributionHashMap>,
        operation: fn(Value, Value) -> Value,
    ) -> DistributionHashMap {
        // let mut m = HashMap::<Value, Prob>::new();
        let len = hashmaps.len();
        if len <= 0 {
            panic!("cannot convolute hashmaps from a zero element vector");
        }
        let mut convoluted_h = hashmaps[0].clone();
        for i in 1..len {
            convoluted_h = Factor::convolute_two_hashmaps(&convoluted_h, &hashmaps[i], operation);
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
                        .map(|e| e.clone())
                        .take(count)
                        .collect();
                    Factor::convolute_hashmaps(&sample_vec, |a, b| a + b)
                }
            };
            count_hashmap.iter_mut().for_each(|e| {
                *e.1 = *e.1 * *count_p;
            });
            hashmap_extensions::merge_hashmaps(&mut total_hashmap, &count_hashmap);
        }
        total_hashmap
    }
}

impl Mul for Box<Factor> {
    type Output = Box<Factor>;

    fn mul(self, rhs: Self) -> Self::Output {
        return Box::new(Factor::ProductCompound(vec![self, rhs]));
    }
}

impl Add for Box<Factor> {
    type Output = Box<Factor>;

    fn add(self, rhs: Self) -> Self::Output {
        return Box::new(Factor::SumCompound(vec![self, rhs]));
    }
}

mod hashmap_extensions {
    use super::DistributionHashMap;

    pub fn merge_hashmaps(first: &mut DistributionHashMap, second: &DistributionHashMap) {
        for (k, v) in second.iter() {
            if first.contains_key(&k) {
                *first.get_mut(&k).unwrap() += v;
            } else {
                first.insert(*k, *v);
            }
        }
    }
}
