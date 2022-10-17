#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use serde::{Deserialize, Serialize};

use fraction::{BigFraction, One, ToPrimitive, Zero};
use std::{fmt::Display, ops::Add};

use crate::{dice_string_parser::DiceBuildingError, DiceBuilder};

use super::dice_builder::{AggrValue, Prob, Value};

/// A [`Dice`] represents a discrete probability distribution, providing paramters like mean, standard deviation and the `roll()` method to randomly sample from this distribution
///
/// A [`Dice`] is always created using a [`DiceBuilder`]. The simplest way is to use:
/// ```
/// let dice = Dice::build_from_string("2w6+3")
/// ```
/// which is equivalent to
/// ```
/// let dice_builder = DiceBuilder::from_string("2w6+3")
/// let dice = dice_builder.build()
/// ```
///
/// Values of the distribution are of type [`i64`]
/// The probabilities are of type [`BigFraction`](fraction::BigFraction) from the [`fraction`](fraction) crate.
/// This allows for precise probabilites with infinite precision, at the cost of some slower operations compared to floats, but avoids pitfalls like floating point precision errors.

pub struct Dice {
    /// a string that can be used to recreate the [`DiceBuilder`] that the [`Dice`] was created from.
    pub builder_string: String,
    /// mininum value of the probability distribution
    pub min: Value,
    /// maximum value of the probability distribution
    pub max: Value,
    /// median  of the probability distribution
    pub median: Value,
    /// mode or modes of the probability distribution
    pub mode: Vec<Value>,
    /// mean of the probability distribution
    pub mean: AggrValue,
    /// standard deviation of the probability distribution
    pub sd: AggrValue,
    /// the probability mass function (pmf) of the dice
    ///
    /// tuples of each value and its probability in ascending order (regarding value)
    pub distribution: Vec<(Value, Prob)>,
    /// the cumulative distribution function (cdf) of the dice
    ///
    /// tuples of each value and its cumulative probability in ascending order (regarding value)
    pub cumulative_distribution: Vec<(Value, Prob)>,
}

impl Dice {
    /// uses the `input` to create a [`DiceBuilder`] and calls `build()` on it
    pub fn build_from_string(input: &str) -> Result<Dice, DiceBuildingError> {
        let builder = DiceBuilder::from_string(input)?;
        Ok(builder.build())
    }

    /// uses the `input` to create a [`DiceBuilder`]. Same as [`DiceBuilder::from_string(input)`]
    pub fn builder(input: &str) -> Result<DiceBuilder, DiceBuildingError> {
        DiceBuilder::from_string(input)
    }

    /// builds a [`Dice`] from a given [`DiceBuilder`]
    ///
    /// this method calculates the distribution and all distribution paramters on the fly, to create the [`Dice`].
    /// Depending on the complexity of the `dice_builder` heavy lifting like convoluting probability distributions may take place here.
    pub fn from_builder(dice_builder: DiceBuilder) -> Dice {
        let distribution: Vec<(Value, Prob)> = dice_builder.distribution_iter().collect();
        let max: Value = distribution.last().map(|e| e.0).unwrap();
        let min: Value = distribution.first().map(|e| e.0).unwrap();
        let mut mean: AggrValue = AggrValue::from(0);

        let mut total_probability: Prob = Prob::new(0u64, 1u64);
        let median_prob: Prob = Prob::new(1u64, 2u64);
        // todo median
        let mut median: Option<Value> = None;
        let mut mode: Option<(Vec<Value>, Prob)> = None;

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
                Some((old_vec, p)) => {
                    if prob > *p {
                        mode = Some((vec![val], prob));
                    } else if prob == *p {
                        let newvec: Vec<Value> = [val].iter().chain(old_vec).map(|&x| x).collect();
                        mode = Some((newvec, prob));
                    }
                }
                None => {
                    mode = Some((vec![val], prob));
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

        let accumulated_distribution = accumulated_distribution_from_distribution(&distribution);

        Dice {
            mean,
            sd,
            mode,
            min,
            max,
            median,
            distribution,
            cumulative_distribution: accumulated_distribution,
            builder_string: dice_builder.to_string(),
        }
    }

    /// Rolls a random number for this [`Dice`].
    ///
    /// For this a random float is uniformly sampled over the interval [0,1) and checked against the accumulated discrete porbability distribution of this [`Dice`].
    ///
    /// # Examples
    ///
    /// rolling 2 standard playing dice:
    /// ```
    /// let d : Dice = Dice::build_from_string("2d6");
    /// println!("rolled: {}", d.roll());
    /// //prints something like: "rolled: 9"
    /// ```
    pub fn roll(&self) -> Value {
        let r: f64 = js_sys::Math::random();
        for (val, prob) in self.cumulative_distribution.iter() {
            if prob.to_f64().unwrap() >= r {
                return *val;
            }
        }
        panic! {"Something went wrong in rolling. random value: {r}"}
    }

    /// rolls the [`Dice`] `n` times and returns the results as a vector
    pub fn roll_multiple(&self, n: usize) -> Vec<Value> {
        (1..n).map(|_| self.roll()).collect()
    }

    /// probability that a number sampled from `self` is `value`
    pub fn prob(&self, value: Value) -> Prob {
        match self.distribution.iter().find(|(v, _)| *v == value) {
            None => Prob::zero(),
            Some((_, p)) => p.clone(),
        }
    }

    /// probability that a number sampled from `self` is less than or equal to `value`
    pub fn prob_lte(&self, value: Value) -> Prob {
        let mut lastp: Option<&Prob> = None;
        for (v, p) in self.cumulative_distribution.iter() {
            if *v > value {
                break;
            }
            lastp = Some(p);
        }
        match lastp {
            None => Prob::zero(),
            Some(p) => p.clone(),
        }
    }

    /// probability that a number sampled from `self` is less than `value`
    pub fn prob_lt(&self, value: Value) -> Prob {
        let mut lastp: Option<&Prob> = None;
        for (v, p) in self.cumulative_distribution.iter() {
            if *v >= value {
                break;
            }
            lastp = Some(p);
        }
        match lastp {
            None => Prob::zero(),
            Some(p) => p.clone(),
        }
    }

    /// probability that a number sampled from `self` is greater than or equal to `value`
    pub fn prob_gte(&self, value: Value) -> Prob {
        return Prob::one() - self.prob_lt(value);
    }

    /// probability that a number sampled from `self` is greater than `value`
    pub fn prob_gt(&self, value: Value) -> Prob {
        return Prob::one() - self.prob_lte(value);
    }
}

fn accumulated_distribution_from_distribution(
    distribution: &[(Value, Prob)],
) -> Vec<(Value, Prob)> {
    let mut acc_distr: Vec<(Value, Prob)> = vec![];
    let mut last_acc_prob: Option<Prob> = None;
    for (val, prob) in distribution {
        match last_acc_prob {
            None => {
                acc_distr.push((*val, prob.clone()));
                last_acc_prob = Some(prob.clone());
            }
            Some(acc_p) => {
                let acc_p = acc_p.clone().add(prob.clone());
                last_acc_prob = Some(acc_p.clone());
                acc_distr.push((*val, acc_p));
            }
        }
    }
    acc_distr
}

#[cfg(feature = "wasm")]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct JsDice {
    dice: Dice,
}

