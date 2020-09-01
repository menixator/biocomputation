use crate::candidate::Candidate;
use crate::candidate::CandidateFitness;
use crate::rule::Rule;
use rand::{self, Rng};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug)]
pub struct CrossoverStrategy {
    matchup_strategy: MatchupStrategy,
    options: CrossoverStrategyCommonOptions,
    mating_strategy: MatingStrategy,
}

impl CrossoverStrategy {
    pub fn new(
        matchup_strategy: MatchupStrategy,
        mating_strategy: MatingStrategy,
        options: CrossoverStrategyCommonOptions,
    ) -> Self {
        Self {
            matchup_strategy,
            mating_strategy,
            options,
        }
    }
}

#[derive(Debug)]
pub struct CrossoverStrategyCommonOptions {
    mirroring: MirroringStrategy,
}

impl CrossoverStrategyCommonOptions {
    pub fn new(mirroring: MirroringStrategy) -> Self {
        Self { mirroring }
    }
}

#[derive(Debug)]
pub enum MatchupStrategy {
    Random {
        allow_asexual: bool,
        allow_duplicates: bool,
    },
    NextFittest,
    LeastFittest,
}

#[derive(Debug)]
pub enum MirroringStrategy {
    MirrorIfAsexual,
    AlwaysMirror,
    Never,
}

#[derive(Debug)]
pub enum MatingStrategy {
    SinglePointAtIndex { split_at: u8 },
    SinglePointAtPercentage { split_at: u8 },
    MultiPointAtIndices { split_at: Vec<(u8, u8)> },
    MultiPointAtPercentages { split_at: Vec<(u8, u8)> },
}

#[derive(Error, Debug)]
pub enum CrossoverError {
    #[error("rng failed to generate a unique value")]
    RngFail,

    #[error("cant generate a non asexual matchup with a single candidate")]
    CantGenerateNonAsexualMatchupWithOneCandidate,
}

