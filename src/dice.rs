use std::ops::Add;

use fraction::ToPrimitive;

use crate::{dice_string_parser::DiceBuildingError, DiceBuilder};

use super::dice_builder::{AggrValue, Prob, Value};

#[derive(Debug)]
pub struct Dice {
    pub builder_string: String,
    pub min: Value,
    pub max: Value,
    pub median: Value,
    pub mode: Value,
    pub mean: AggrValue,
    pub sd: AggrValue,
    pub distribution: Vec<(Value, Prob)>,
    pub accumulated_distribution: Vec<(Value, Prob)>,
}

impl Dice {
    pub fn build_from_string(input: &str) -> Result<Dice, DiceBuildingError> {
        let builder = DiceBuilder::from_string(input)?;
        Ok(builder.build())
    }

    pub fn builder(input: &str) -> Result<DiceBuilder, DiceBuildingError> {
        DiceBuilder::from_string(input)
    }

    pub fn from_builder(dice_builder: DiceBuilder) -> Dice {
        let distribution: Vec<(Value, Prob)> = dice_builder.distribution_iter().collect();
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

        let accumulated_distribution = accumulated_distribution_from_distribution(&distribution);

        Dice {
            mean,
            sd,
            mode,
            min,
            max,
            median,
            distribution,
            accumulated_distribution,
            builder_string: dice_builder.to_string(),
        }
    }

    pub fn roll(&self) -> Value {
        let r: f64 = rand::random();
        for (val, prob) in self.accumulated_distribution.iter() {
            if prob.to_f64().unwrap() >= r {
                return *val;
            }
        }
        panic! {"Something went wrong in rolling. random value: {r}"}
    }

    /// get the discrete probability distribution
    ///
    /// this is a simple getter, calculating the distribution is performed in the DiceBuilder::build() function
    pub fn distribution(&self) -> &[(Value, Prob)] {
        &self.distribution
    }

    pub fn accumulated_distribution(&self) -> &[(Value, Prob)] {
        &self.accumulated_distribution
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