#[cfg(feature = "wasm")]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl JsDice {
    #[cfg_attr(feature = "wasm", wasm_bindgen(getter))]
    pub fn builder_string(&self) -> String {
        self.dice.builder_string.to_string()
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen(getter))]
    pub fn min(&self) -> Value {
        self.dice.min
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen(getter))]
    pub fn max(&self) -> Value {
        self.dice.max
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen(getter))]
    pub fn median(&self) -> Value {
        self.dice.median
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen(getter))]
    pub fn mode(&self) -> Vec<Value> {
        self.dice.mode.iter().cloned().collect()
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen(getter))]
    pub fn mean(&self) -> JsFraction {
        JsFraction::from_big_fraction(&self.dice.mean)
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen(getter))]
    pub fn sd(&self) -> JsFraction {
        JsFraction::from_big_fraction(&self.dice.sd)
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen(getter))]
    pub fn distribution(&self) -> wasm_bindgen::JsValue {
        let js_dist = JsDistribution::from_distribution(&self.dice.distribution);
        serde_wasm_bindgen::to_value(&js_dist).unwrap()
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen(getter))]
    pub fn cumulative_distribution(&self) -> wasm_bindgen::JsValue {
        let js_dist = JsDistribution::from_distribution(&self.dice.cumulative_distribution);
        serde_wasm_bindgen::to_value(&js_dist).unwrap()
    }

    pub fn build_from_string(input: &str) -> Result<JsDice, i64> {
        match DiceBuilder::from_string(input) {
            Ok(builder) => Ok(JsDice {
                dice: builder.build(),
            }),
            Err(_) => Err(0),
        }
    }

    pub fn roll(&self) -> Value {
        self.dice.roll()
    }
}

#[cfg(feature = "wasm")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsDistribution {
    pub values: Vec<(Value, JsFraction)>,
}

#[cfg(feature = "wasm")]
impl JsDistribution {
    pub fn from_distribution(dist: &Vec<(Value, Prob)>) -> JsDistribution {
        JsDistribution {
            values: dist
                .iter()
                .map(|e| (e.0, JsFraction::from_big_fraction(&e.1)))
                .collect(),
        }
    }
}

#[cfg(feature = "wasm")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct JsFraction {
    string: String,
    pub float: f32,
}

#[cfg(feature = "wasm")]
impl Display for JsFraction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }
}

#[cfg(feature = "wasm")]
impl JsFraction {
    pub fn from_big_fraction(big_fraction: &BigFraction) -> JsFraction {
        JsFraction {
            string: big_fraction.to_string(),
            float: big_fraction.to_f32().unwrap(),
        }
    }
}

// https://rustwasm.github.io/wasm-bindgen/reference/arbitrary-data-with-serde.html
