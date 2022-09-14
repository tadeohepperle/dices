use std::vec;

use super::factor::{Factor, Value};

// pub fn from_string(input: &str) -> Box<Factor> {
//     /*
//     Example input: max(1w10, 1w3+3w2)+3

//     1. remove whitespace
//     2. convert into string of symbols

//     */
// }

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputSymbol {
    Constant(Value),
    FairDie { min: Value, max: Value },
    Add,
    Multiply,
    Comma,
    Closing,
    Opening,
    MaxOpening,
    MinOpening,
    SampleSum,
}

impl InputSymbol {
    fn is_atomic(&self) -> bool {
        return match &self {
            Self::FairDie { min, max } => true,
            Self::Constant(_) => true,
            _ => false,
        };
    }

    fn is_opening(&self) -> bool {
        match self {
            InputSymbol::MinOpening => true,
            InputSymbol::MaxOpening => true,
            InputSymbol::Opening => true,
            _ => false,
        }
    }
    fn is_closing(&self) -> bool {
        match self {
            InputSymbol::Closing => true,
            _ => false,
        }
    }
}

pub fn string_to_factor(input: &str) -> Result<Factor, GraphBuildingError> {
    let symbols = string_to_input_symbols(input);
    let graph_seq = input_symbols_to_graph_seq(&symbols)?;
    let factor = graph_seq_to_factor(graph_seq);
    Ok(factor)
}

fn string_to_input_symbols(input: &str) -> Vec<InputSymbol> {
    let mut input = input.to_owned();
    string_utils::clean_string(&mut input);
    let mut symbols: Vec<InputSymbol> = vec![];

    let mut char_iterator = input.chars();
    let mut last_taken_not_processed: Option<char> = None;
    'outer: loop {
        let c = match last_taken_not_processed {
            Some(a) => {
                last_taken_not_processed = None;
                a
            }
            None => match char_iterator.next() {
                Some(e) => e,
                None => break 'outer,
            },
        };

        match c {
            'M' => {
                symbols.push(InputSymbol::MaxOpening);
            }
            'm' => symbols.push(InputSymbol::MinOpening),
            '(' => symbols.push(InputSymbol::Opening),
            ')' => symbols.push(InputSymbol::Closing),
            ',' => symbols.push(InputSymbol::Comma),
            '*' => symbols.push(InputSymbol::Multiply),
            'x' => symbols.push(InputSymbol::SampleSum),
            '+' => symbols.push(InputSymbol::Add),
            'd' => {
                let mut num_char_vec: Vec<char> = vec![];
                'inner: loop {
                    let c2 = match char_iterator.next() {
                        Some(e) => e,
                        None => break 'inner,
                    };
                    if c2.is_numeric() {
                        num_char_vec.push(c2)
                    } else {
                        last_taken_not_processed = Some(c2);
                        break;
                    }
                }
                let max: String = num_char_vec.into_iter().collect();
                let max: i64 = max.parse().unwrap();
                symbols.push(InputSymbol::FairDie { min: 1, max });
            }
            '-' => {
                symbols.push(InputSymbol::Add);
                symbols.push(InputSymbol::Constant(-1));
                symbols.push(InputSymbol::Multiply);
            }
            n => {
                let mut num_char_vec: Vec<char> = vec![n];
                'inner: loop {
                    let c2 = match char_iterator.next() {
                        Some(e) => e,
                        None => break 'inner,
                    };
                    if c2.is_numeric() {
                        num_char_vec.push(c2)
                    } else {
                        last_taken_not_processed = Some(c2);
                        break;
                    }
                }
                let n: String = num_char_vec.into_iter().collect();
                let n: i64 = n.parse().unwrap();
                symbols.push(InputSymbol::Constant(n));
            }
        }
    }

    symbols
}

#[derive(Debug, PartialEq, Eq)]
enum GraphSeq {
    Atomic(Factor),
    Add(Vec<GraphSeq>),
    Mul(Vec<GraphSeq>),
    Min(Vec<GraphSeq>),
    Max(Vec<GraphSeq>),
    SampleSum(Box<GraphSeq>, Box<GraphSeq>),
}

#[derive(Debug)]
pub enum GraphBuildingError {
    GraphSeqWithoutVec,
    AddSymbolInNonAddSequence,
    MulSymbolWithoutAnElementInCurrentSequence,
    SampleSumSymbolWithoutAnElementInCurrentSequence,
    SequenceHierarchyEmpty,
    CommaSymbolInAddSequence,
    MoreThan2ElementsUsedForSampleSum,
    UnknownSyntaxError(Vec<InputSymbol>),
    OneInputSymbolButNotAtomic(InputSymbol),
}

