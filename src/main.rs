use clap::{Parser, Subcommand};
use serde::Serialize;
use textdistance_rs::algorithms::simple;
use textdistance_rs::Algorithm;

#[derive(Parser)]
#[command(name = "textdistance-rs")]
#[command(about = "Compute distance between sequences")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Prefix similarity
    Prefix { sequences: Vec<String> },
    /// Postfix similarity
    Postfix { sequences: Vec<String> },
    /// Length distance
    Length { sequences: Vec<String> },
    /// Identity similarity
    Identity { sequences: Vec<String> },
    /// Matrix similarity
    Matrix {
        sequences: Vec<String>,
        #[arg(long, default_value = "0")]
        mismatch_cost: usize,
        #[arg(long, default_value = "1")]
        match_cost: usize,
    },
}

#[derive(Serialize)]
struct Output {
    algorithm: String,
    distance: f64,
    similarity: f64,
    normalized_distance: f64,
    normalized_similarity: f64,
}

fn to_vec_strings(raw: &[String]) -> Vec<Vec<String>> {
    raw.iter()
        .map(|s| s.chars().map(|c| c.to_string()).collect())
        .collect()
}

fn run_algorithm(alg: &dyn Algorithm, name: &str, sequences: &[Vec<String>]) {
    let output = Output {
        algorithm: name.to_string(),
        distance: alg.distance(sequences),
        similarity: alg.similarity(sequences),
        normalized_distance: alg.normalized_distance(sequences),
        normalized_similarity: alg.normalized_similarity(sequences),
    };
    println!("{}", serde_json::to_string(&output).unwrap());
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prefix { sequences } => {
            let seqs = to_vec_strings(&sequences);
            run_algorithm(&simple::Prefix, "prefix", &seqs);
        }
        Commands::Postfix { sequences } => {
            let seqs = to_vec_strings(&sequences);
            run_algorithm(&simple::Postfix, "postfix", &seqs);
        }
        Commands::Length { sequences } => {
            let seqs = to_vec_strings(&sequences);
            run_algorithm(&simple::Length, "length", &seqs);
        }
        Commands::Identity { sequences } => {
            let seqs = to_vec_strings(&sequences);
            run_algorithm(&simple::Identity, "identity", &seqs);
        }
        Commands::Matrix {
            sequences,
            mismatch_cost,
            match_cost,
        } => {
            let seqs = to_vec_strings(&sequences);
            let alg = simple::Matrix::new(None, mismatch_cost, match_cost, true);
            run_algorithm(&alg, "matrix", &seqs);
        }
    }
}
