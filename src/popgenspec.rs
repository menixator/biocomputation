#[derive(Debug)]
pub struct PopGenSpec {
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

    pub(crate) alphabet: &'static str,
}