fn input_symbols_to_graph_seq(symbols: &Vec<InputSymbol>) -> Result<GraphSeq, GraphBuildingError> {
    if symbols.len() == 1 {
        let sym = symbols[0];
        match sym {
            InputSymbol::Constant(i) => return Ok(GraphSeq::Atomic(Factor::Constant(i))),
            InputSymbol::FairDie { min, max } => {
                return Ok(GraphSeq::Atomic(Factor::FairDie { min, max }))
            }
            e => return Err(GraphBuildingError::OneInputSymbolButNotAtomic(e)),
        }
    }
    let is_pure_bracket_compound = symbols_indicate_pure_bracket_compund(&symbols);
    if is_pure_bracket_compound {
        let reduced_vec = vec_without_last_and_first(&symbols);
        return input_symbols_to_graph_seq(&reduced_vec);
    }
    let is_max_compound = symbols_indicate_max_compound(&symbols);

    let is_min_compound = symbols_indicate_min_compound(&symbols);
    if is_max_compound || is_min_compound {
        let reduced_vec = vec_without_last_and_first(&symbols);
        let parts = partition_input_symbols_bracket_aware(&reduced_vec, InputSymbol::Comma);
        let mut sub_sequences: Vec<GraphSeq> = vec![];
        for p in parts {
            let graph_seq_for_p = input_symbols_to_graph_seq(&p)?;
            sub_sequences.push(graph_seq_for_p);
        }
        if is_max_compound {
            return Ok(GraphSeq::Max(sub_sequences));
        }
        if is_min_compound {
            return Ok(GraphSeq::Min(sub_sequences));
        }
        panic!("should never get here");
    }

    let add_partitioning = partition_input_symbols_bracket_aware(&symbols, InputSymbol::Add);
    if add_partitioning.len() >= 2 {
        let sub_sequences = input_symbol_partitioning_to_sub_sequnces(add_partitioning)?;
        return Ok(GraphSeq::Add(sub_sequences));
    }
    let mul_partitioning = partition_input_symbols_bracket_aware(&symbols, InputSymbol::Multiply);
    if mul_partitioning.len() >= 2 {
        let sub_sequences = input_symbol_partitioning_to_sub_sequnces(mul_partitioning)?;
        return Ok(GraphSeq::Mul(sub_sequences));
    }
    let sample_sum_partitioning =
        partition_input_symbols_bracket_aware(&symbols, InputSymbol::SampleSum);

    if sample_sum_partitioning.len() >= 2 {
        if sample_sum_partitioning.len() > 2 {
            return Err(GraphBuildingError::MoreThan2ElementsUsedForSampleSum);
        }
        let count_seq = input_symbols_to_graph_seq(&sample_sum_partitioning[0])?;
        let sample_seq = input_symbols_to_graph_seq(&sample_sum_partitioning[1])?;
        return Ok(GraphSeq::SampleSum(
            Box::new(count_seq),
            Box::new(sample_seq),
        ));
    }
    println!("{:?}", symbols);
    return Err(GraphBuildingError::UnknownSyntaxError(symbols.clone()));
}

fn input_symbol_partitioning_to_sub_sequnces(
    partitioning: Vec<Vec<InputSymbol>>,
) -> Result<Vec<GraphSeq>, GraphBuildingError> {
    let mut sub_sequences: Vec<GraphSeq> = vec![];
    for p in partitioning {
        let graph_seq_for_p = input_symbols_to_graph_seq(&p)?;
        sub_sequences.push(graph_seq_for_p);
    }
    return Ok(sub_sequences);
}

fn symbols_indicate_max_compound(symbols: &Vec<InputSymbol>) -> bool {
    if let Some(InputSymbol::MaxOpening) = symbols.first() {
        if let Some(InputSymbol::Closing) = symbols.last() {
            return true;
        }
    }
    false
}

fn symbols_indicate_min_compound(symbols: &Vec<InputSymbol>) -> bool {
    if let Some(InputSymbol::MinOpening) = symbols.first() {
        if let Some(InputSymbol::Closing) = symbols.last() {
            return true;
        }
    }
    false
}

