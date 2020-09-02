use crate::candidate::CandidateFitness;
use crate::candidate::{Candidate, FitnessCalculationError};
use crate::dataitem::DataItem;
use crate::dataset::DataSet;
use crate::ga_spec::GaSpec;
use rand::{self, Rng};
use std::collections::HashSet;

/// A population is a collection of candidates
#[derive(Debug)]
pub struct Population {
    generation: usize,
    candidates: Vec<Candidate>,
}

impl Population {
    pub fn candidates(&self) -> &Vec<Candidate> {
        &self.candidates
    }

    pub fn candidates_mut(&mut self) -> &mut Vec<Candidate> {
        &mut self.candidates
    }

    pub fn len(&self) -> usize {
        self.candidates.len()
    }

    fn push(&mut self, candidate: Candidate) -> bool {
        if self.candidates.contains(&candidate) {
            return false;
        }
        self.candidates.push(candidate);
        true
    }

    pub fn calculate_fitness<'a>(
        &self,
        data_set: &'_ DataSet,
    ) -> Result<Vec<CandidateFitness>, FitnessCalculationError> {
        let mut fitness_values = Vec::with_capacity(self.candidates.len());
        for candidate in &self.candidates {
            fitness_values.push(CandidateFitness {
                candidate,
                fitness: candidate.calculate_fitness(&data_set)?,
            });
        }
        fitness_values.sort_by_key(|candidate_with_fitness| candidate_with_fitness.fitness);
        Ok(fitness_values)
    }

    // Generates a a random population for a given data set
    pub fn generate(spec: &GaSpec) -> Self {
        let mut candidates = HashSet::with_capacity(spec.max_candidates);
        let mut consecutive_fails = 0;
        let mut rng = rand::thread_rng();
        let number_of_candidates = rng.gen_range(spec.min_candidates, spec.max_candidates);
        while candidates.len() < number_of_candidates {
            if !candidates.insert(Candidate::generate(&mut rng, spec)) {
                consecutive_fails += 1;
                if consecutive_fails >= spec.max_candidate_generation_consecutive_fail {
                    break;
                }
            } else {
                consecutive_fails = 0;
            }
        }

        let candidates = candidates.into_iter().collect();

        Population {
            generation: 1,
            candidates,
        }
    }
}

impl std::convert::AsRef<Vec<Candidate>> for Population {
    fn as_ref(&self) -> &Vec<Candidate> {
        &self.candidates
    }
}
