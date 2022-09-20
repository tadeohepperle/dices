use super::{
    dice::Dice,
    dice_string_parser::{self, DiceBuildingError},
};
use core::panic;
use std::{
    collections::HashMap,
    ops::{Add, Mul},
};
pub type Value = i64;
pub type Prob = fraction::BigFraction;
pub type AggrValue = fraction::BigFraction;
type Distribution = Box<dyn Iterator<Item = (Value, Prob)>>;
pub type DistributionHashMap = HashMap<Value, Prob>;

/// A [`DiceBuilder`] tree-like data structure representing the components of a dice formula like `max(2d6+4,d20)`
///
/// The tree can be used to calculate a discrete probability distribution. This happens when the `build()` method is called and creates a [`Dice`].
///
/// # Examples
/// ```
/// let dice_builder = DiceBuilder::from_string("2d6+4").unwrap();
/// let dice = dice_builder.build();
/// let mean = dice.mean.to_f64().unwrap();
/// assert_eq!(mean, 11.0);
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum DiceBuilder {
    /// A constant value (i64) that does not
    Constant(Value),
    /// A discrete uniform distribution over the integer interval `[min, max]`
    FairDie {
        /// minimum value of the die, inclusive
        min: Value,
        /// maximum value of the die, inclusive
        max: Value,
    },
    /// the sum of multiple [DiceBuilder] instances, like: d6 + 3 + d20
    SumCompound(Vec<DiceBuilder>),
    /// the product of multiple [DiceBuilder] instances, like: d6 * 3 * d20
    ProductCompound(Vec<DiceBuilder>),
    /// the maximum of multiple [DiceBuilder] instances, like: max(d6,3,d20)
    MaxCompound(Vec<DiceBuilder>),
    /// the minimum of multiple [DiceBuilder] instances, like: min(d6,3,d20)
    MinCompound(Vec<DiceBuilder>),
    /// SampleSumCompound(a,b) can be interpreted as follows:
    /// A [`DiceBuilder`] `b` is sampled `a` times independently of each other.
    ///
    /// # Examples
    /// throwing 5 six-sided dice:
    /// ```
    /// let five_six_sided_dice = SampleSumCompound(
    ///     Box::new(Constant(5)),
    ///     Box::new(FairDie{min: 1, max: 6})
    /// )
    /// ```
    ///
    /// throwing 1, 2 or 3 (randomly determined) six-sided and summing them up:
    /// ```
    /// let dice_1_2_or_3 = SampleSumCompound(
    ///     Box::new(FairDie{min: 1, max: 3}),
    ///     Box::new(FairDie{min: 1, max: 6})
    /// )
    /// ```
    ///
    /// for two constants, it is the same as multiplication:
    /// ```
    /// let b1 = SampleSumCompound(
    ///     Box::new(Constant(2)),
    ///     Box::new(Constant(3))
    /// )
    /// let b2 = ProductCompound(vec![Constant(2),Constant(3)])
    /// assert_eq!(b1.build(), b2.build())
    ///
    /// ```
    SampleSumCompound(Box<DiceBuilder>, Box<DiceBuilder>),
}

impl DiceBuilder {
    /// parses the string into a tree-like structure to create a [`DiceBuilder`]
    ///
    /// # Syntax Examples:
    /// |-----|
    /// |     |
    /// 4 six-sided dice: "4d6"
    ///
    /// # Examples:
    /// throwing 3 six-sided dice:
    /// ```
    /// let builder = DiceBuilder::from_string("3d6")
    /// let builder_2 = DiceBuilder::from_string("3 d6  ")
    /// let builder_2 = DiceBuilder::from_string("3xd6") // explicitly using sample sum
    /// assert_eq!(builder, builder_2)
    /// assert_eq!(builder_2, builder_3)
    /// ```
    ///
    /// the minimum and maximum of multiple dice:
    /// ```
    /// let min_builder = DiceBuilder::from_string("min(d6,d6)")
    /// let max_builder = DiceBuilder::from_string("max(d6,d6,d20)")
    /// ```
    ///
    pub fn from_string(input: &str) -> Result<Self, DiceBuildingError> {
        dice_string_parser::string_to_factor(input)
    }