impl CrossoverStrategy {
    /// Returns an iterator of matchups
    /// Assumes candidates is sorted
    pub fn matchup<'a, 'b: 'a>(
        &'b self,
        candidates: &'b Vec<CandidateFitness<'a>>,
    ) -> Result<
        Box<dyn Iterator<Item = (CandidateFitness<'a>, CandidateFitness<'a>)> + 'b>,
        CrossoverError,
    > {
        match self.matchup_strategy {
            MatchupStrategy::LeastFittest => Ok(Box::new(
                candidates
                    .iter()
                    .take(candidates.len() - 1)
                    .zip(candidates.iter().skip(1).rev())
                    .map(|(a, b)| (a.clone(), b.clone())),
            )),

            MatchupStrategy::NextFittest => Ok(Box::new(
                candidates
                    .iter()
                    .take(candidates.len() - 1)
                    .zip(candidates.iter().skip(1))
                    .map(|(a, b)| (a.clone(), b.clone())),
            )),

            MatchupStrategy::Random {
                allow_asexual,
                allow_duplicates,
            } => {
                if candidates.len() == 1 && !allow_asexual {
                    return Err(CrossoverError::CantGenerateNonAsexualMatchupWithOneCandidate);
                } else {
                    let mut matchups = Vec::with_capacity(candidates.len() - 1);
                    let mut rng = rand::thread_rng();
                    'main: for candidate_index in 0..candidates.len() {
                        let mut matchup = rng.gen_range(0, candidates.len());
                        if !allow_asexual && matchup == candidate_index {
                            // Roll over the matchup if we don't allow asexual reproduction
                            matchup = (matchup + 1) % candidates.len();
                        }

                        if !allow_duplicates {
                            let mut checks = 0;
                            while matchups.contains(&(candidate_index, matchup)) {
                                matchup += 1;
                                matchup %= candidates.len();
                                if checks >= candidates.len() {
                                    // Assume a non-duplicate cannot be found
                                    continue 'main;
                                }
                                checks += 1;
                            }
                        }
                        matchups.push((candidate_index, matchup))
                    }
                    Ok(Box::new(matchups.into_iter().map(move |(a, b)| {
                        (candidates[a].clone(), candidates[b].clone())
                    })))
                }
            }
        }
    }

    pub fn crossover(
        &'_ self,
        candidates: &Vec<CandidateFitness<'_>>,
    ) -> Result<Vec<Candidate>, CrossoverError> {
        let mut results = Vec::new();
        let matchup = self.matchup(candidates)?;
        match &self.mating_strategy {
            MatingStrategy::SinglePointAtIndex { split_at } => {
                for (a, b) in matchup {
                    let split_at_a = *split_at as usize;
                    let split_at_b = match self.options.mirroring {
                        MirroringStrategy::MirrorIfAsexual => {
                            // This is kind of unpredictable
                            // If the number of rules within a candidate are less than than
                            if a == b {
                                std::cmp::max(b.candidate.rules().len(), split_at_a)
                                    - std::cmp::min(b.candidate.rules().len(), split_at_a)
                            } else {
                                split_at_a
                            }
                        }
                        MirroringStrategy::AlwaysMirror => {
                            std::cmp::max(b.candidate.rules().len(), split_at_a)
                                - std::cmp::min(b.candidate.rules().len(), split_at_a)
                        }
                        MirroringStrategy::Never => split_at_a,
                    };

                    let first_child = a
                        .candidate
                        .rules()
                        .iter()
                        .take(split_at_a)
                        .chain(b.candidate.rules().iter().skip(split_at_b))
                        .map(|rule| rule.clone())
                        .collect();

                    results.push(Candidate::from_rules(&first_child));
                    let second_child = b
                        .candidate
                        .rules()
                        .iter()
                        .take(split_at_b)
                        .chain(a.candidate.rules().iter().skip(split_at_a))
                        .map(|rule| rule.clone())
                        .collect();
                    results.push(Candidate::from_rules(&second_child));
                }
                Ok(results)
            }
            MatingStrategy::SinglePointAtPercentage { split_at } => {
                for (a, b) in matchup {
                    // TODO: values over 100 will fail silently
                    let split_at_a =
                        ((*split_at as f64 / 100.0) * a.candidate.rules().len() as f64) as usize;
                    let split_at_b_no_mirror =
                        ((*split_at as f64 / 100.0) * b.candidate.rules().len() as f64) as usize;

                    let split_at_b = match self.options.mirroring {
                        MirroringStrategy::MirrorIfAsexual => {
                            if a == b {
                                b.candidate.rules().len() - split_at_b_no_mirror
                            } else {
                                split_at_b_no_mirror
                            }
                        }
                        MirroringStrategy::AlwaysMirror => {
                            b.candidate.rules().len() - split_at_b_no_mirror
                        }
                        MirroringStrategy::Never => split_at_b_no_mirror,
                    };

                    let first_child = a
                        .candidate
                        .rules()
                        .iter()
                        .take(split_at_a)
                        .chain(b.candidate.rules().iter().skip(split_at_b))
                        .map(|rule| rule.clone())
                        .collect();

                    results.push(Candidate::from_rules(&first_child));
                    let second_child = b
                        .candidate
                        .rules()
                        .iter()
                        .take(split_at_b)
                        .chain(a.candidate.rules().iter().skip(split_at_a))
                        .map(|rule| rule.clone())
                        .collect();
                    results.push(Candidate::from_rules(&second_child));
                }
                Ok(results)
            }
            MatingStrategy::MultiPointAtIndices { split_at } => {
                for (a, b) in matchup {
                    let mut first_child: HashSet<Rule> = HashSet::new();
                    let mut second_child: HashSet<Rule> = HashSet::new();

                    for (split_at_a_start, split_at_a_end) in split_at.iter() {
                        let split_at_a_start = *split_at_a_start as usize;
                        let split_at_a_end = *split_at_a_end as usize;

                        let split_at_a_start = std::cmp::min(split_at_a_start, split_at_a_end);
                        let split_at_a_end = std::cmp::max(split_at_a_start, split_at_a_end);

                        let (split_at_b_start, split_at_b_end) = match self.options.mirroring {
                            MirroringStrategy::MirrorIfAsexual => {
                                // This is kind of unpredictable
                                // If the number of rules within a candidate are less than than
                                if a == b {
                                    let split_at_b_start =
                                        std::cmp::max(b.candidate.rules().len(), split_at_a_start)
                                            - std::cmp::min(
                                                b.candidate.rules().len(),
                                                split_at_a_start,
                                            );

                                    let split_at_b_end =
                                        std::cmp::max(b.candidate.rules().len(), split_at_a_end)
                                            - std::cmp::min(
                                                b.candidate.rules().len(),
                                                split_at_a_end,
                                            );
                                    (split_at_b_start, split_at_b_end)
                                } else {
                                    (split_at_a_start, split_at_a_end)
                                }
                            }
                            MirroringStrategy::AlwaysMirror => {
                                let split_at_b_start =
                                    std::cmp::max(b.candidate.rules().len(), split_at_a_start)
                                        - std::cmp::min(
                                            b.candidate.rules().len(),
                                            split_at_a_start,
                                        );

                                let split_at_b_end =
                                    std::cmp::max(b.candidate.rules().len(), split_at_a_end)
                                        - std::cmp::min(b.candidate.rules().len(), split_at_a_end);
                                (split_at_b_start, split_at_b_end)
                            }
                            MirroringStrategy::Never => (split_at_a_start, split_at_a_end),
                        };

                        first_child.extend(
                            a.candidate
                                .rules()
                                .iter()
                                .skip(split_at_a_start)
                                .take(split_at_a_end - split_at_a_start)
                                .chain(
                                    b.candidate
                                        .rules()
                                        .iter()
                                        .skip(split_at_b_start)
                                        .take(split_at_b_end - split_at_b_start),
                                )
                                .map(|rule| rule.clone()),
                        );

                        second_child.extend(
                            b.candidate
                                .rules()
                                .iter()
                                .skip(split_at_b_start)
                                .take(split_at_b_end - split_at_b_start)
                                .chain(
                                    a.candidate
                                        .rules()
                                        .iter()
                                        .skip(split_at_a_start)
                                        .take(split_at_a_end - split_at_a_start),
                                )
                                .map(|rule| rule.clone()),
                        );
                    }
                    results.push(Candidate::from_rules(&first_child));
                    results.push(Candidate::from_rules(&second_child));
                }
                Ok(results)
            }
            MatingStrategy::MultiPointAtPercentages { split_at } => {
                for (a, b) in matchup {
                    let mut first_child: HashSet<Rule> = HashSet::new();
                    let mut second_child: HashSet<Rule> = HashSet::new();

                    for (percent_split_at_a_start, percent_split_at_a_end) in split_at.iter() {
                        let split_at_a_start = (((*percent_split_at_a_start as f64 / 100.0) as f64)
                            * a.candidate.rules().len() as f64)
                            as usize;
                        let split_at_a_end = (((*percent_split_at_a_end as f64 / 100.0) as f64)
                            * a.candidate.rules().len() as f64)
                            as usize;

                        let split_at_a_start = std::cmp::min(split_at_a_start, split_at_a_end);
                        let split_at_a_end = std::cmp::max(split_at_a_start, split_at_a_end);

                        let split_at_b_start_no_mirror =
                            (((*percent_split_at_a_start as f64 / 100.0) as f64)
                                * b.candidate.rules().len() as f64)
                                as usize;
                        let split_at_b_end_no_mirror = (((*percent_split_at_a_end as f64 / 100.0)
                            as f64)
                            * b.candidate.rules().len() as f64)
                            as usize;

                        let split_at_b_start_no_mirror =
                            std::cmp::min(split_at_b_start_no_mirror, split_at_b_end_no_mirror);
                        let split_at_b_end_no_mirror =
                            std::cmp::max(split_at_b_start_no_mirror, split_at_b_end_no_mirror);

                        let (split_at_b_start, split_at_b_end) = match self.options.mirroring {
                            MirroringStrategy::MirrorIfAsexual => {
                                // This is kind of unpredictable
                                // If the number of rules within a candidate are less than than
                                if a == b {
                                    let split_at_b_start =
                                        b.candidate.rules().len() - split_at_b_start_no_mirror;

                                    let split_at_b_end =
                                        b.candidate.rules().len() - split_at_b_end_no_mirror;
                                    (split_at_b_start, split_at_b_end)
                                } else {
                                    (split_at_b_start_no_mirror, split_at_b_end_no_mirror)
                                }
                            }
                            MirroringStrategy::AlwaysMirror => {
                                let split_at_b_start =
                                    b.candidate.rules().len() - split_at_b_start_no_mirror;

                                let split_at_b_end =
                                    b.candidate.rules().len() - split_at_b_end_no_mirror;
                                (split_at_b_start, split_at_b_end)
                            }
                            MirroringStrategy::Never => {
                                (split_at_b_start_no_mirror, split_at_b_end_no_mirror)
                            }
                        };

                        first_child.extend(
                            a.candidate
                                .rules()
                                .iter()
                                .skip(split_at_a_start)
                                .take(split_at_a_end - split_at_a_start)
                                .chain(
                                    b.candidate
                                        .rules()
                                        .iter()
                                        .skip(split_at_b_start)
                                        .take(split_at_b_end - split_at_b_start),
                                )
                                .map(|rule| rule.clone()),
                        );

                        second_child.extend(
                            b.candidate
                                .rules()
                                .iter()
                                .skip(split_at_b_start)
                                .take(split_at_b_end - split_at_b_start)
                                .chain(
                                    a.candidate
                                        .rules()
                                        .iter()
                                        .skip(split_at_a_start)
                                        .take(split_at_a_end - split_at_a_start),
                                )
                                .map(|rule| rule.clone()),
                        );
                    }
                    results.push(Candidate::from_rules(&first_child));
                    results.push(Candidate::from_rules(&second_child));
                }
                Ok(results)
            }
        }
    }
}
