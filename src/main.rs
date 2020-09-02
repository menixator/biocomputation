use serde::{Deserialize, Serialize};
use serde_json;
use std::path::PathBuf;
use structopt::StructOpt;
use thiserror::Error;
#[deny(missing_copy_implementations, missing_debug_implementations)]
mod candidate;
mod crossover;
mod dataitem;
mod dataset;
mod ga_spec;
mod mutation;
mod population;
mod rule;
mod selection;

use candidate::CandidateFitness;
use crossover::{
    CrossoverStrategy, CrossoverStrategyCommonOptions, MatchupStrategy, MatingStrategy,
    MirroringStrategy,
};
use mutation::{MutationStrategy, MutationStrategyCommonOptions, MutationStrategyVariant};

use ga_spec::{CalculatedSpecs, GaSpec, GaSpecInput};
use population::Population;
use selection::{
    DuplicateHandlingStrategy, RouletteSelection, Selection, SelectionStrategy,
    SelectionStrategyCommonOptions, SelectionStrategyVariant, TournamentSelection,
};

#[derive(Error, Debug)]
pub enum PercentageParseError {
    #[error("cannot parse a percentage from the provided split value")]
    FloatParseError(#[from] std::num::ParseFloatError),

    #[error("percentages cannot be greater than 100")]
    InvalidPercentage,

    #[error("cannot split at zero")]
    CannotSplitAtZero,
}

fn parse_percentage(src: &str) -> Result<f64, PercentageParseError> {
    let value: f64 = src.parse()?;
    if value > 100.0 {
        return Err(PercentageParseError::InvalidPercentage);
    }

    if value == 0.0 {
        return Err(PercentageParseError::CannotSplitAtZero);
    }

    Ok(value)
}

#[derive(StructOpt, Debug)]
#[structopt(name = "biocomputation_ga")]
struct Opt {
    #[structopt(short,long, parse(try_from_str = parse_percentage), default_value="50.0")]
    split_percentage: f64,

    #[structopt(long, parse(from_os_str))]
    spec: PathBuf,

    #[structopt(name = "FILE", parse(from_os_str))]
    data: PathBuf,
}

fn main() {
    if let Err(err) = run_ga() {
        println!("program exited due to error: {}", err);
    }
}
fn run_ga() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let ga_specs_input = GaSpecInput::from_file(opt.spec)?;
    let data_set = dataset::DataSet::from_file(opt.data)?;
    let (training, test) = data_set.split_at_percentage(opt.split_percentage)?;

    let width = training.width().expect("no training data");
    let alphabet = "01";
    let calculated = CalculatedSpecs {
        alphabet,
        max_index: width,
    };
    let ga_specs = (ga_specs_input, calculated).into();

    println!("{:#?}", ga_specs);
    println!("training data set size: {}", training.len());

    let mut population = Population::generate(&ga_specs);

    population.increment_generation();

    for i in 0..ga_specs.max_evolutions {
        let fitness = population.calculate_fitness(&training)?;

        let mut max = None;
        let mut min = None;
        let mut total = 0;

        for CandidateFitness { fitness, candidate } in &fitness {
            match &max {
                Some(max_fitness) => {
                    if fitness > max_fitness {
                        max = Some(*fitness)
                    }
                }
                None => max = Some(*fitness),
            }

            match &min {
                Some(min_fitness) => {
                    if fitness < min_fitness {
                        min = Some(*fitness)
                    }
                }
                None => min = Some(*fitness),
            }

            total += fitness;
        }

        let average: f64 = total as f64 / fitness.len() as f64;
        println!("generation: {}", population.generation());
        println!("population.size={}", population.len());
        println!("population.averageFitness={}", average);
        println!("population.maxFitness={}", max.unwrap());
        println!("population.minFitness={}", min.unwrap());

        let selection = ga_specs.selection.select(&fitness)?;
        println!("{} candidates selected for crossover", selection.len());
        let offsprings = ga_specs.crossover.crossover(&selection)?;
        println!("{} new offsprings", offsprings.len());
        population.append(offsprings);
        ga_specs.mutation.mutate(&mut population, &ga_specs)?;

        if max.unwrap() == training.len() && ga_specs.stop_at_optimum_fitness {
            break;
        }
        population.increment_generation();
    }
    Ok(())
}
