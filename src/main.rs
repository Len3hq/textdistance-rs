use clap::{Parser, Subcommand};
use serde::Serialize;
use textdistance_rs::algorithms::{edit_based, phonetic, sequence_based, simple, token_based};
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
    /// Jaccard similarity
    Jaccard {
        sequences: Vec<String>,
        #[arg(long)]
        qval: Option<usize>,
        #[arg(long)]
        as_set: bool,
    },
    /// Sorensen (Dice) similarity
    Sorensen {
        sequences: Vec<String>,
        #[arg(long)]
        qval: Option<usize>,
        #[arg(long)]
        as_set: bool,
    },
    /// Tversky similarity
    Tversky {
        sequences: Vec<String>,
        #[arg(long)]
        qval: Option<usize>,
        #[arg(long, num_args = 1.., default_values_t = [1.0, 1.0])]
        ks: Vec<f64>,
        #[arg(long)]
        bias: Option<f64>,
        #[arg(long)]
        as_set: bool,
    },
    /// Overlap similarity
    Overlap {
        sequences: Vec<String>,
        #[arg(long)]
        qval: Option<usize>,
        #[arg(long)]
        as_set: bool,
    },
    /// Cosine similarity
    Cosine {
        sequences: Vec<String>,
        #[arg(long)]
        qval: Option<usize>,
        #[arg(long)]
        as_set: bool,
    },
    /// Tanimoto similarity
    Tanimoto {
        sequences: Vec<String>,
        #[arg(long)]
        qval: Option<usize>,
        #[arg(long)]
        as_set: bool,
    },
    /// Bag distance
    Bag { sequences: Vec<String> },
    /// Hamming distance
    Hamming { sequences: Vec<String> },
    /// Levenshtein distance
    Levenshtein { sequences: Vec<String> },
    /// Damerau-Levenshtein distance
    DamerauLevenshtein {
        sequences: Vec<String>,
        #[arg(long)]
        restricted: bool,
    },
    /// Jaro similarity
    Jaro { sequences: Vec<String> },
    /// Jaro-Winkler similarity
    JaroWinkler {
        sequences: Vec<String>,
        #[arg(long)]
        winklerize: bool,
    },
    /// Needleman-Wunsch similarity
    NeedlemanWunsch { sequences: Vec<String> },
    /// Smith-Waterman similarity
    SmithWaterman { sequences: Vec<String> },
    /// Gotoh similarity
    Gotoh { sequences: Vec<String> },
    /// StrCmp95 similarity
    StrCmp95 { sequences: Vec<String> },
    /// MLIPNS distance
    MLIPNS { sequences: Vec<String> },
    /// LCSSeq similarity
    Lcsseq { sequences: Vec<String> },
    /// LCSStr similarity
    Lcsstr { sequences: Vec<String> },
    /// Ratcliff-Obershelp similarity
    RatcliffObershelp { sequences: Vec<String> },
    /// MRA similarity
    Mra { sequences: Vec<String> },
    /// Editex distance
    Editex { sequences: Vec<String> },
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
        Commands::Jaccard {
            sequences,
            qval,
            as_set,
        } => {
            let seqs = to_vec_strings(&sequences);
            let alg = token_based::Jaccard::new(qval, as_set);
            run_algorithm(&alg, "jaccard", &seqs);
        }
        Commands::Sorensen {
            sequences,
            qval,
            as_set,
        } => {
            let seqs = to_vec_strings(&sequences);
            let alg = token_based::Sorensen::new(qval, as_set);
            run_algorithm(&alg, "sorensen", &seqs);
        }
        Commands::Tversky {
            sequences,
            qval,
            ks,
            bias,
            as_set,
        } => {
            let seqs = to_vec_strings(&sequences);
            let alg = token_based::Tversky::new(qval, ks, bias, as_set);
            run_algorithm(&alg, "tversky", &seqs);
        }
        Commands::Overlap {
            sequences,
            qval,
            as_set,
        } => {
            let seqs = to_vec_strings(&sequences);
            let alg = token_based::Overlap::new(qval, as_set);
            run_algorithm(&alg, "overlap", &seqs);
        }
        Commands::Cosine {
            sequences,
            qval,
            as_set,
        } => {
            let seqs = to_vec_strings(&sequences);
            let alg = token_based::Cosine::new(qval, as_set);
            run_algorithm(&alg, "cosine", &seqs);
        }
        Commands::Tanimoto {
            sequences,
            qval,
            as_set,
        } => {
            let seqs = to_vec_strings(&sequences);
            let alg = token_based::Tanimoto::new(qval, as_set);
            run_algorithm(&alg, "tanimoto", &seqs);
        }
        Commands::Bag { sequences } => {
            let seqs = to_vec_strings(&sequences);
            run_algorithm(&token_based::Bag, "bag", &seqs);
        }
        Commands::Hamming { sequences } => {
            let seqs = to_vec_strings(&sequences);
            run_algorithm(&edit_based::Hamming, "hamming", &seqs);
        }
        Commands::Levenshtein { sequences } => {
            let seqs = to_vec_strings(&sequences);
            run_algorithm(&edit_based::Levenshtein, "levenshtein", &seqs);
        }
        Commands::DamerauLevenshtein {
            sequences,
            restricted,
        } => {
            let seqs = to_vec_strings(&sequences);
            let alg = edit_based::DamerauLevenshtein::new(restricted);
            run_algorithm(&alg, "damerau_levenshtein", &seqs);
        }
        Commands::Jaro { sequences } => {
            let seqs = to_vec_strings(&sequences);
            run_algorithm(&edit_based::Jaro, "jaro", &seqs);
        }
        Commands::JaroWinkler {
            sequences,
            winklerize,
        } => {
            let seqs = to_vec_strings(&sequences);
            let alg = edit_based::JaroWinkler::new(winklerize);
            run_algorithm(&alg, "jaro_winkler", &seqs);
        }
        Commands::NeedlemanWunsch { sequences } => {
            let seqs = to_vec_strings(&sequences);
            run_algorithm(
                &edit_based::NeedlemanWunsch::default(),
                "needleman_wunsch",
                &seqs,
            );
        }
        Commands::SmithWaterman { sequences } => {
            let seqs = to_vec_strings(&sequences);
            run_algorithm(
                &edit_based::SmithWaterman::default(),
                "smith_waterman",
                &seqs,
            );
        }
        Commands::Gotoh { sequences } => {
            let seqs = to_vec_strings(&sequences);
            run_algorithm(&edit_based::Gotoh::default(), "gotoh", &seqs);
        }
        Commands::StrCmp95 { sequences } => {
            let seqs = to_vec_strings(&sequences);
            run_algorithm(&edit_based::StrCmp95, "strcmp95", &seqs);
        }
        Commands::MLIPNS { sequences } => {
            let seqs = to_vec_strings(&sequences);
            run_algorithm(&edit_based::MLIPNS::default(), "mlipns", &seqs);
        }
        Commands::Lcsseq { sequences } => {
            let seqs = to_vec_strings(&sequences);
            run_algorithm(&sequence_based::LCSSeq, "lcsseq", &seqs);
        }
        Commands::Lcsstr { sequences } => {
            let seqs = to_vec_strings(&sequences);
            run_algorithm(&sequence_based::LCSStr, "lcsstr", &seqs);
        }
        Commands::RatcliffObershelp { sequences } => {
            let seqs = to_vec_strings(&sequences);
            run_algorithm(
                &sequence_based::RatcliffObershelp,
                "ratcliff_obershelp",
                &seqs,
            );
        }
        Commands::Mra { sequences } => {
            let seqs = to_vec_strings(&sequences);
            run_algorithm(&phonetic::MRA, "mra", &seqs);
        }
        Commands::Editex { sequences } => {
            let seqs = to_vec_strings(&sequences);
            run_algorithm(&phonetic::Editex::default(), "editex", &seqs);
        }
    }
}