fn symbols_indicate_pure_bracket_compund(symbols: &Vec<InputSymbol>) -> bool {
    if let Some(InputSymbol::Opening) = symbols.first() {
        if let Some(InputSymbol::Closing) = symbols.last() {
            return true;
        }
    }
    false
}

fn graph_seq_to_factor(graph_seq: GraphSeq) -> Factor {
    match graph_seq {
        GraphSeq::Add(vec) => {
            return Factor::SumCompound(
                vec.into_iter()
                    .map(move |g| graph_seq_to_factor(g))
                    .collect::<Vec<Factor>>(),
            );
        }
        GraphSeq::Atomic(f) => f,
        GraphSeq::Mul(vec) => {
            return Factor::ProductCompound(
                vec.into_iter()
                    .map(move |g| graph_seq_to_factor(g))
                    .collect::<Vec<Factor>>(),
            );
        }
        GraphSeq::Min(vec) => {
            return Factor::MinCompound(
                vec.into_iter()
                    .map(move |g| graph_seq_to_factor(g))
                    .collect::<Vec<Factor>>(),
            );
        }
        GraphSeq::Max(vec) => {
            return Factor::MaxCompound(
                vec.into_iter()
                    .map(move |g| graph_seq_to_factor(g))
                    .collect::<Vec<Factor>>(),
            );
        }
        GraphSeq::SampleSum(g1, g2) => {
            return Factor::SampleSumCompound(
                Box::new(graph_seq_to_factor(*g1)),
                Box::new(graph_seq_to_factor(*g2)),
            );
        }
    }
}

fn vec_without_last_and_first(vec: &Vec<InputSymbol>) -> Vec<InputSymbol> {
    vec.iter()
        .skip(1)
        .rev()
        .skip(1)
        .rev()
        .map(|e| e.clone())
        .collect::<Vec<InputSymbol>>()
}

fn partition_input_symbols_bracket_aware(
    input_symbols: &Vec<InputSymbol>,
    sep_symbol: InputSymbol,
) -> Vec<Vec<InputSymbol>> {
    let mut bracket_level = 0;
    let mut result: Vec<Vec<InputSymbol>> = vec![vec![]];
    for i in input_symbols {
        if *i == sep_symbol && bracket_level == 0 {
            result.push(vec![]);
        } else {
            match result.last_mut() {
                None => panic!("result has no last element"),
                Some(last) => {
                    last.push(*i);
                }
            }
            if i.is_opening() {
                bracket_level += 1;
            }
            if i.is_closing() {
                bracket_level -= 1;
            }
        }
    }
    result
}

mod string_utils {
    use regex::Regex;
    const PERMITTED_CHARACTERS: &str = "minax(,)dw0123456789+-*";
    // pub fn remove_from_string(input: &str, remove: &str) -> String {
    //     let re = Regex::new(remove).unwrap();
    //     return re.replace_all(input, "").to_string();
    // }

    pub fn clean_string(s: &mut String) {
        *s = s.to_lowercase();
        s.retain(|c| PERMITTED_CHARACTERS.chars().into_iter().any(|c2| c == c2));
        *s = s.replace("max(", "M");
        *s = s.replace("min(", "m");
        *s = s.replace("w", "d");
        let re_dice_with_factor = Regex::new(r"(\d)d").unwrap();
        *s = re_dice_with_factor.replace_all(s, "$1/d").to_string();
        *s = s.replace("/", "x");
    }
}

#[cfg(test)]
mod test {
    use std::vec;

    use super::*;

