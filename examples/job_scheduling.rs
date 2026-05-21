/*!
# Job Scheduling -- Permutation-Based Makespan Minimization

This example demonstrates how to use the `genetic_algorithms` library to solve a parallel machine
scheduling problem using permutation encoding.

## Problem

15 jobs must be scheduled on 5 machines. Each job has a known processing time on each machine.
The goal is to find the job ordering (permutation) that minimizes the **makespan** -- the total
completion time, defined as the maximum finish time across all machines.

## Representation

- Chromosome: `RangeChromosome<i32>` where each gene holds a job index in `[0, 14]`
- Each chromosome is a permutation of job indices -- no repetition allowed
- Permutation initialization is achieved by passing `Some(false)` to `range_random_initialization`

## Operators

- **Order crossover (OX)** (`Crossover::Order`): preserves the relative ordering of jobs from
  each parent, ensuring the offspring is also a valid permutation
- **Insertion mutation** (`Mutation::Insertion`): picks a random gene and moves it to a new
  random position, producing another valid permutation

Both operators are permutation-safe and will never introduce duplicate job indices.

## Features demonstrated
- Permutation chromosomes (`Range<i32>`) for combinatorial optimization
- Order crossover (OX) and Insertion mutation
- Minimization mode (makespan)
- LogObserver lifecycle hooks

## Scheduling Heuristic

The fitness function uses a greedy FIFO heuristic: each job in the permutation order is assigned
to the machine that has the earliest available finish time. The makespan is the maximum finish
time across all machines after all jobs have been scheduled.

## Run

```sh
cargo run --example job_scheduling
```
*/

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::{Ga, TerminationCause};
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::population::Population;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::{
    ConfigurationT, CrossoverConfig, LinearChromosome, MutationConfig, SelectionConfig,
    StoppingConfig,
};
use genetic_algorithms::LogObserver;
use std::sync::Arc;

/// Processing times matrix: PROCESSING_TIMES[job][machine] = time units for job on that machine.
/// 15 jobs x 5 machines, values in range 1-20.
const PROCESSING_TIMES: [[u32; 5]; 15] = [
    [5, 10, 6, 3, 7],  // Job 0
    [8, 3, 12, 5, 9],  // Job 1
    [4, 7, 3, 11, 6],  // Job 2
    [9, 5, 8, 4, 12],  // Job 3
    [3, 12, 7, 9, 5],  // Job 4
    [11, 4, 9, 6, 3],  // Job 5
    [6, 8, 5, 12, 10], // Job 6
    [7, 6, 11, 3, 8],  // Job 7
    [12, 9, 4, 7, 5],  // Job 8
    [5, 11, 8, 10, 4], // Job 9
    [8, 3, 6, 5, 11],  // Job 10
    [4, 10, 12, 8, 7], // Job 11
    [10, 7, 3, 6, 9],  // Job 12
    [6, 5, 9, 12, 3],  // Job 13
    [3, 8, 10, 4, 6],  // Job 14
];

fn main() {
    // --- Problem parameters ---
    const N_JOBS: usize = 15;
    const N_MACHINES: usize = 5;
    const POP_SIZE: usize = 100;
    const MAX_GENERATIONS: usize = 500;

    // --- Fitness function: greedy parallel machine scheduling (minimize makespan) ---
    // Jobs are assigned in permutation order to whichever machine finishes earliest.
    let fitness_fn = |dna: &[RangeGenotype<i32>]| -> f64 {
        let mut machine_finish = [0u32; N_MACHINES];
        for gene in dna {
            let job = gene.value as usize;
            // Assign job to the machine with the earliest available finish time
            let (m, _) = machine_finish
                .iter()
                .enumerate()
                .min_by_key(|(_, &t)| t)
                .unwrap();
            machine_finish[m] += PROCESSING_TIMES[job][m];
        }
        *machine_finish.iter().max().unwrap() as f64
    };

    // --- Alleles: each gene is a job index in [0, N_JOBS-1] ---
    // RangeGenotype::new(id, ranges, default_value)
    let alleles = vec![RangeGenotype::new(0, vec![(0, N_JOBS as i32 - 1)], 0)];
    let _alleles_clone = alleles.clone();

    // --- Build the GA configuration ---
    let mut ga = Ga::new()
        // Each chromosome encodes a permutation of N_JOBS job indices
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(N_JOBS))
        .with_population_size(POP_SIZE)
        // Phase 47 compile-fix: manual permutation initializer;
        // Phase 48 will migrate to UniqueChromosome<i32> for cleaner permutation support.
        .with_initialization_fn(move |n, _| {
            use rand::seq::SliceRandom;
            let mut rng = rand::rng();
            let mut ids: Vec<i32> = (0..n as i32).collect();
            ids.shuffle(&mut rng);
            ids.iter().enumerate()
                .map(|(i, &job)| RangeGenotype::new(i as i32, vec![(0, n as i32 - 1)], job))
                .collect()
        })
        .with_fitness_fn(fitness_fn)
        // Selection: Tournament -- competitive pressure, works well for combinatorial problems
        .with_selection_method(Selection::Tournament)
        // Crossover: Order (OX) -- preserves relative job order, permutation-safe
        .with_crossover_method(Crossover::Order)
        // Mutation: Insertion -- moves a job to a new position, permutation-safe
        .with_mutation_method(Mutation::Insertion)
        // Survivor selection: Fitness-based
        .with_survivor_method(Survivor::Fitness)
        // Objective: minimize makespan
        .with_problem_solving(ProblemSolving::Minimization)
        .with_max_generations(MAX_GENERATIONS)
        // Observer: LogObserver logs every lifecycle hook via the `log` crate
        .with_observer(Arc::new(LogObserver))
        .build()
        .expect("Failed to build GA configuration");

    println!("== Job Scheduling -- Permutation-Based Makespan Minimization ==");
    println!(
        "Jobs: {}, Machines: {}, Population: {}, Max generations: {}",
        N_JOBS, N_MACHINES, POP_SIZE, MAX_GENERATIONS
    );
    println!("Operators: Selection=Tournament, Crossover=Order, Mutation=Insertion");
    println!("-------------------------------------------------------");

    // --- Run the GA with a progress callback every 50 generations ---
    let report_interval = 50;
    let result = ga.run_with_callback(
        Some(
            |gen: &usize,
             pop: &Population<RangeChromosome<i32>>,
             _stats: &GenerationStats,
             _cause: &TerminationCause|
             -> std::ops::ControlFlow<()> {
                println!(
                    "Generation {:4}: best makespan = {:.0}",
                    gen, pop.best_chromosome.fitness
                );
                std::ops::ControlFlow::Continue(())
            },
        ),
        report_interval,
    );

    // --- Show the final result ---
    match result {
        Ok(population) => {
            let dna = population.best_chromosome.dna();
            let ordering = dna
                .iter()
                .map(|g| g.value.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            println!("-------------------------------------------------------");
            println!("Best makespan: {:.0}", population.best_chromosome.fitness);
            println!("Best ordering: [{}]", ordering);
        }
        Err(e) => {
            eprintln!("GA failed: {:?}", e);
        }
    }
}
