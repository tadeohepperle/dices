use std::{
    collections::HashMap,
    iter::once,
    ops::{Add, Mul},
};
pub type Value = i64;
pub type Prob = fraction::Fraction;
type Distribution = Box<dyn Iterator<Item = (Value, Prob)>>;

pub enum Factor {
    Constant(Value),
    FairDie { min: Value, max: Value },
    SumCompound(Box<Factor>, Box<Factor>),
    ProductCompound(Box<Factor>, Box<Factor>),
    MaxCompound(Box<Factor>, Box<Factor>),
    MinCompound(Box<Factor>, Box<Factor>),
}

impl Factor {
    pub fn boxed_zero() -> Box<Factor> {
        Box::new(Factor::Constant(0))
    }

    pub fn boxed_one() -> Box<Factor> {
        Box::new(Factor::Constant(1))
    }

    pub fn distribution_vec(&self) -> Vec<(Value, Prob)> {
        self.distribution().collect()
    }

    pub fn distribution(&self) -> Distribution {
        match self {
            Factor::Constant(v) => Box::new(once((*v, Prob::from(1.0)))),
            Factor::FairDie { min, max } => {
                assert!(max >= min);
                let min: i64 = *min;
                let max: i64 = *max;
                let prob: Prob = Prob::new(1u64, (max - min + 1) as u64);
                return Box::new((min..=max).map(move |e| (e, prob)));
            }
            Factor::SumCompound(f1, f2) => Factor::convolute_distributions(f1, f2, |a, b| a + b),
            Factor::ProductCompound(f1, f2) => {
                Factor::convolute_distributions(f1, f2, |a, b| a * b)
            }
            Factor::MaxCompound(f1, f2) => {
                Factor::convolute_distributions(f1, f2, |a, b| std::cmp::max(a, b))
            }
            Factor::MinCompound(f1, f2) => {
                Factor::convolute_distributions(f1, f2, |a, b| std::cmp::min(a, b))
            }
        }
    }

    fn convolute_distributions(
        f1: &Factor,
        f2: &Factor,
        operation: fn(Value, Value) -> Value,
    ) -> Distribution {
        // let mut m: HashMap<Value, Prob> = HashMap::new();
        let mut m = HashMap::<Value, Prob>::new();
        for (v1, p1) in f1.distribution() {
            // println!("loop1 v1:{} p1:{}", v1, p1);
            for (v2, p2) in f2.distribution() {
                // println!("loop2 v2:{} p2:{}", v2, p2);
                let v = operation(v1, v2);
                let p = p1 * p2;
                if m.contains_key(&v) {
                    *m.get_mut(&v).unwrap() += p;
                } else {
                    m.insert(v, p);
                }
            }
        }
        let mut distribution_vec = m.into_iter().collect::<Vec<(Value, Prob)>>();
        distribution_vec.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        return Box::new(distribution_vec.into_iter());
    }
}

impl Mul for Box<Factor> {
    type Output = Box<Factor>;

    fn mul(self, rhs: Self) -> Self::Output {
        return Box::new(Factor::ProductCompound(self, rhs));
    }
}

impl Add for Box<Factor> {
    type Output = Box<Factor>;

    fn add(self, rhs: Self) -> Self::Output {
        return Box::new(Factor::SumCompound(self, rhs));
    }
}
