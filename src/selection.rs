use crate::candidate::Candidate;
use crate::candidate::CandidateFitness;
use rand::{self, Rng};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct SelectionStrategy {
    options: SelectionStrategyCommonOptions,
    variant: SelectionStrategyKind,
}

#[derive(Debug, Clone)]
pub struct DuplicateHandlingStrategy {
    allow: bool,
    retries: usize,
}

#[derive(Debug, Clone)]
pub struct SelectionStrategyCommonOptions {
    /// selection size
    size: usize,
    /// whether or not to allow duplicates
    duplicates: DuplicateHandlingStrategy,
}

#[derive(Debug, Clone)]
pub enum SelectionStrategyKind {
    Roulette(RouletteSelection),
    Tournament(TournamentSelection),
}

#[derive(Error, Debug)]
pub enum SelectionError {
    #[error("candidates are empty")]
    EmptyCandidates,

    #[error("rng failed to get a unique random value")]
    RngFail,
}

trait Selection {
    fn select<'a>(
        &'_ self,
        candidates: &Vec<CandidateFitness<'a>>,
        options: &SelectionStrategyCommonOptions,
    ) -> Result<Vec<CandidateFitness<'a>>, SelectionError>;
}

#[derive(Debug, Clone)]
pub struct TournamentSelection {
    /// The tournament size
    size: usize,
}

impl Selection for TournamentSelection {
    fn select<'a>(
        &'_ self,
        candidates: &Vec<CandidateFitness<'a>>,
        options: &SelectionStrategyCommonOptions,
    ) -> Result<Vec<CandidateFitness<'a>>, SelectionError> {
        // options.size is the selection size, not the tournament size
        let mut results: Vec<CandidateFitness> = Vec::with_capacity(options.size);

        let mut rng = rand::thread_rng();

        // TODO: self.size or options.size could be 0
        // TODO: candidates could be 0

        while results.len() < options.size {
            let mut best: Option<&CandidateFitness> = None;
            for i in 0..self.size {
                // Rng can fail you
                let index = {
                    let mut failures = 0;
                    loop {
                        let rng = rng.gen_range(0, candidates.len());
                        if options.duplicates.allow {
                            break Ok(rng);
                        }
                        if results.contains(&&candidates[rng]) {
                            failures += 1;
                            if failures >= options.duplicates.retries {
                                break Err(SelectionError::RngFail);
                            }
                        } else {
                            break Ok(rng);
                        }
                    }
                }?;

                match best {
                    Some(ref prev_best) => {
                        let new_best = &candidates[index];
                        if new_best.fitness > prev_best.fitness {
                            best = Some(new_best)
                        }
                    }
                    None => best = Some(&candidates[index]),
                }
            }

            // TODO: can best be none?
            results.push(best.ok_or(SelectionError::EmptyCandidates)?.clone());
        }
        Ok(results)
    }
}

#[derive(Debug, Clone)]
pub struct RouletteSelection;

impl Selection for RouletteSelection {
    fn select<'a>(
        &'_ self,
        candidates: &Vec<CandidateFitness<'a>>,
        options: &SelectionStrategyCommonOptions,
    ) -> Result<Vec<CandidateFitness<'a>>, SelectionError> {
        let mut results: Vec<CandidateFitness> = Vec::with_capacity(options.size);
        let mut rng = rand::thread_rng();

        // First sum up the fitness values
        let total: usize = candidates.iter().map(|candidate| candidate.fitness).sum();
        let mut cumulative_total = 0;
        let rng = rng.gen_range(0, total);
        let mut failures = 0;

        while results.len() < options.size {
            let mut selected = None;
            for candidate in candidates {
                cumulative_total += candidate.fitness;
                if rng < cumulative_total {
                    selected = Some(candidate);
                    break;
                }
            }
            let selected = selected.ok_or(SelectionError::EmptyCandidates)?;
            // Check if it is a duplicate
            if !options.duplicates.allow && results.contains(selected) {
                failures += 1;
                if failures >= options.duplicates.retries {
                    return Err(SelectionError::RngFail);
                }
            } else {
                failures = 0;
                results.push(selected.clone());
            }
        }
        Ok(results)
    }
}
