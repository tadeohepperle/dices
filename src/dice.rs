use std::ops::Add;

use fraction::ToPrimitive;

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
///
#[derive(Debug)]
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
        let r: f64 = rand::random();
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
