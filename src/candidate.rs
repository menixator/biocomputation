use crate::dataset::DataSet;
use crate::ga_spec::GaSpec;
use crate::rule::{Rule, RuleEvaluationError};
use rand::{self, Rng};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use thiserror::Error;

/// A candidate is a collection of rules
#[derive(Debug, Clone, Eq)]
pub struct Candidate {
    rules: HashSet<Rule>,
    mutation_count: usize,
    birth_generation_id: Option<usize>,
}

impl PartialEq<Candidate> for Candidate {
    fn eq(&self, rhs: &Candidate) -> bool {
        self.rules() == rhs.rules()
    }
}

#[derive(Error, Debug, PartialEq, Clone, Copy)]
pub enum FitnessCalculationError {
    #[error(transparent)]
    RuleEvaluationError(#[from] RuleEvaluationError),
}

#[derive(Debug, PartialEq, Clone, Eq, Copy)]
pub struct CandidateFitness<'a> {
    pub candidate: &'a Candidate,
    pub fitness: usize,
}

impl Candidate {
    pub fn from_rules(rules: &HashSet<Rule>) -> Self {
        Self {
            rules: rules.clone(),
            mutation_count: 0,
            birth_generation_id: None,
        }
    }

    pub fn mutation_count(&self) -> usize {
        self.mutation_count
    }

    pub fn set_mutation_count(&mut self, new_mutation_count: usize) {
        self.mutation_count = new_mutation_count;
    }

    pub fn increment_mutation_count(&mut self) {
        self.mutation_count += 1;
    }

    pub fn age(&mut self, current_generation_id: usize) -> Option<isize> {
        self.birth_generation_id.clone().map(|birth_generation_id| {
            current_generation_id as isize - birth_generation_id as isize
        })
    }

    pub fn set_birth_generation_id(&mut self, new_birth_generation_id: usize) {
        self.birth_generation_id = Some(new_birth_generation_id);
    }

    pub fn birth_generation_id(&self) -> Option<usize> {
        self.birth_generation_id
    }

    pub fn rules(&self) -> &HashSet<Rule> {
        &self.rules
    }

    pub fn rules_mut(&mut self) -> &mut HashSet<Rule> {
        &mut self.rules
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
        let number_of_rules: usize = rng.gen_range(
            spec.initial_generation.rules.min,
            spec.initial_generation.rules.max,
        );
        let mut rules = HashSet::with_capacity(number_of_rules);

        let mut consecutive_fails = 0;

        while rules.len() < number_of_rules {
            if !rules.insert(Rule::generate(&mut rng, spec)) {
                consecutive_fails += 1;
                if consecutive_fails >= spec.initial_generation.rules.rng_fail_retries {
                    break;
                }
            } else {
                consecutive_fails = 0;
            }
        }
        Candidate {
            rules,
            mutation_count: 0,
            birth_generation_id: None,
        }
    }
}

impl Hash for Candidate {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut rules: Vec<&Rule> = self.rules.iter().collect();
        rules.sort_by_key(|rule| (rule.len(), rule.to_string()));
        rules.hash(state);
    }
}
