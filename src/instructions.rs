use crate::{
    utilities::{precision_of_value, RemoveUncertaintyDigits},
    RNG,
};
use std::collections::HashMap;

use rand::Rng;

#[derive(PartialEq, Debug, Clone)]
enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Range,
    None,
}

impl From<&str> for Operator {
    fn from(s: &str) -> Self {
        match s {
            "+" => Operator::Add,
            "-" => Operator::Subtract,
            "*" => Operator::Multiply,
            "/" => Operator::Divide,
            "^" => Operator::Power,
            "--" => Operator::Range,
            _ => Operator::None,
        }
    }
}

#[derive(Debug, Clone)]
struct Instruction {
    keyword: String,
    operator: Operator,
    value_a: f64,
    value_b: Option<f64>,
}

// TODO: Make this a command line option.
const KEEP_PRECISION: bool = true;

impl Instruction {
    fn apply(&self, cif_value: String) -> anyhow::Result<String> {
        let value = cif_value.remove_uncertainty_digits();

        let value_precision = precision_of_value(&value);

        let value = value.parse::<f64>()?;

        let new_value = match self.operator {
            Operator::Add => value + self.value_a,
            Operator::Subtract => value - self.value_a,
            Operator::Multiply => value * self.value_a,
            Operator::Divide => value / self.value_a,
            Operator::Power => value.powf(self.value_a),
            Operator::Range => {
                if let Some(value_b) = self.value_b {
                    let lower_value = self.value_a.min(value_b);
                    let upper_value = self.value_a.max(value_b);

                    if lower_value == upper_value {
                        return Err(anyhow::anyhow!(format!(
                            "Lower and upper values are the same - {:?}",
                            self
                        )));
                    }

                    RNG.with(|rng| rng.borrow_mut().gen_range(lower_value..upper_value))
                } else {
                    let lower_value = self.value_a.min(value);
                    let upper_value = self.value_a.max(value);

                    if lower_value == upper_value {
                        return Err(anyhow::anyhow!(format!(
                            "Lower and upper values are the same - {:?}",
                            self
                        )));
                    }

                    RNG.with(|rng| rng.borrow_mut().gen_range(lower_value..upper_value))
                }
            }
            Operator::None => value,
        };

        if KEEP_PRECISION {
            let new_value_precision = precision_of_value(&new_value.to_string());

            if new_value_precision < value_precision {
                let new_value = format!("{:.*}", value_precision, new_value);
                Ok(new_value)
            } else {
                Ok(format!("{:.*}", value_precision, new_value))
            }
        } else {
            Ok(new_value.to_string())
        }
    }
}