    #[test]
    fn clean_string_test() {
        let mut input = r#" max(3w6)        "#.to_owned();
        string_utils::clean_string(&mut input);
        assert_eq!("M3xd6)", input);
    }
    #[test]
    fn string_to_input_symbols_1() {
        let real: Vec<InputSymbol> = string_to_input_symbols("max(13,2)");
        let expected: Vec<InputSymbol> = vec![
            InputSymbol::MaxOpening,
            InputSymbol::Constant(13),
            InputSymbol::Comma,
            InputSymbol::Constant(2),
            InputSymbol::Closing,
        ];
        assert_eq!(real, expected);
    }
    #[test]
    fn string_to_input_symbols_2() {
        let real: Vec<InputSymbol> = string_to_input_symbols("4 d32 - 3");
        let expected: Vec<InputSymbol> = vec![
            InputSymbol::Constant(4),
            InputSymbol::SampleSum,
            InputSymbol::FairDie { min: 1, max: 32 },
            InputSymbol::Add,
            InputSymbol::Constant(-1),
            InputSymbol::Multiply,
            InputSymbol::Constant(3),
        ];
        assert_eq!(real, expected);
    }
    #[test]
    fn partition_input_symbols_bracket_aware_test() {
        // 5+3*6*max(3+4)+5
        let symbols = vec![
            InputSymbol::Constant(5),
            InputSymbol::Add,
            InputSymbol::Constant(3),
            InputSymbol::Multiply,
            InputSymbol::Constant(6),
            InputSymbol::Multiply,
            InputSymbol::MaxOpening,
            InputSymbol::Constant(3),
            InputSymbol::Add,
            InputSymbol::Constant(4),
            InputSymbol::Closing,
            InputSymbol::Add,
            InputSymbol::Constant(5),
        ];
        let expected = vec![
            vec![InputSymbol::Constant(5)],
            vec![
                InputSymbol::Constant(3),
                InputSymbol::Multiply,
                InputSymbol::Constant(6),
                InputSymbol::Multiply,
                InputSymbol::MaxOpening,
                InputSymbol::Constant(3),
                InputSymbol::Add,
                InputSymbol::Constant(4),
                InputSymbol::Closing,
            ],
            vec![InputSymbol::Constant(5)],
        ];

        let res = partition_input_symbols_bracket_aware(&symbols, InputSymbol::Add);
        assert_eq!(res, expected);
    }

    mod graph_building {
        use crate::data_structures::{
            dice_string_parser::{
                input_symbols_to_graph_seq, string_to_input_symbols, GraphSeq, InputSymbol,
            },
            factor::Factor,
        };

        #[test]
        /// see if graph in constructed correctly
        fn input_symbols_to_graph_seq_test() {
            let input = "max(1,2,3)";
            let symbols = string_to_input_symbols(input);
            assert_eq!(
                symbols,
                vec![
                    InputSymbol::MaxOpening,
                    InputSymbol::Constant(1),
                    InputSymbol::Comma,
                    InputSymbol::Constant(2),
                    InputSymbol::Comma,
                    InputSymbol::Constant(3),
                    InputSymbol::Closing
                ]
            );
            let graph = input_symbols_to_graph_seq(&symbols).unwrap();
            let expected_graph = GraphSeq::Max(vec![
                GraphSeq::Atomic(Factor::Constant(1)),
                GraphSeq::Atomic(Factor::Constant(2)),
                GraphSeq::Atomic(Factor::Constant(3)),
            ]);
            assert_eq!(graph, expected_graph);
        }
    }

    mod input_to_factor {
        use crate::data_structures::{
            dice_string_parser::{
                graph_seq_to_factor, string_to_factor, test::input_to_factor, GraphSeq,
            },
            factor::Factor,
        };

        #[test]
        fn graph_seq_to_factor_test() {
            let graph = GraphSeq::Max(vec![
                GraphSeq::Atomic(Factor::Constant(1)),
                GraphSeq::Atomic(Factor::Constant(2)),
                GraphSeq::Atomic(Factor::Constant(3)),
            ]);
            let factor = graph_seq_to_factor(graph);
            let expected_factor = Factor::MaxCompound(vec![
                Factor::Constant(1),
                Factor::Constant(2),
                Factor::Constant(3),
            ]);
            assert_eq!(factor, expected_factor);
        }

        #[test]
        fn string_to_factor_test() {
            let factor = string_to_factor("max(1 ,2,3)  ").unwrap();
            let expected_factor = Factor::MaxCompound(vec![
                Factor::Constant(1),
                Factor::Constant(2),
                Factor::Constant(3),
            ]);
            assert_eq!(factor, expected_factor);
        }

        #[test]
        fn string_to_factor_test_2() {
            let factor = string_to_factor("4*5+2*3").unwrap();
            let expected_factor = Factor::SumCompound(vec![
                Factor::ProductCompound(vec![Factor::Constant(4), Factor::Constant(5)]),
                Factor::ProductCompound(vec![Factor::Constant(2), Factor::Constant(3)]),
            ]);
            assert_eq!(factor, expected_factor);

            let factor2 = string_to_factor("26").unwrap();
            let expected_factor_2 = Factor::Constant(26);
            assert_eq!(factor2, expected_factor_2);
        }

        #[test]
        fn string_to_factor_test_3() {
            let factor = string_to_factor("min(8w5,8w5)+4").unwrap();
            let max = factor.distribution_vec().iter().map(|e| e.0).max().unwrap();
            assert_eq!(max, 44);
        }
    }
}
