mod data_structures;

#[cfg(test)]
mod tests {
    use crate::data_structures::factor::{Factor, Prob};

    #[test]
    fn adding_distributions_coin_times_2() {
        let f1 = Factor::Constant(2);
        let f2 = Factor::FairDie { min: 0, max: 1 };
        let f3 = Factor::ProductCompound(vec![Box::new(f1), Box::new(f2)]);
        let d_vec = f3.distribution_vec();
        println!("{:?}", d_vec);

        assert_eq!(
            d_vec,
            vec![(0, Prob::new(1u64, 2u64)), (2, Prob::new(1u64, 2u64))]
        );
    }

    #[test]
    fn adding_distributions_two_dice() {
        let f1 = Factor::FairDie { min: 1, max: 5 };
        let f2 = Factor::FairDie { min: 1, max: 5 };
        let f3 = Factor::SumCompound(vec![Box::new(f1), Box::new(f2)]);
        let d_vec = f3.distribution_vec();
        println!("{:?}", d_vec);
        assert_eq!(d_vec[0], (2, Prob::new(1u64, 25u64)));
    }

    #[test]
    fn adding_20_dice() {
        let mut f = Factor::boxed_zero();
        for _ in 0..20 {
            f = f + Box::new(Factor::FairDie { min: 1, max: 6 });
        }

        let maxval = f.distribution_vec().iter().map(|e| e.0).max().unwrap();

        assert_eq!(maxval, 120);
    }
}