impl From<&str> for Instruction {
    fn from(s: &str) -> Self {
        let words = s.split_whitespace();

        let mut keyword: Option<String> = None;
        let mut operator: Option<Operator> = None;
        let mut value_a: Option<f64> = None;
        let mut value_b: Option<f64> = None;

        for word in words.into_iter() {
            let possible_operator = Operator::from(word);

            let possible_value = word.parse::<f64>();

            if let Ok(value) = possible_value {
                if value_a.is_none() {
                    value_a = Some(value);
                } else {
                    value_b = Some(value);
                }
            } else if possible_operator != Operator::None {
                operator = Some(possible_operator);
            } else {
                let is_key_word_shortcut = crate::SHORT_KEYWORDS.contains(&word);

                if is_key_word_shortcut {
                    let index = match crate::SHORT_KEYWORDS.iter().position(|&r| r == word) {
                        Some(index) => index,
                        None => {
                            log::warn!("This should never happen. Please report this bug. Short keyword found but could not determine index. Results may be unexpected.");
                            0
                        }
                    };

                    keyword = Some(crate::KEYWORDS[index].to_string());
                } else {
                    log::warn!(
                        "{} is not a known keyword. Results may be unexpected.",
                        word
                    );

                    keyword = Some(word.to_string())
                }
            }
        }

        let keyword = match keyword {
            Some(keyword) => keyword,
            None => {
                log::warn!("No keyword found. Results may be unexpected.");
                "".to_string()
            }
        };

        let operator = match operator {
            Some(operator) => operator,
            None => {
                log::warn!("No operator found. Results may be unexpected.");
                Operator::None
            }
        };

        let value_a = match value_a {
            Some(value_a) => value_a,
            None => {
                log::warn!("No value found. Results may be unexpected.");
                0.0
            }
        };

        Instruction {
            keyword: keyword,
            operator: operator,
            value_a: value_a,
            value_b,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Instructions(HashMap<String, Vec<Instruction>>);

impl Instructions {
    pub fn apply(&self, keyword: &str, value: String) -> anyhow::Result<Option<String>> {
        let instructions = self.0.get(keyword);

        if let Some(instructions) = instructions {
            let mut new_value = value;

            for instruction in instructions {
                new_value = instruction.apply(new_value)?;
            }

            Ok(Some(new_value))
        } else {
            Ok(None)
        }
    }

    pub fn from_string(s: &str) -> Self {
        s.replace(";", "\n").replace(",", "\n").as_str().into()
    }

    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let s = std::fs::read_to_string(path)?
            .replace(";", "\n")
            .replace(",", "\n");

        Ok(s.as_str().into())
    }
}

impl From<&str> for Instructions {
    fn from(s: &str) -> Self {
        let mut map: HashMap<String, Vec<Instruction>> = HashMap::new();

        let lines = s.split('\n');

        for line in lines {
            let instruction = Instruction::from(line);

            let keyword = instruction.keyword.clone();

            if let Some(instructions) = map.get_mut(&keyword) {
                instructions.push(instruction);
            } else {
                map.insert(keyword, vec![instruction]);
            }
        }

        Instructions(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_from() {
        assert_eq!(Operator::from("+"), Operator::Add);
        assert_eq!(Operator::from("-"), Operator::Subtract);
        assert_eq!(Operator::from("*"), Operator::Multiply);
        assert_eq!(Operator::from("/"), Operator::Divide);
        assert_eq!(Operator::from("^"), Operator::Power);
        assert_eq!(Operator::from("--"), Operator::Range);
        assert_eq!(Operator::from("hello"), Operator::None);
    }

    #[test]
    fn test_instruction_from() {
        let instruction = Instruction::from("a + 1");
        assert_eq!(instruction.keyword, "_cell_length_a");
        assert_eq!(instruction.operator, Operator::Add);
        assert_eq!(instruction.value_a, 1.0);
        assert_eq!(instruction.value_b, None);

        let instruction = Instruction::from("a - 1");
        assert_eq!(instruction.keyword, "_cell_length_a");
        assert_eq!(instruction.operator, Operator::Subtract);
        assert_eq!(instruction.value_a, 1.0);
        assert_eq!(instruction.value_b, None);

        let instruction = Instruction::from("a * 1");
        assert_eq!(instruction.keyword, "_cell_length_a");
        assert_eq!(instruction.operator, Operator::Multiply);
        assert_eq!(instruction.value_a, 1.0);
        assert_eq!(instruction.value_b, None);

        let instruction = Instruction::from("a / 1");
        assert_eq!(instruction.keyword, "_cell_length_a");
        assert_eq!(instruction.operator, Operator::Divide);
        assert_eq!(instruction.value_a, 1.0);
        assert_eq!(instruction.value_b, None);

        let instruction = Instruction::from("a ^ 1");
        assert_eq!(instruction.keyword, "_cell_length_a");
        assert_eq!(instruction.operator, Operator::Power);
        assert_eq!(instruction.value_a, 1.0);
        assert_eq!(instruction.value_b, None);

        let instruction = Instruction::from("a -- 1");
        assert_eq!(instruction.keyword, "_cell_length_a");
        assert_eq!(instruction.operator, Operator::Range);
        assert_eq!(instruction.value_a, 1.0);
        assert_eq!(instruction.value_b, None);

        let instruction = Instruction::from("0 -- b -- 1");
        assert_eq!(instruction.keyword, "_cell_length_b");
        assert_eq!(instruction.operator, Operator::Range);
        assert_eq!(instruction.value_a, 0.0);
        assert_eq!(instruction.value_b, Some(1.0));
    }

    #[test]
    fn test_apply() {
        let instruction = Instruction::from("a + 1");
        let result = instruction.apply("1.0".to_string()).unwrap();
        assert_eq!(result, "2.0");

        let instruction = Instruction::from("a - 1");
        let result = instruction.apply("1.0".to_string()).unwrap();
        assert_eq!(result, "0.0");

        let instruction = Instruction::from("a * 2");
        let result = instruction.apply("1.0".to_string()).unwrap();
        assert_eq!(result, "2.0");

        let instruction = Instruction::from("a / 2");
        let result = instruction.apply("1.0".to_string()).unwrap();
        assert_eq!(result, "0.5");

        let instruction = Instruction::from("a ^ 2");
        let result = instruction.apply("1.0".to_string()).unwrap();
        assert_eq!(result, "1.0");

        let instruction = Instruction::from("a -- 2");
        let result = instruction.apply("1.0".to_string()).unwrap();
        assert!(result.parse::<f64>().unwrap() >= 1.0);
    }
}
