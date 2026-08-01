use crate::Algorithm;
use std::collections::HashMap;

// ── SqrtNCD ──

pub struct SqrtNCD;

impl Algorithm for SqrtNCD {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if sequences.is_empty() {
            return 0.0;
        }
        let prepared = self.prepare(sequences, Some(1));
        let compressed_lens: Vec<f64> = prepared.iter().map(|s| self.get_size(s)).collect();
        let max_len = compressed_lens.iter().cloned().fold(0.0f64, f64::max);
        if max_len == 0.0 {
            return 0.0;
        }
        // Concat: try permutations, pick min
        let mut concat_min = f64::INFINITY;
        let empty: Vec<String> = vec![];
        // Simple: concatenate all (only 2 sequences in practice)
        if prepared.len() == 2 {
            let concat: Vec<String> = prepared[0]
                .iter()
                .chain(prepared[1].iter())
                .cloned()
                .collect();
            concat_min = concat_min.min(self.get_size(&concat));
            let concat2: Vec<String> = prepared[1]
                .iter()
                .chain(prepared[0].iter())
                .cloned()
                .collect();
            concat_min = concat_min.min(self.get_size(&concat2));
        }
        (concat_min
            - compressed_lens
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
                * (prepared.len() as f64 - 1.0))
            / max_len
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        1.0
    }

    fn is_similarity(&self) -> bool {
        false
    }
}

impl SqrtNCD {
    fn get_size(&self, data: &[String]) -> f64 {
        let mut counter: HashMap<String, usize> = HashMap::new();
        for item in data {
            *counter.entry(item.clone()).or_insert(0) += 1;
        }
        counter.values().map(|&c| (c as f64).sqrt()).sum()
    }
}

// ── EntropyNCD ──

pub struct EntropyNCD {
    pub coef: f64,
}

impl EntropyNCD {
    pub fn new(coef: f64) -> Self {
        EntropyNCD { coef }
    }
}

impl Default for EntropyNCD {
    fn default() -> Self {
        EntropyNCD { coef: 1.0 }
    }
}

impl Algorithm for EntropyNCD {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if sequences.is_empty() {
            return 0.0;
        }
        let prepared = self.prepare(sequences, Some(1));
        let compressed_lens: Vec<f64> = prepared.iter().map(|s| self.get_size(s)).collect();
        let max_len = compressed_lens.iter().cloned().fold(0.0f64, f64::max);
        if max_len == 0.0 {
            return 0.0;
        }
        let mut concat_min = f64::INFINITY;
        if prepared.len() == 2 {
            let concat: Vec<String> = prepared[0]
                .iter()
                .chain(prepared[1].iter())
                .cloned()
                .collect();
            concat_min = concat_min.min(self.get_size(&concat));
            let concat2: Vec<String> = prepared[1]
                .iter()
                .chain(prepared[0].iter())
                .cloned()
                .collect();
            concat_min = concat_min.min(self.get_size(&concat2));
        }
        (concat_min
            - compressed_lens
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
                * (prepared.len() as f64 - 1.0))
            / max_len
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        1.0
    }

    fn is_similarity(&self) -> bool {
        false
    }
}

impl EntropyNCD {
    fn get_size(&self, data: &[String]) -> f64 {
        let total = data.len() as f64;
        if total == 0.0 {
            return 0.0;
        }
        let mut counter: HashMap<String, usize> = HashMap::new();
        for item in data {
            *counter.entry(item.clone()).or_insert(0) += 1;
        }
        let mut entropy = 0.0f64;
        for &count in counter.values() {
            let p = count as f64 / total;
            entropy -= p * p.log2();
        }
        self.coef + entropy
    }
}

// ── RLENCD (Run-Length Encoding NCD) ──

pub struct RLENCD;

impl Algorithm for RLENCD {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        ncd_compute(self, sequences)
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        1.0
    }

    fn is_similarity(&self) -> bool {
        false
    }
}

impl RLENCD {
    fn compress(&self, data: &[String]) -> String {
        if data.is_empty() {
            return String::new();
        }
        let mut result = String::new();
        let mut count = 1usize;
        let mut current = &data[0];
        for item in &data[1..] {
            if item == current {
                count += 1;
            } else {
                if count > 2 {
                    result.push_str(&count.to_string());
                } else if count == 2 {
                    result.push_str(current);
                }
                result.push_str(current);
                current = item;
                count = 1;
            }
        }
        if count > 2 {
            result.push_str(&count.to_string());
        } else if count == 2 {
            result.push_str(current);
        }
        result.push_str(current);
        result
    }

    fn get_size(&self, data: &[String]) -> f64 {
        self.compress(data).len() as f64
    }
}

// ── BWTRLENCD ──

pub struct BWTRLENCD;

impl Algorithm for BWTRLENCD {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        ncd_compute(self, sequences)
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        1.0
    }

    fn is_similarity(&self) -> bool {
        false
    }
}

