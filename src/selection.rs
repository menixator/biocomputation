use crate::candidate::Candidate;
use crate::candidate::CandidateFitness;
use rand::{self, Rng};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize)]
pub struct SelectionStrategy {
    #[serde(flatten)]
    pub options: SelectionStrategyCommonOptions,
    #[serde(flatten)]
    pub variant: SelectionStrategyVariant,
}

impl SelectionStrategy {
    pub fn select<'a>(
        &'_ self,
        candidates: &Vec<CandidateFitness<'a>>,
    ) -> Result<Vec<CandidateFitness<'a>>, SelectionError> {
        match &self.variant {
            SelectionStrategyVariant::Tournament(tourney) => {
                tourney.select(candidates, &self.options)
            }
            SelectionStrategyVariant::Roulette(roulette) => {
                roulette.select(candidates, &self.options)
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "setting")]
pub enum DuplicateHandlingStrategy {
    Allow,
    Disallow { retries: usize },
}

#[derive(Debug, Clone, Deserialize)]
pub struct SelectionStrategyCommonOptions {
    /// selection size
    pub selection_size: usize,
    /// whether or not to allow duplicates
    pub duplicates: DuplicateHandlingStrategy,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum SelectionStrategyVariant {
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

pub trait Selection {
    fn select<'a>(
        &'_ self,
        candidates: &Vec<CandidateFitness<'a>>,
        options: &SelectionStrategyCommonOptions,
    ) -> Result<Vec<CandidateFitness<'a>>, SelectionError>;
}

#[derive(Debug, Clone, Deserialize)]
pub struct TournamentSelection {
    /// The tournament size
    pub tournament_size: usize,
}

impl Selection for TournamentSelection {
    fn select<'a>(
        &'_ self,
        candidates: &Vec<CandidateFitness<'a>>,
        options: &SelectionStrategyCommonOptions,
    ) -> Result<Vec<CandidateFitness<'a>>, SelectionError> {
        // options.selection_size is the selection size, not the tournament size
        let mut results: Vec<CandidateFitness> = Vec::with_capacity(options.selection_size);

        let mut rng = rand::thread_rng();

        // TODO: self.size or options.selection_size could be 0
        // TODO: candidates could be 0

        while results.len() < options.selection_size {
            let mut best: Option<&CandidateFitness> = None;
            for i in 0..self.tournament_size {
                // Rng can fail you
                let index = {
                    let mut failures = 0;
                    loop {
                        let rng = rng.gen_range(0, candidates.len());
                        match options.duplicates {
                            DuplicateHandlingStrategy::Allow => break Ok(rng),
                            DuplicateHandlingStrategy::Disallow { retries } => {
                                if results.contains(&&candidates[rng]) {
                                    failures += 1;
                                    if failures >= retries {
                                        break Err(SelectionError::RngFail);
                                    }
                                } else {
                                    break Ok(rng);
                                }
                            }
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

#[derive(Debug, Clone, Deserialize)]
pub struct RouletteSelection;

impl Selection for RouletteSelection {
    fn select<'a>(
        &'_ self,
        candidates: &Vec<CandidateFitness<'a>>,
        options: &SelectionStrategyCommonOptions,
    ) -> Result<Vec<CandidateFitness<'a>>, SelectionError> {
        let mut results: Vec<CandidateFitness> = Vec::with_capacity(options.selection_size);
        let mut rng = rand::thread_rng();

        // First sum up the fitness values
        let total: usize = candidates.iter().map(|candidate| candidate.fitness).sum();
        let mut cumulative_total = 0;
        let rng = rng.gen_range(0, total);
        let mut failures = 0;

        while results.len() < options.selection_size {
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
            match (&options.duplicates, results.contains(selected)) {
                (DuplicateHandlingStrategy::Disallow { retries }, true) => {
                    failures += 1;
                    if failures >= *retries {
                        return Err(SelectionError::RngFail);
                    }
                }
                _ => {
                    failures = 0;
                    results.push(selected.clone());
                }
            }
        }
        Ok(results)
    }
}
