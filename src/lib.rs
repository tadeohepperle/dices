//! A crate for calculating distrete probability distributions of dice.
//!
//!
//! To create a [`Dice`], build it from a [`DiceBuilder`] or directly from a string:
//! ```
//! use dices::{Dice, DiceBuilder};
//! let dice: Dice = DiceBuilder::from_string("2d6").unwrap().build();
//! let dice: Dice = Dice::build_from_string("2d6").unwrap();
//! ```
//! ---
//! Properties of these dice are calculated in the `build()` function:
//! ```txt
//! min: 2
//! max: 12
//! mode: vec![7],
//! mean: 7,
//! median: 7,
//! distribution: vec![(2, 1/36), (3, 1/18), (4, 1/12), (5, 1/9), (5, 1/9), (6, 5/36), (7, 1/6), ...]
//! cumulative_distribution: vec![(2, 1/36), (3, 1/12), (4, 1/6), ...]
//! ```
//! A DiceBuildingError could be returned, if the `input` string could not be parsed into a proper syntax tree for the [`DiceBuilder`].
//! ---
//! To roll a [`Dice`] call the `roll()` function, for rolling multiple times call the `roll_multiple()` function:
//! ```
//! use dices::Dice;
//! let dice = Dice::build_from_string("2d6").unwrap();
//! let num = dice.roll();
//! let nums = dice.roll_multiple(10);
//! // num will be some i64 between 2 and 12, sampled according to the dices distribution
//! // nums could be vec![7,3,9,11,7,8,5,6,3,6]
//! ```
//!
//! ---
//! # Syntax Examples:
//! Some exaple strings that can be passed into the `DiceBuilder::from_string(input)` function
//!
//! 3 six-sided dice:
//! ```txt
//! "3d6", "3w6" or "3xw6"
//! ```
//! one six-sided die multiplied by 3:
//! ```txt
//! "3*d6" or "d6*3"
//! ```
//! rolling one or two six sided dice and summing them up
//! ```txt
//! "d2xd6"
//! ```
//! the maximum of two six-sided-dice minus the minimum of two six sided dice
//! ```txt
//! "max(d6,d6)-min(d6,d6)""
//! ```
//! rolling a die but any value below 2 becomes 2 and above 5 becomes 5
//! ```txt
//! "min(max(2,d6),5)"
//! ```
//! multiplying 3 20-sided-dice
//! ```txt
//! "d20*d20*d20"
//! ```   
//!
//! # Calculating Probabilities
//!
//!
//!
//! # Background Information
//! This [`crate`] uses the [`BigFraction`](fraction::BigFraction) data type from the [`fraction`](fraction) crate to represent probabilities
//! This is quite nice because it allows for precise probabilities with infinite precision.
//! The drawback is that it is less efficient than using floats.
//!
//! While `"d100*d100"` takes about 100ms for me, something like "d10xd100" took 9.000 ms to finish calculating the probability distribution.
//! There is room for optimization.
//!
//!

#![warn(missing_docs)]
mod dice;
mod dice_builder;
mod dice_string_parser;
mod wasm_safe;

pub use dice::Dice;

pub use dice_builder::DiceBuilder;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

// wasm-pack build --release --features wasm --no-default-features
#[cfg(feature = "wasm")]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
/// just a test function for wasm pack
pub fn greet() -> String {
    format!("Hello, from dices instantly")
}

#[cfg(test)]
mod tests {
    use fraction::{ToPrimitive, Zero};

    use crate::{
        dice_builder::{DiceBuilder, DistributionHashMap, Prob, Value},
        Dice,
    };

    #[test]
    fn adding_distributions_coin_times_2() {
        let f1 = DiceBuilder::Constant(2);
        let f2 = DiceBuilder::FairDie { min: 0, max: 1 };
        let f3 = DiceBuilder::ProductCompound(vec![f1, f2]);
        let dice = f3.build();
        let d_vec = dice.distribution;
        assert_eq!(
            d_vec,
            vec![(0, Prob::new(1u64, 2u64)), (2, Prob::new(1u64, 2u64))]
        );
    }

    #[test]
    fn adding_distributions_two_dice() {
        let f1 = DiceBuilder::FairDie { min: 1, max: 5 };
        let f2 = DiceBuilder::FairDie { min: 1, max: 5 };
        let f3 = DiceBuilder::SumCompound(vec![f1, f2]);
        let dice = f3.build();
        let d_vec = dice.distribution;
        println!("{:?}", d_vec);
        assert_eq!(d_vec[0], (2, Prob::new(1u64, 25u64)));
    }