impl BWTRLENCD {
    fn compress(&self, data: &[String]) -> String {
        let text = data.join("");
        if text.is_empty() {
            return String::from("\0");
        }
        // Burrows-Wheeler Transform
        let text_with_term = if !text.contains('\0') {
            text.clone() + "\0"
        } else {
            text.clone()
        };
        let mut rotations: Vec<String> = (0..text_with_term.len())
            .map(|i| {
                let mut s = text_with_term[i..].to_string();
                s.push_str(&text_with_term[..i]);
                s
            })
            .collect();
        rotations.sort();
        let last_col: String = rotations
            .iter()
            .map(|s| s.chars().last().unwrap())
            .collect();
        // Then RLE
        let rle = RLENCD;
        let chars: Vec<String> = last_col.chars().map(|c| c.to_string()).collect();
        rle.compress(&chars)
    }

    fn get_size(&self, data: &[String]) -> f64 {
        self.compress(data).len() as f64
    }
}

// ── ArithNCD ──

pub struct ArithNCD {
    pub base: f64,
}

impl ArithNCD {
    pub fn new(base: f64) -> Self {
        ArithNCD { base }
    }
}

impl Default for ArithNCD {
    fn default() -> Self {
        ArithNCD { base: 2.0 }
    }
}

impl Algorithm for ArithNCD {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        ncd_compute(self, sequences)
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        1.0
    }

    fn is_similarity(&self) -> bool {
        false
    }
}

impl ArithNCD {
    fn compress(&self, data: &[String]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        // Simplified arithmetic coding: use the sum of log probabilities
        let mut counter: HashMap<String, usize> = HashMap::new();
        for item in data {
            *counter.entry(item.clone()).or_insert(0) += 1;
        }
        let total = data.len() as f64;
        let mut size = 0.0f64;
        for &count in counter.values() {
            let p = count as f64 / total;
            size -= p.log2();
        }
        size * total
    }

    fn get_size(&self, data: &[String]) -> f64 {
        let compressed = self.compress(data);
        if compressed == 0.0 {
            return 0.0;
        }
        compressed.log(self.base)
    }
}

// ── BZ2NCD ──

pub struct BZ2NCD;

impl Algorithm for BZ2NCD {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        // BZ2 requires external crate — approximate with entropy
        let entropy = EntropyNCD::default();
        entropy.compute(sequences)
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        1.0
    }

    fn is_similarity(&self) -> bool {
        false
    }
}

// ── LZMANCD ──

pub struct LZMANCD;

impl Algorithm for LZMANCD {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        let entropy = EntropyNCD::default();
        entropy.compute(sequences)
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        1.0
    }

    fn is_similarity(&self) -> bool {
        false
    }
}

// ── ZLIBNCD ──

pub struct ZLIBNCD;

impl Algorithm for ZLIBNCD {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        let entropy = EntropyNCD::default();
        entropy.compute(sequences)
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        1.0
    }

    fn is_similarity(&self) -> bool {
        false
    }
}

// ── Shared NCD computation ──

fn ncd_compute(alg: &dyn NCDLike, sequences: &[Vec<String>]) -> f64 {
    if sequences.is_empty() {
        return 0.0;
    }
    let prepared: Vec<Vec<String>> = sequences.to_vec();
    let compressed_lens: Vec<f64> = prepared.iter().map(|s| alg.get_size(s)).collect();
    let max_len = compressed_lens.iter().cloned().fold(0.0f64, f64::max);
    let min_len = compressed_lens
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);
    if max_len == 0.0 {
        return 0.0;
    }
    let mut concat_min = f64::INFINITY;
    if prepared.len() == 2 {
        let concat: Vec<String> = prepared[0]
            .iter()
            .chain(prepared[1].iter())
            .cloned()
            .collect();
        concat_min = concat_min.min(alg.get_size(&concat));
        let concat2: Vec<String> = prepared[1]
            .iter()
            .chain(prepared[0].iter())
            .cloned()
            .collect();
        concat_min = concat_min.min(alg.get_size(&concat2));
    }
    (concat_min - min_len * (prepared.len() as f64 - 1.0)) / max_len
}

trait NCDLike {
    fn get_size(&self, data: &[String]) -> f64;
}

impl NCDLike for SqrtNCD {
    fn get_size(&self, data: &[String]) -> f64 {
        SqrtNCD::get_size(self, data)
    }
}

impl NCDLike for EntropyNCD {
    fn get_size(&self, data: &[String]) -> f64 {
        EntropyNCD::get_size(self, data)
    }
}

impl NCDLike for RLENCD {
    fn get_size(&self, data: &[String]) -> f64 {
        RLENCD::get_size(self, data)
    }
}

impl NCDLike for BWTRLENCD {
    fn get_size(&self, data: &[String]) -> f64 {
        BWTRLENCD::get_size(self, data)
    }
}

impl NCDLike for ArithNCD {
    fn get_size(&self, data: &[String]) -> f64 {
        ArithNCD::get_size(self, data)
    }
}
