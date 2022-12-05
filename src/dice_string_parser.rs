// use std::{slice::Iter, vec};

// use regex::Regex;

use super::dice_builder::{DiceBuilder, Value};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AtomicInputSymbol {
    Constant(Value),
    FairDie { min: Value, max: Value },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OperatorInputSymbol {
    Add,
    Mul,
    SampleSum,
    Div,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SeparatorInputSymbol {
    Comma,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ClosingInputSymbol {
    BClosing,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OpeningInputSymbol {
    BOpening,
    MaxOpening,
    MinOpening,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputSymbol {
    Atomic(AtomicInputSymbol),
    Operator(OperatorInputSymbol),
    Separator(SeparatorInputSymbol),
    Opening(OpeningInputSymbol),
    Closing(ClosingInputSymbol),
}

use AtomicInputSymbol::*;
use ClosingInputSymbol::*;
use InputSymbol::*;
use OpeningInputSymbol::*;
use OperatorInputSymbol::*;
use SeparatorInputSymbol::*;

pub fn string_to_factor(input: &str) -> Result<DiceBuilder, DiceBuildingError> {
    let symbols = string_to_input_symbols(input)?;
    let graph_seq = input_symbols_to_graph_seq(&symbols)?;
    let factor = graph_seq_to_factor(graph_seq);
    Ok(factor)
}

fn string_to_input_symbols(input: &str) -> Result<Vec<InputSymbol>, DiceBuildingError> {
    let input = string_utils::clean_string(input)?;
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
            'M' => symbols.push(Opening(MaxOpening)),
            'm' => symbols.push(Opening(MinOpening)),
            '(' => symbols.push(Opening(BOpening)),
            ')' => symbols.push(Closing(BClosing)),
            ',' => symbols.push(Separator(Comma)),
            '*' => symbols.push(Operator(Mul)),
            'x' => symbols.push(Operator(SampleSum)),
            '+' => symbols.push(Operator(Add)),
            '/' => symbols.push(Operator(Div)),
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
                let max: i64 = match max.parse() {
                    Ok(i) => i,
                    Err(_) => {
                        return Err(DiceBuildingError::NonDigitSymbolAfterDiceD);
                    }
                };

                symbols.push(InputSymbol::Atomic(AtomicInputSymbol::FairDie {
                    min: 1,
                    max,
                }));
            }
            '-' => {
                symbols.push(InputSymbol::Operator(OperatorInputSymbol::Add));
                symbols.push(InputSymbol::Atomic(AtomicInputSymbol::Constant(-1)));
                symbols.push(InputSymbol::Operator(OperatorInputSymbol::Mul));
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
                let n: i64 = match n.parse() {
                    Ok(i) => i,
                    Err(_) => {
                        return Err(DiceBuildingError::NonDigitNumericCharacter);
                    }
                };
                symbols.push(InputSymbol::Atomic(AtomicInputSymbol::Constant(n)));
            }
        }
    }

    // purge empty add symbols, that is all add symbols that are not behind a closing, fairdie or constant
    // example: + "-1" * "d3" => "-1" * "d3"
    symbols = symbols
        .iter()
        .enumerate()
        .filter(|(i, e)| {
            !(**e == InputSymbol::Operator(OperatorInputSymbol::Add)
                && (*i == 0
                    || *i == symbols.len() - 1
                    || match symbols[i - 1] {
                        InputSymbol::Atomic(_) | InputSymbol::Closing(_) => false,

                        _ => true,
                    }))
        })
        .map(|(_, e)| e)
        .cloned()
        .collect();

    Ok(symbols)
}

#[derive(Debug, PartialEq, Eq)]
enum GraphSeq {
    Atomic(DiceBuilder),
    Add(Vec<GraphSeq>),
    Mul(Vec<GraphSeq>),
    Div(Vec<GraphSeq>),
    Min(Vec<GraphSeq>),
    Max(Vec<GraphSeq>),
    SampleSum(Vec<GraphSeq>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum DiceBuildingError {
    UnknownSyntaxError(Vec<InputSymbol>),
    OneInputSymbolButNotAtomic(InputSymbol),
    NonDigitSymbolAfterDiceD,
    NonDigitNumericCharacter,
    /// more closing brackets than opening brackets up to one point
    NegativeScope,
    MultipleOperatorsBehindEachOther,
    EmptySubSequence,
    InvalidCharacterInInput(char),
}

fn input_symbols_to_graph_seq(symbols: &[InputSymbol]) -> Result<GraphSeq, DiceBuildingError> {
    match symbols.len() {
        0 => return Err(DiceBuildingError::EmptySubSequence),
        1 => {
            let sym = symbols[0];
            return match sym {
                Atomic(a) => match a {
                    Constant(i) => Ok(GraphSeq::Atomic(DiceBuilder::Constant(i))),
                    FairDie { min, max } => Ok(GraphSeq::Atomic(DiceBuilder::FairDie { min, max })),
                },
                e => Err(DiceBuildingError::OneInputSymbolButNotAtomic(e)),
            };
        }
        _ => {
            // precedence of operators (high -> low):  x -> * -> / -> +
            // example: 4+3*d3xd2 is  4+(3*(d3xd2))
            // check for operators in ascending precedence to build sequence by splitting on operators:

            // consists of adds in global scope:
            if global_scope_contains_operator(symbols, Add)? {
                return Ok(GraphSeq::Add(split_and_assemble(symbols, Operator(Add))?));
            }

            if global_scope_contains_operator(symbols, Div)? {
                return Ok(GraphSeq::Div(split_and_assemble(symbols, Operator(Div))?));
            }

            if global_scope_contains_operator(symbols, Mul)? {
                return Ok(GraphSeq::Mul(split_and_assemble(symbols, Operator(Mul))?));
            }

            if global_scope_contains_operator(symbols, SampleSum)? {
                return Ok(GraphSeq::SampleSum(split_and_assemble(
                    symbols,
                    Operator(SampleSum),
                )?));
            }

            let first = *symbols.first().unwrap();
            let last = *symbols.last().unwrap();
            return match (first, last) {
                (Opening(o), Closing(_)) => {
                    let symbols_no_first_and_last = &symbols[1..(symbols.len() - 1)];
                    match o {
                        BOpening => Ok(input_symbols_to_graph_seq(symbols_no_first_and_last)?),
                        MaxOpening => Ok(GraphSeq::Max(split_and_assemble(
                            symbols_no_first_and_last,
                            Separator(Comma),
                        )?)),
                        MinOpening => Ok(GraphSeq::Min(split_and_assemble(
                            symbols_no_first_and_last,
                            Separator(Comma),
                        )?)),
                    }
                }
                _ => Err(DiceBuildingError::UnknownSyntaxError(
                    symbols.iter().cloned().collect(),
                )),
            };
        }
    }
}

// fn determineTypeOfGraphSeqBySequentialScan(){
fn global_scope_contains_operator(
    symbols: &[InputSymbol],
    operator: OperatorInputSymbol,
) -> Result<bool, DiceBuildingError> {
    let mut scope_depth: usize = 0;
    for i in 0..symbols.len() {
        if scope_depth == 0 {
            if let InputSymbol::Operator(a) = symbols[i] {
                if a == operator {
                    return Ok(true);
                }
            }
        }
        match symbols[i] {
            InputSymbol::Opening(_) => {
                scope_depth += 1;
            }
            InputSymbol::Closing(_) => {
                if scope_depth == 0 {
                    return Err(DiceBuildingError::NegativeScope);
                }
                scope_depth -= 1;
            }
            _ => (),
        }
    }
    return Ok(false);
}

fn split_and_assemble(
    symbols: &[InputSymbol],
    splitter: InputSymbol,
) -> Result<Vec<GraphSeq>, DiceBuildingError> {
    let segments_or_errors: Vec<Result<_, _>> = symbols
        .split_bracket_aware(splitter)?
        .iter()
        .map(|segment| input_symbols_to_graph_seq(segment))
        .collect();
    let mut segments: Vec<GraphSeq> = vec![];
    for segment in segments_or_errors.into_iter() {
        segments.push(segment?);
    }
    return Ok(segments);
}

trait BracketAwareSplittable {
    fn split_bracket_aware(
        &self,
        splitter: InputSymbol,
    ) -> Result<Vec<&[InputSymbol]>, DiceBuildingError>;
}

impl BracketAwareSplittable for &[InputSymbol] {
    fn split_bracket_aware(
        &self,
        splitter: InputSymbol,
    ) -> Result<Vec<&[InputSymbol]>, DiceBuildingError> {
        let mut index_chunks: Vec<(Option<usize>, Option<usize>)> = vec![(None, None)];
        let mut scope_depth: usize = 0;
        for (i, e) in self.iter().enumerate() {
            if *e == splitter && scope_depth == 0 {
                index_chunks.push((None, None));
            } else {
                let last = index_chunks.last_mut().unwrap();
                match last {
                    (None, None) => {
                        *last = (Some(i), Some(i));
                    }
                    (Some(_), Some(_)) => {
                        *last = (last.0, Some(i));
                    }
                    _ => panic!("should not happen"),
                }
                match *e {
                    InputSymbol::Opening(_) => scope_depth += 1,
                    InputSymbol::Closing(_) => {
                        if scope_depth == 0 {
                            return Err(DiceBuildingError::NegativeScope);
                        }
                        scope_depth -= 1
                    }
                    _ => (),
                }
            }
        }
        for e in index_chunks.iter() {
            match e {
                (None, None) => return Err(DiceBuildingError::MultipleOperatorsBehindEachOther),
                _ => (),
            }
        }
        let res = index_chunks
            .iter()
            .map(|(s, e)| &self[s.unwrap()..=e.unwrap()])
            .collect();
        return Ok(res);
    }
}

fn graph_seq_to_factor(graph_seq: GraphSeq) -> DiceBuilder {
    match graph_seq {
        GraphSeq::Atomic(f) => f,
        GraphSeq::Add(vec) => DiceBuilder::SumCompound(
            vec.into_iter()
                .map(graph_seq_to_factor)
                .collect::<Vec<DiceBuilder>>(),
        ),
        GraphSeq::Mul(vec) => DiceBuilder::ProductCompound(
            vec.into_iter()
                .map(graph_seq_to_factor)
                .collect::<Vec<DiceBuilder>>(),
        ),
        GraphSeq::Min(vec) => DiceBuilder::MinCompound(
            vec.into_iter()
                .map(graph_seq_to_factor)
                .collect::<Vec<DiceBuilder>>(),
        ),
        GraphSeq::Max(vec) => DiceBuilder::MaxCompound(
            vec.into_iter()
                .map(graph_seq_to_factor)
                .collect::<Vec<DiceBuilder>>(),
        ),
        GraphSeq::SampleSum(vec) => DiceBuilder::SampleSumCompound(
            vec.into_iter()
                .map(graph_seq_to_factor)
                .collect::<Vec<DiceBuilder>>(),
        ),
        GraphSeq::Div(vec) => DiceBuilder::DivisionCompound(
            vec.into_iter()
                .map(graph_seq_to_factor)
                .collect::<Vec<DiceBuilder>>(),
        ),
    }
}

mod string_utils {
    use regex::Regex;

    use super::DiceBuildingError;
    const PERMITTED_CHARACTERS: &str = "minax(,)dw0123456789+-*/";
    pub fn clean_string(s: &str) -> Result<String, DiceBuildingError> {
        let mut new_s = String::new();
        for ch in s.to_lowercase().chars() {
            if PERMITTED_CHARACTERS
                .chars()
                .into_iter()
                .any(|ch2| ch2 == ch)
            {
                new_s.push(ch);
            } else if !ch.is_whitespace() {
                return Err(DiceBuildingError::InvalidCharacterInInput(ch));
            }
        }
        let s = &mut new_s;
        s.retain(|c| PERMITTED_CHARACTERS.chars().into_iter().any(|c2| c == c2));
        *s = s.replace("max(", "M");
        *s = s.replace("min(", "m");
        *s = s.replace('w', "d");

        // 3d6 => 3xd6
        add_token_in_string(s, "", r"\d", "d", "", "x");

        // )( => )x(
        add_token_in_string(s, r"\)", "", r"\(", "x", "");

        // )M => )xM
        add_token_in_string(s, r"\)", "", "M", "x", "");

        // )m => )xm
        add_token_in_string(s, r"\)", "", "m", "x", "");

        // 3(...) => 3x(...),   d3(d3) => d3x(d3)
        add_token_in_string(s, r"", r"(\d|d)", r"\(", "", "x");
        Ok(new_s)
    }

    fn add_token_in_string(
        string: &mut String,
        before: &str,
        search_token: &str,
        after: &str,
        put_before_search_token: &str,
        put_after_search_token: &str,
    ) {
        let re = Regex::new(&format!("{}({}){}", before, search_token, after)).unwrap();
        *string = re
            .replace_all(string, &format!("{}□$1■{}", before, after))
            .to_string();
        *string = string
            .replace('□', put_before_search_token)
            .replace('■', put_after_search_token)
            .replace("\\", "");
    }
}

#[cfg(test)]
mod test {
    use std::vec;

    use super::*;

    #[test]
    fn clean_string_test() {
        let input = r#" max(3w6)(3+4)+d3(d3)-3()  min(3,4)       "#.to_owned();

        let input = string_utils::clean_string(&input).unwrap();
        dbg!(&input);
        assert_eq!("M3xd6)x(3+4)+d3x(d3)-3x()xm3,4)", input);
    }
    #[test]
    fn string_to_input_symbols_1() {
        let real: Vec<InputSymbol> = string_to_input_symbols("max(13,2)").unwrap();
        let expected: Vec<InputSymbol> = vec![
            Opening(MaxOpening),
            Atomic(Constant(13)),
            Separator(Comma),
            Atomic(Constant(2)),
            Closing(BClosing),
        ];
        assert_eq!(real, expected);
    }

    #[test]
    fn string_to_input_symbols_2() {
        let real: Vec<InputSymbol> = string_to_input_symbols("4 d32 - 3").unwrap();
        let expected: Vec<InputSymbol> = vec![
            Atomic(Constant(4)),
            Operator(SampleSum),
            Atomic(FairDie { min: 1, max: 32 }),
            Operator(Add),
            Atomic(Constant(-1)),
            Operator(Mul),
            Atomic(Constant(3)),
        ];
        assert_eq!(real, expected);
    }

    mod graph_building {
        use super::*;
        use crate::{
            dice_builder::DiceBuilder,
            dice_string_parser::{input_symbols_to_graph_seq, string_to_input_symbols, GraphSeq},
        };

        #[test]
        /// see if graph in constructed correctly
        fn input_symbols_to_graph_seq_test() {
            let input = "max(1,2,3)";

            let symbols = string_to_input_symbols(input).unwrap();
            assert_eq!(
                symbols,
                vec![
                    Opening(MaxOpening),
                    Atomic(Constant(1)),
                    Separator(Comma),
                    Atomic(Constant(2)),
                    Separator(Comma),
                    Atomic(Constant(3)),
                    Closing(BClosing)
                ]
            );
            let graph = input_symbols_to_graph_seq(&symbols).unwrap();
            let expected_graph = GraphSeq::Max(vec![
                GraphSeq::Atomic(DiceBuilder::Constant(1)),
                GraphSeq::Atomic(DiceBuilder::Constant(2)),
                GraphSeq::Atomic(DiceBuilder::Constant(3)),
            ]);
            assert_eq!(graph, expected_graph);
        }
    }

    mod input_to_factor {
        use crate::dice_builder::AggrValue;
        use crate::dice_string_parser::DiceBuildingError;
        use crate::{
            dice_builder::DiceBuilder,
            dice_string_parser::{graph_seq_to_factor, string_to_factor, GraphSeq},
        };

        #[test]
        fn graph_seq_to_factor_test() {
            let graph = GraphSeq::Max(vec![
                GraphSeq::Atomic(DiceBuilder::Constant(1)),
                GraphSeq::Atomic(DiceBuilder::Constant(2)),
                GraphSeq::Atomic(DiceBuilder::Constant(3)),
            ]);
            let factor = graph_seq_to_factor(graph);
            let expected_factor = DiceBuilder::MaxCompound(vec![
                DiceBuilder::Constant(1),
                DiceBuilder::Constant(2),
                DiceBuilder::Constant(3),
            ]);
            assert_eq!(factor, expected_factor);
        }

        #[test]
        fn string_to_factor_test() {
            let factor = string_to_factor("max(1,2,3)  ").unwrap();
            let expected_factor = DiceBuilder::MaxCompound(vec![
                DiceBuilder::Constant(1),
                DiceBuilder::Constant(2),
                DiceBuilder::Constant(3),
            ]);
            assert_eq!(factor, expected_factor);

            let factor_failed = string_to_factor("max(1:,2,3)  ");
            assert_eq!(
                factor_failed,
                Err(DiceBuildingError::InvalidCharacterInInput(':'))
            );
        }

        #[test]
        fn string_to_factor_test_2() {
            let factor = string_to_factor("4*5+2*3").unwrap();
            let expected_factor = DiceBuilder::SumCompound(vec![
                DiceBuilder::ProductCompound(vec![
                    DiceBuilder::Constant(4),
                    DiceBuilder::Constant(5),
                ]),
                DiceBuilder::ProductCompound(vec![
                    DiceBuilder::Constant(2),
                    DiceBuilder::Constant(3),
                ]),
            ]);
            assert_eq!(factor, expected_factor);

            let factor2 = string_to_factor("26").unwrap();
            let expected_factor_2 = DiceBuilder::Constant(26);
            assert_eq!(factor2, expected_factor_2);
        }

        #[test]
        fn string_to_factor_test_3() {
            let factor = string_to_factor("min(8w5,8w5)+4").unwrap();
            let max = factor
                .build()
                .distribution
                .iter()
                .map(|e| e.0)
                .max()
                .unwrap();
            assert_eq!(max, 44);
        }

        #[test]
        fn test_factor_stats() {
            let factor = DiceBuilder::from_string("2w6").unwrap();
            let stats = factor.build();
            assert_eq!(stats.mean, AggrValue::new(7u64, 1u64));
        }
    }
}