    #[test]
    fn adding_20_dice() {
        let mut f = Box::new(DiceBuilder::Constant(0));
        for _ in 0..20 {
            f = f + Box::new(DiceBuilder::FairDie { min: 1, max: 6 });
        }

        let maxval = f.build().distribution.iter().map(|e| e.0).max().unwrap();

        assert_eq!(maxval, 120);
    }

    #[test]
    fn sample_sum_convolute_1() {
        let f1 = DiceBuilder::Constant(2);
        let f2 = DiceBuilder::FairDie { min: 1, max: 2 };
        let f = DiceBuilder::SampleSumCompound(Box::new(f1), Box::new(f2));
        let dice = f.build();
        let d = dice.distribution;
        assert_eq!(d, unif(vec![2, 3, 3, 4]));
    }
    #[test]
    /// two dice
    fn sample_sum_convolute_2() {
        let f1 = DiceBuilder::FairDie { min: 1, max: 2 };
        let f2 = DiceBuilder::FairDie { min: 1, max: 2 };
        let f = DiceBuilder::SampleSumCompound(Box::new(f1), Box::new(f2));
        let dice = f.build();
        let d = dice.distribution;
        assert_eq!(d, unif(vec![1, 2, 1, 2, 2, 3, 3, 4]));
    }

    #[test]
    /// 0 or one d2
    fn sample_sum_convolute_3() {
        let f1 = DiceBuilder::FairDie { min: 0, max: 1 };
        let f2 = DiceBuilder::FairDie { min: 1, max: 2 };
        let f = DiceBuilder::SampleSumCompound(Box::new(f1), Box::new(f2));
        let dice = f.build();
        let d = dice.distribution;
        assert_eq!(d, unif(vec![0, 0, 1, 2]));
    }

    #[test]
    /// zero dice
    fn sample_sum_convolute_4() {
        let f1 = DiceBuilder::Constant(0);
        let f2 = DiceBuilder::FairDie { min: 1, max: 6 };
        let f = DiceBuilder::SampleSumCompound(Box::new(f1), Box::new(f2));
        let dice = f.build();
        let d = dice.distribution;
        assert_eq!(d, unif(vec![0]));
    }

    fn unif(v: Vec<Value>) -> Vec<(Value, Prob)> {
        let mut hashmap = DistributionHashMap::new();
        let l = v.len();
        let prob = Prob::new(1u64, l as u64);
        v.iter().for_each(|e| {
            if hashmap.contains_key(e) {
                *hashmap.get_mut(e).unwrap() += &prob;
            } else {
                hashmap.insert(*e, prob.clone());
            }
        });
        let mut distribution_vec = hashmap.into_iter().collect::<Vec<(Value, Prob)>>();
        distribution_vec.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        return distribution_vec;
    }
    #[test]
    fn calculating_accumulated_distribution_test() {
        let dices = vec!["1w6+1", "3w8-3", "max(1,2,3)"];

        let last_elements_of_acc_distr: Vec<Prob> = dices
            .iter()
            .map(|e| {
                DiceBuilder::from_string(&e)
                    .unwrap()
                    .build()
                    .cumulative_distribution
                    .last()
                    .unwrap()
                    .1
                    .clone()
            })
            .collect();
        for e in last_elements_of_acc_distr {
            assert_eq!(e, Prob::new(1u64, 1u64));
        }
    }
    #[test]
    fn test_dice_builder_to_string() {
        let string_in = "1xd6+7";
        let string_out = DiceBuilder::from_string(string_in).unwrap().to_string();
        assert_eq!(string_in, string_out)
    }

    #[test]
    fn test_build_and_mean() {
        let dice_builder = DiceBuilder::from_string("2d6+4").unwrap();
        let dice = dice_builder.build();
        let mean = dice.mean;
        assert_eq!(mean.to_f64().unwrap(), 11.0);
    }

    #[test]
    fn prob_tests() {
        let d = Dice::build_from_string("2w6").unwrap();
        assert_eq!(d.prob(7), Prob::new(1u64, 6u64));
        assert_eq!(d.prob_lt(7), Prob::new(15u64, 36u64));
        assert_eq!(d.prob_gt(7), Prob::new(15u64, 36u64));
        assert_eq!(d.prob_lte(7), Prob::new(21u64, 36u64));
        assert_eq!(d.prob_gte(7), Prob::new(21u64, 36u64));

        assert_eq!(d.prob_lt(-3), Prob::zero());
        assert_eq!(d.prob_lt(-3), Prob::zero());
    }
}
