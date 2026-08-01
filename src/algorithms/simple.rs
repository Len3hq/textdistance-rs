use crate::Algorithm;
use std::collections::HashMap;

// ── Prefix ──

pub struct Prefix;

impl Algorithm for Prefix {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if sequences.is_empty() {
            return 0.0;
        }
        let min_len = sequences.iter().map(|s| s.len()).min().unwrap_or(0);
        let mut count = 0;
        for i in 0..min_len {
            let first = &sequences[0][i];
            if sequences.iter().all(|s| &s[i] == first) {
                count += 1;
            } else {
                break;
            }
        }
        count as f64
    }

    fn is_similarity(&self) -> bool {
        true
    }
}

// ── Postfix ──

pub struct Postfix;

impl Algorithm for Postfix {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if sequences.is_empty() {
            return 0.0;
        }
        let reversed: Vec<Vec<String>> = sequences
            .iter()
            .map(|s| s.iter().rev().cloned().collect())
            .collect();
        Prefix.compute(&reversed)
    }

    fn is_similarity(&self) -> bool {
        true
    }
}

// ── Length ──

pub struct Length;

impl Algorithm for Length {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        let lengths: Vec<usize> = sequences.iter().map(|s| s.len()).collect();
        let max_len = lengths.iter().max().copied().unwrap_or(0);
        let min_len = lengths.iter().min().copied().unwrap_or(0);
        (max_len - min_len) as f64
    }

    fn is_similarity(&self) -> bool {
        false
    }
}

// ── Identity ──

pub struct Identity;

impl Algorithm for Identity {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if sequences.is_empty() || sequences.len() == 1 {
            return 1.0;
        }
        if sequences.windows(2).all(|w| w[0] == w[1]) {
            1.0
        } else {
            0.0
        }
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        1.0
    }

    fn is_similarity(&self) -> bool {
        true
    }
}

// ── Matrix ──

pub struct Matrix {
    pub mat: Option<HashMap<(String, String), usize>>,
    pub mismatch_cost: usize,
    pub match_cost: usize,
    pub symmetric: bool,
}

impl Matrix {
    pub fn new(
        mat: Option<HashMap<(String, String), usize>>,
        mismatch_cost: usize,
        match_cost: usize,
        symmetric: bool,
    ) -> Self {
        Matrix {
            mat,
            mismatch_cost,
            match_cost,
            symmetric,
        }
    }
}

impl Default for Matrix {
    fn default() -> Self {
        Matrix {
            mat: None,
            mismatch_cost: 0,
            match_cost: 1,
            symmetric: true,
        }
    }
}

impl Algorithm for Matrix {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if sequences.is_empty() || sequences.len() < 2 {
            return self.match_cost as f64;
        }

        let a = &sequences[0][0];
        let b = &sequences[1][0];

        if let Some(ref mat) = self.mat {
            let key = (a.clone(), b.clone());
            if let Some(&val) = mat.get(&key) {
                return val as f64;
            }
            if self.symmetric {
                let rkey = (b.clone(), a.clone());
                if let Some(&val) = mat.get(&rkey) {
                    return val as f64;
                }
            }
        }

        if a == b {
            self.match_cost as f64
        } else {
            self.mismatch_cost as f64
        }
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        self.match_cost as f64
    }

    fn is_similarity(&self) -> bool {
        true
    }
}
