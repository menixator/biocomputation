use crate::crossover::CrossoverStrategy;
use crate::mutation::MutationStrategy;
use crate::selection::SelectionStrategy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug)]
pub struct GaSpec {
    pub initial_generation: InitialGenerationSpec,
    pub max_evolutions: usize,
    pub stop_at_optimum_fitness: bool,
    pub selection: SelectionStrategy,
    pub crossover: CrossoverStrategy,
    pub mutation: MutationStrategy,
    pub calculated: CalculatedSpecs,
}

impl From<(GaSpecInput, CalculatedSpecs)> for GaSpec {
    fn from((ga_spec_input, calculated): (GaSpecInput, CalculatedSpecs)) -> Self {
        let GaSpecInput {
            initial_generation,
            max_evolutions,
            stop_at_optimum_fitness,
            selection,
            crossover,
            mutation,
        } = ga_spec_input;

        GaSpec {
            initial_generation,
            max_evolutions,
            stop_at_optimum_fitness,
            selection,
            crossover,
            mutation,
            calculated,
        }
    }
}

#[derive(Error, Debug)]
pub enum GaSpecInputParseError {
    #[error("an io error occured")]
    IoError(std::io::ErrorKind),

    #[error("json parse error: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl GaSpecInput {
    pub fn from_file<T: AsRef<Path>>(path: T) -> Result<GaSpecInput, GaSpecInputParseError> {
        let config = fs::read_to_string(path.as_ref())
            .map_err(|err| GaSpecInputParseError::IoError(err.kind()))?;
        let ga_spec = serde_json::from_str(&config)?;
        Ok(ga_spec)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CalculatedSpecs {
    pub alphabet: &'static str,
    pub max_index: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GaSpecInput {
    initial_generation: InitialGenerationSpec,
    max_evolutions: usize,
    stop_at_optimum_fitness: bool,
    selection: SelectionStrategy,
    crossover: CrossoverStrategy,
    mutation: MutationStrategy,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InitialGenerationSpec {
    pub candidates: InitialGenerationComponentSpec,
    pub rules: InitialGenerationComponentSpec,
    pub constraints: InitialGenerationComponentSpec,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(try_from = "InitialGenerationComponentSpecShadow")]
pub struct InitialGenerationComponentSpec {
    pub min: usize,
    pub max: usize,
    pub rng_fail_retries: usize,
}

#[derive(Deserialize, Debug, Clone, Copy)]
struct InitialGenerationComponentSpecShadow {
    min: usize,
    max: usize,
    rng_fail_retries: usize,
}

#[derive(Error, Debug)]
pub enum InitialGenerationComponentSpecParseError {
    #[error("max is less than min")]
    MaxIsLessThanMin,
    #[error("max and min are zero")]
    MaxAndMinAreZero,
}

impl std::convert::TryFrom<InitialGenerationComponentSpecShadow>
    for InitialGenerationComponentSpec
{
    type Error = InitialGenerationComponentSpecParseError;
    fn try_from(shadow: InitialGenerationComponentSpecShadow) -> Result<Self, Self::Error> {
        let InitialGenerationComponentSpecShadow {
            min,
            max,
            rng_fail_retries,
        } = shadow;

        if min == 0 && max == 0 {
            return Err(InitialGenerationComponentSpecParseError::MaxAndMinAreZero);
        }
        if min < max {
            return Err(InitialGenerationComponentSpecParseError::MaxIsLessThanMin);
        }
        Ok(InitialGenerationComponentSpec {
            min,
            max,
            rng_fail_retries,
        })
    }
}
