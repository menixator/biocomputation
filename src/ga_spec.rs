use crate::crossover::CrossoverStrategy;
use crate::mutation::MutationStrategy;
use crate::selection::SelectionStrategy;

#[derive(Debug)]
pub struct GaSpec {
    pub(crate) max_candidates: usize,
    pub(crate) min_candidates: usize,
    pub(crate) max_candidate_generation_consecutive_fail: usize,
    pub(crate) max_rules: usize,
    pub(crate) min_rules: usize,
    pub(crate) max_rule_generation_consecutive_fail: usize,

    pub(crate) max_index: usize,
    pub(crate) max_rule_constraints: usize,
    pub(crate) min_rule_constraints: usize,
    pub(crate) max_rule_constraint_generation_consecutive_fail: usize,
    pub(crate) selection_strategy: SelectionStrategy,
    pub(crate) crossover_strategy: CrossoverStrategy,
    pub(crate) mutation_strategy: MutationStrategy,
    pub(crate) alphabet: &'static str,
}