    /// builds a [`Dice`] from [`self`]
    ///
    /// this method calculates the distribution and all distribution paramters on the fly, to create the [`Dice`].
    /// Depending on the complexity of the `dice_builder` heavy lifting like convoluting probability distributions may take place here.
    pub fn build(self) -> Dice {
        Dice::from_builder(self)
    }

    /// shortcut for `DiceBuilder::from_string(input).build()`
    pub fn build_from_string(input: &str) -> Result<Dice, DiceBuildingError> {
        let builder = DiceBuilder::from_string(input)?;
        Ok(builder.build())
    }

    /// constructs a string from the DiceBuilder that can be used to reconstruct an equivalent DiceBuilder from it.
    ///
    /// currently fails to construct a correct string in case dices with a non-1 minimum are present. This is because there is no string notation for dices with a non-1 minimum yet.
    pub fn to_string(&self) -> String {
        match self {
            DiceBuilder::Constant(i) => i.to_string(),
            DiceBuilder::FairDie { min, max } => match *min == 1 {
                true => format!("d{max}"),
                false => "".to_owned(), // this is currently a weak point where errors can occur
            },
            DiceBuilder::SumCompound(v) => v
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>()
                .join("+"),
            DiceBuilder::ProductCompound(v) => v
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>()
                .join("*"),
            DiceBuilder::MaxCompound(v) => format!(
                "max({})",
                v.iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            DiceBuilder::MinCompound(v) => format!(
                "min({})",
                v.iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            DiceBuilder::SampleSumCompound(a, b) => format!("{}x{}", a.to_string(), b.to_string()),
        }
    }

    fn distribution_hashmap(&self) -> DistributionHashMap {
        match self {
            DiceBuilder::Constant(v) => {
                let mut m = DistributionHashMap::new();
                m.insert(*v, Prob::new(1u64, 1u64));
                m
            }
            DiceBuilder::FairDie { min, max } => {
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

            DiceBuilder::SumCompound(vec) => {
                let hashmaps = vec
                    .iter()
                    .map(|e| e.distribution_hashmap())
                    .collect::<Vec<DistributionHashMap>>();
                convolute_hashmaps(&hashmaps, |a, b| a + b)
            }
            DiceBuilder::ProductCompound(vec) => {
                let hashmaps = vec
                    .iter()
                    .map(|e| e.distribution_hashmap())
                    .collect::<Vec<DistributionHashMap>>();
                convolute_hashmaps(&hashmaps, |a, b| a * b)
            }
            DiceBuilder::MaxCompound(vec) => {
                let hashmaps = vec
                    .iter()
                    .map(|e| e.distribution_hashmap())
                    .collect::<Vec<DistributionHashMap>>();
                convolute_hashmaps(&hashmaps, std::cmp::max)
            }
            DiceBuilder::MinCompound(vec) => {
                let hashmaps = vec
                    .iter()
                    .map(|e| e.distribution_hashmap())
                    .collect::<Vec<DistributionHashMap>>();
                convolute_hashmaps(&hashmaps, std::cmp::min)
            }
            DiceBuilder::SampleSumCompound(f1, f2) => sample_sum_convolute_two_hashmaps(
                f1.distribution_hashmap(),
                f2.distribution_hashmap(),
            ),
        }
    }

    /// iterator for the probability mass function (pmf) of the [`DiceBuilder`], with tuples for each value with its probability in ascending order (regarding value)
    ///
    /// Calculates the distribution and all distribution paramters.
    /// Depending on the complexity of [`self`] heavy lifting like convoluting probability distributions may take place here.
    pub fn distribution_iter(&self) -> Distribution {
        let mut distribution_vec = self
            .distribution_hashmap()
            .into_iter()
            .collect::<Vec<(Value, Prob)>>();
        distribution_vec.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        Box::new(distribution_vec.into_iter())
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

impl Mul for Box<DiceBuilder> {
    type Output = Box<DiceBuilder>;

    fn mul(self, rhs: Self) -> Self::Output {
        Box::new(DiceBuilder::ProductCompound(vec![*self, *rhs]))
    }
}

impl Add for Box<DiceBuilder> {
    type Output = Box<DiceBuilder>;

    fn add(self, rhs: Self) -> Self::Output {
        Box::new(DiceBuilder::SumCompound(vec![*self, *rhs]))
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
