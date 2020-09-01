use crate::popgenspec::PopGenSpec;
use crate::rule::Rule;
use rand::{self, Rng};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

/// A candidate is a collection of rules
#[derive(PartialEq, Debug, Clone, Eq)]
pub struct Candidate {
    rules: HashSet<Rule>,
}

impl Candidate {
    pub fn rules(&self) -> &HashSet<Rule> {
        &self.rules
    }

    pub fn generate<T: Rng>(mut rng: &mut T, spec: &PopGenSpec) -> Self {
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
