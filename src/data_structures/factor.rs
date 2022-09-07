use std::iter::*;

type Value = i64;
type Prob = f64;

/// a constant value like the 3 in front of 3W6
#[derive(Debug, PartialEq, Eq)]
pub struct ConstantFactor(Value);

// impl Factor for ConstantFactor {
//     fn all_values(&self) -> Box<dyn Iterator<Item = (Value, Prob)>> {
//         Box::new(once((self.0, 1.0)))
//     }
// }
/// a fair die
pub struct FairDie {
    min: i64,
    max: i64,
}

pub trait Factor {
    fn all_values(&self) {}
}
