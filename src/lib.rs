//! A crate for calculating distrete probability distributions of dice.
//!
//!

#![warn(missing_docs)]
mod dice;
mod dice_builder;
mod dice_string_parser;

pub use dice::Dice;
pub use dice_builder::DiceBuilder;

#[cfg(test)]
mod tests {
    use fraction::ToPrimitive;

    use crate::dice_builder::{DiceBuilder, DistributionHashMap, Prob, Value};

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
}
