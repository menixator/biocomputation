use crate::dataset::DataSet;
use crate::ga_spec::GaSpec;
use crate::rule::{Rule, RuleEvaluationError};
use rand::{self, Rng};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use thiserror::Error;

/// A candidate is a collection of rules
#[derive(PartialEq, Debug, Clone, Eq)]
pub struct Candidate {
    rules: HashSet<Rule>,
}

#[derive(Error, Debug)]
pub enum FitnessCalculationError {
    #[error(transparent)]
    RuleEvaluationError(#[from] RuleEvaluationError),
}

#[derive(Debug, PartialEq, Clone)]
pub struct CandidateFitness<'a> {
    pub candidate: &'a Candidate,
    pub fitness: usize,
}

impl Candidate {
    pub fn rules(&self) -> &HashSet<Rule> {
        &self.rules
    }

    /// Fitness is simply the number of test data a candidate's ruleset can classify correctly
    pub fn calculate_fitness(&self, data_set: &DataSet) -> Result<usize, FitnessCalculationError> {
        let mut fitness = 0;

        for data_item in data_set.as_ref() {
            for rule in &self.rules {
                let result = if rule.evaluate(data_item.as_str())? {
                    "1"
                } else {
                    "0"
                };

                if result == data_item.output() {
                    fitness += 1;
                    break;
                }
            }
        }
        Ok(fitness)
    }

    pub fn generate<T: Rng>(mut rng: &mut T, spec: &GaSpec) -> Self {
        let number_of_rules: usize = rng.gen_range(spec.min_rules, spec.max_rules);
        let mut rules = HashSet::with_capacity(number_of_rules);

        let mut consecutive_fails = 0;

        while rules.len() < number_of_rules {
            if !rules.insert(Rule::generate(&mut rng, spec)) {
                consecutive_fails += 1;
                if consecutive_fails >= spec.max_rule_generation_consecutive_fail {
                    break;
                }
            } else {
                consecutive_fails = 0;
            }
        }
        Candidate { rules }
    }
}

impl Hash for Candidate {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut rules: Vec<&Rule> = self.rules.iter().collect();
        rules.sort_by_key(|rule| (rule.len(), rule.to_string()));
        rules.hash(state);
    }
}
