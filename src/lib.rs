pub mod algorithms;
pub mod utils;

use serde::Serialize;
use std::collections::HashMap;

/// Trait for all distance/similarity algorithms.
pub trait Algorithm {
    /// Primary computation — distance for Base, similarity for BaseSimilarity.
    fn compute(&self, sequences: &[Vec<String>]) -> f64;

    /// Maximum possible value between sequences.
    fn maximum(&self, sequences: &[Vec<String>]) -> f64 {
        sequences.iter().map(|s| s.len()).max().unwrap_or(0) as f64
    }

    /// Whether this algorithm computes similarity (true) or distance (false).
    fn is_similarity(&self) -> bool;

    fn distance(&self, sequences: &[Vec<String>]) -> f64 {
        if self.is_similarity() {
            self.maximum(sequences) - self.compute(sequences)
        } else {
            self.compute(sequences)
        }
    }

    fn similarity(&self, sequences: &[Vec<String>]) -> f64 {
        if self.is_similarity() {
            self.compute(sequences)
        } else {
            self.maximum(sequences) - self.compute(sequences)
        }
    }

    fn normalized_distance(&self, sequences: &[Vec<String>]) -> f64 {
        let max = self.maximum(sequences);
        if max == 0.0 {
            return 0.0;
        }
        self.distance(sequences) / max
    }

    fn normalized_similarity(&self, sequences: &[Vec<String>]) -> f64 {
        1.0 - self.normalized_distance(sequences)
    }

    /// Quick answer: empty seqs, single seq, identical seqs, or any empty seq.
    fn quick_answer(&self, sequences: &[Vec<String>]) -> Option<f64> {
        if sequences.is_empty() || sequences.len() == 1 {
            return Some(if self.is_similarity() { self.maximum(sequences) } else { 0.0 });
        }
        if sequences.windows(2).all(|w| w[0] == w[1]) {
            return Some(if self.is_similarity() { self.maximum(sequences) } else { 0.0 });
        }
        if sequences.iter().any(|s| s.is_empty()) {
            return Some(if self.is_similarity() { 0.0 } else { self.maximum(sequences) });
        }
        None
    }

    /// Prepare sequences: split by qval.
    fn prepare(&self, sequences: &[Vec<String>], qval: Option<usize>) -> Vec<Vec<String>> {
        match qval {
            None | Some(0) => {
                // split by words
                sequences.iter().map(|s| {
                    let joined = s.join(" ");
                    if joined.is_empty() {
                        vec![]
                    } else {
                        joined.split_whitespace().map(|w| w.to_string()).collect()
                    }
                }).collect()
            }
            Some(1) => sequences.to_vec(),
            Some(n) => {
                sequences.iter().map(|s| utils::find_ngrams(&s.join(""), n)).collect()
            }
        }
    }

    /// Convert sequences to character-level HashMaps (Counters).
    fn to_counters(&self, sequences: &[Vec<String>]) -> Vec<HashMap<String, usize>> {
        sequences.iter().map(|s| {
            let mut counter = HashMap::new();
            for item in s {
                *counter.entry(item.clone()).or_insert(0) += 1;
            }
            counter
        }).collect()
    }

    fn intersect_counters(&self, counters: &[HashMap<String, usize>]) -> HashMap<String, usize> {
        let mut result = counters[0].clone();
        for c in &counters[1..] {
            result.retain(|k, v| {
                if let Some(count) = c.get(k) {
                    *v = (*v).min(*count);
                    true
                } else {
                    false
                }
            });
        }
        result
    }

    fn union_counters(&self, counters: &[HashMap<String, usize>]) -> HashMap<String, usize> {
        let mut result = counters[0].clone();
        for c in &counters[1..] {
            for (k, v) in c {
                let entry = result.entry(k.clone()).or_insert(0);
                *entry = (*entry).max(*v);
            }
        }
        result
    }

    fn sum_counters(&self, counters: &[HashMap<String, usize>]) -> HashMap<String, usize> {
        let mut result = counters[0].clone();
        for c in &counters[1..] {
            for (k, v) in c {
                *result.entry(k.clone()).or_insert(0) += v;
            }
        }
        result
    }

    fn count_counter(&self, counter: &HashMap<String, usize>, as_set: bool) -> usize {
        if as_set {
            counter.len()
        } else {
            counter.values().sum()
        }
    }
}

#[derive(Serialize)]
pub struct AlgorithmResult {
    pub algorithm: String,
    pub distance: f64,
    pub similarity: f64,
    pub normalized_distance: f64,
    pub normalized_similarity: f64,
}
