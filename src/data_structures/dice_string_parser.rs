use super::factor::{Factor, Value};

// pub fn from_string(input: &str) -> Box<Factor> {
//     /*
//     Example input: max(1w10, 1w3+3w2)+3

//     1. remove whitespace
//     2. convert into string of symbols

//     */
// }

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum InputSymbol {
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

        let flush = || {};
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
                println!("n is {n}");
                let n: i64 = n.parse().unwrap();
                symbols.push(InputSymbol::Constant(n));
            }
        }
    }

    symbols
}

enum GraphSeq {
    Atomic(Factor),
    Add(Vec<GraphSeq>),
    Mul(Vec<GraphSeq>),
    Min(Vec<GraphSeq>),
    Max(Vec<GraphSeq>),
    SampleSum(Box<GraphSeq>, Box<GraphSeq>),
}

pub enum GraphBuildingError {
    GraphSeqWithoutVec,
    AddSymbolInNonAddSequence,
    MulSymbolWithoutAnElementInCurrentSequence,
    SampleSumSymbolWithoutAnElementInCurrentSequence,
    SequenceHierarchyEmpty,
    CommaSymbolInAddSequence,
}

fn input_symbols_to_graph_seq(symbols: &Vec<InputSymbol>) -> Result<GraphSeq, GraphBuildingError> {
    let is_max_compound = symbols_indicate_max_compound(&symbols);
    let is_min_compung = symbols_indicate_min_compound(&symbols);

    let add_partitioning = partition_input_symbols_bracket_aware(&symbols, InputSymbol::Add);
    let add_partitioning = partition_input_symbols_bracket_aware(&symbols, InputSymbol::Add);

    todo!()
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

fn graph_seq_to_factor(g: GraphSeq) -> Factor {
    todo!()
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
        *s = re_dice_with_factor.replace(s, "$1xd").to_string();
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
        assert_eq!("M3*d6)", input);
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
        let real: Vec<InputSymbol> = string_to_input_symbols("4 w32 - 3");
        let expected: Vec<InputSymbol> = vec![
            InputSymbol::Constant(4),
            InputSymbol::Multiply,
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
}
