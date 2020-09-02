use crate::candidate::Candidate;
use crate::ga_spec::GaSpec;
use crate::population::Population;
use crate::rule::Rule;
use rand::{self, Rng};
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct MutationStrategy {
    pub options: MutationStrategyCommonOptions,
    pub variant: MutationStrategyVariant,
}

#[derive(Clone, Debug)]
pub enum MutationStrategyVariant {
    ConstraintSwap { delta: isize },
    ConstraintRandomize { swap_if_fail: bool, retries: usize },
    ConstraintValueRandomize,
}

#[derive(Clone, Debug)]
pub struct MutationStrategyCommonOptions {
    pub chance: Option<usize>,
    pub chance_per_candidate: Option<usize>,
    pub chance_per_rule: Option<usize>,
    pub chance_per_constraint: Option<usize>,
}

#[derive(Error, Debug)]
pub enum MutationError {
    #[error("rng failed to generate a unique value")]
    RngFail,
}

impl MutationStrategy {
    pub fn mutate(
        &self,
        mut population: &mut Population,
        ga_spec: &GaSpec,
    ) -> Result<(), MutationError> {
        let mut rng = rand::thread_rng();
        // Rng makes  it very easy to generate a boolean based on a probablity
        let chance = self.options.chance.unwrap_or_default();

        // Early return if chance is 0
        if chance == 0 {
            return Ok(());
        }

        // If the rng gods tell us not to mutate, we wont mutate
        // TODO: Add a proper percentage type
        if !rng.gen_ratio(chance as u32, 100) {
            return Ok(());
        }

        let candidates = population.candidates();
        let mut changes: Vec<(Candidate, Candidate)> = Vec::new();

        for orig_candidate in candidates {
            if !rng.gen_ratio(
                self.options.chance_per_candidate.unwrap_or_default() as u32,
                100,
            ) {
                continue;
            }

            let candidate = orig_candidate.clone();

            let mut rules: Vec<Rule> = candidate.rules().iter().map(|rule| rule.clone()).collect();
            let mut ran = false;

            for rule in rules.iter_mut() {
                if !rng.gen_ratio(self.options.chance_per_rule.unwrap_or_default() as u32, 100) {
                    continue;
                }

                for constraint_key in 0..ga_spec.max_index {
                    if !rng.gen_ratio(
                        self.options.chance_per_constraint.unwrap_or_default() as u32,
                        100,
                    ) {
                        continue;
                    }
                    ran = true;

                    // There is a chance that we might end up creating a candidate that is similar
                    match &self.variant {
                        MutationStrategyVariant::ConstraintSwap { delta } => {
                            let new_key = constraint_key + (*delta as usize) % ga_spec.max_index;
                            let constraints = rule.constraints_mut();
                            let value_at_new_key = constraints.remove(&new_key);
                            let value_at_current_key = constraints.remove(&constraint_key);

                            if let Some(value_at_current_key) = value_at_current_key {
                                constraints.insert(new_key, value_at_current_key);
                            }

                            if let Some(value_at_new_key) = value_at_new_key {
                                constraints.insert(constraint_key, value_at_new_key);
                            }
                        }
                        MutationStrategyVariant::ConstraintRandomize {
                            swap_if_fail,
                            retries,
                        } => {
                            let mut swap = false;
                            let mut fails = 0;
                            let new_key = loop {
                                let rng_index = rng.gen_range(0, ga_spec.max_index);
                                if rule.constraints().contains_key(&rng_index) {
                                    if *swap_if_fail {
                                        swap = true;
                                    }
                                    if fails >= *retries {
                                        break Err(MutationError::RngFail);
                                    }
                                    fails += 1;
                                    continue;
                                }
                                break Ok(rng_index);
                            }?;

                            if swap {
                                let constraints = rule.constraints_mut();
                                let value_at_new_key = constraints.remove(&new_key);
                                let value_at_current_key = constraints.remove(&constraint_key);

                                if let Some(value_at_current_key) = value_at_current_key {
                                    constraints.insert(new_key, value_at_current_key);
                                }

                                if let Some(value_at_new_key) = value_at_new_key {
                                    constraints.insert(constraint_key, value_at_new_key);
                                }
                            } else {
                                // The new key does not exist
                                // The current key might exist
                                let constraints = rule.constraints_mut();
                                let value = constraints.remove(&constraint_key);
                                if let Some(value) = value {
                                    constraints.insert(new_key, value);
                                }
                            }
                        }
                        MutationStrategyVariant::ConstraintValueRandomize => {
                            let change = if let Some(character) =
                                rule.constraints().get(&constraint_key)
                            {
                                // Lets calculate the chance of the existing constraint being
                                // removed
                                // The chance is 1 in alphabet+1
                                if rng.gen_ratio(1, ga_spec.max_index as u32 + 1) {
                                    None
                                } else {
                                    let pos = rng.gen_range(0, ga_spec.max_index as u32) as usize;
                                    let new_char = ga_spec.alphabet.chars().nth(pos).unwrap();
                                    Some(if new_char == *character {
                                        ga_spec.alphabet.chars().nth(pos).unwrap()
                                    } else {
                                        new_char
                                    })
                                }
                            } else {
                                let pos = rng.gen_range(0, ga_spec.max_index as u32) as usize;
                                let new_char = ga_spec.alphabet.chars().nth(pos).unwrap();
                                Some(new_char)
                            };

                            if let Some(character) = change {
                                rule.constraints_mut().insert(constraint_key, character);
                            } else {
                                rule.constraints_mut().remove(&constraint_key);
                            }
                        }
                    }
                }
            }

            if ran {
                let new_candidate = Candidate::from_rules(&rules.into_iter().collect());
                if new_candidate != candidate
                    && !population.contains(&new_candidate)
                    && changes
                        .iter()
                        .position(|(_, existing_new_candidate)| {
                            *existing_new_candidate == new_candidate
                        })
                        .is_none()
                {
                    changes.push((candidate, new_candidate));
                }
            }
        }

        for (remove_me, add_me) in changes.into_iter() {
            population.remove(&remove_me);
            population.insert(add_me);
        }

        Ok(())
    }
}
