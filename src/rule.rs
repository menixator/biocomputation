use crate::ga_spec::GaSpec;
use rand::{self, Rng};
use std::collections::HashMap;
use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};
use std::string::ToString;
use thiserror::Error;

/// A rule is a list of checks to do to yield 1
#[derive(Eq, PartialEq, Clone)]
pub struct Rule {
    constraints: HashMap<usize, char>,
}

#[derive(Error, Debug)]
pub enum RuleEvaluationError {
    #[error("the rule contains constraints that is out of the index range of the input")]
    IndexOutOfRange,
}

impl Rule {
    pub fn constraints(&self) -> &HashMap<usize, char> {
        &self.constraints
    }

    pub fn constraints_mut(&mut self) -> &mut HashMap<usize, char> {
        &mut self.constraints
    }

    pub fn evaluate(&self, input: &str) -> Result<bool, RuleEvaluationError> {
        for (index, character) in &self.constraints {
            if input
                .chars()
                .nth(*index)
                .ok_or(RuleEvaluationError::IndexOutOfRange)?
                != *character
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn len(&self) -> usize {
        self.constraints.len()
    }

    pub fn generate<T: Rng>(mut rng: &mut T, spec: &GaSpec) -> Self {
        let number_of_constraints: usize =
            rng.gen_range(spec.min_rule_constraints, spec.max_rule_constraints);
        let mut constraints = HashMap::with_capacity(number_of_constraints);

        let mut consecutive_fails = 0;

        while constraints.len() < number_of_constraints {
            let index: usize = rng.gen_range(0, spec.max_index);

            if constraints.contains_key(&index) {
                consecutive_fails += 1;
                if consecutive_fails >= spec.max_rule_generation_consecutive_fail {
                    break;
                }
            } else {
                consecutive_fails = 0;
                let character_index = rng.gen_range(0, spec.alphabet.len());
                constraints.insert(index, spec.alphabet.chars().nth(character_index).unwrap());
            }
        }
        Rule { constraints }
    }
}

impl Debug for Rule {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_string())
    }
}

impl ToString for Rule {
    fn to_string(&self) -> String {
        let mut output = String::with_capacity(self.constraints.len());
        let mut entries: Vec<(&usize, &char)> = self.constraints.iter().collect();

        entries.sort_by_key(|(&index, _)| index);

        for (index, character) in entries.into_iter() {
            while *index > output.len() {
                output.push('_');
            }
            output.push(*character);
        }
        output
    }
}

impl Hash for Rule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let as_str = format!("{:?}", self);
        as_str.hash(state);
    }
}
