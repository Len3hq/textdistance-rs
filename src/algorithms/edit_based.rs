use crate::Algorithm;
use std::cmp::min;

// ── Hamming ──

pub struct Hamming;

impl Algorithm for Hamming {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if sequences.len() < 2 {
            return 0.0;
        }
        let s1 = &sequences[0];
        let s2 = &sequences[1];
        let max_len = s1.len().max(s2.len());
        let min_len = s1.len().min(s2.len());
        let mut distance = max_len - min_len;
        for i in 0..min_len {
            if s1[i] != s2[i] {
                distance += 1;
            }
        }
        distance as f64
    }

    fn is_similarity(&self) -> bool {
        false
    }
}

// ── Levenshtein ──

pub struct Levenshtein;

impl Algorithm for Levenshtein {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if let Some(result) = self.quick_answer(sequences) {
            return result;
        }
        let s1 = &sequences[0];
        let s2 = &sequences[1];
        let len1 = s1.len();
        let len2 = s2.len();

        if len1 < len2 {
            return Levenshtein.compute(&[sequences[1].clone(), sequences[0].clone()]);
        }

        // Use two-row DP for memory efficiency
        let mut prev: Vec<usize> = (0..=len2).collect();
        let mut curr = vec![0usize; len2 + 1];

        for i in 1..=len1 {
            curr[0] = i;
            for j in 1..=len2 {
                let cost = if s1[i - 1] == s2[j - 1] { 0 } else { 1 };
                curr[j] = min(min(prev[j] + 1, curr[j - 1] + 1), prev[j - 1] + cost);
            }
            std::mem::swap(&mut prev, &mut curr);
        }
        prev[len2] as f64
    }

    fn is_similarity(&self) -> bool {
        false
    }
}

// ── DamerauLevenshtein ──

pub struct DamerauLevenshtein {
    pub restricted: bool,
}

impl DamerauLevenshtein {
    pub fn new(restricted: bool) -> Self {
        DamerauLevenshtein { restricted }
    }
}

impl Default for DamerauLevenshtein {
    fn default() -> Self {
        DamerauLevenshtein { restricted: true }
    }
}

impl Algorithm for DamerauLevenshtein {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if let Some(result) = self.quick_answer(sequences) {
            return result;
        }
        if self.restricted {
            self.restricted_damerau(sequences)
        } else {
            self.unrestricted_damerau(sequences)
        }
    }

    fn is_similarity(&self) -> bool {
        false
    }
}

impl DamerauLevenshtein {
    fn restricted_damerau(&self, sequences: &[Vec<String>]) -> f64 {
        let s1 = &sequences[0];
        let s2 = &sequences[1];
        let len1 = s1.len();
        let len2 = s2.len();

        // Optimal String Alignment (restricted edit distance)
        let mut d: Vec<Vec<usize>> = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            d[i][0] = i;
        }
        for j in 0..=len2 {
            d[0][j] = j;
        }

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1[i - 1] == s2[j - 1] { 0 } else { 1 };
                d[i][j] = min(
                    min(d[i - 1][j] + 1, d[i][j - 1] + 1),
                    d[i - 1][j - 1] + cost,
                );
                if i > 1 && j > 1 && s1[i - 1] == s2[j - 2] && s1[i - 2] == s2[j - 1] {
                    d[i][j] = min(d[i][j], d[i - 2][j - 2] + cost);
                }
            }
        }
        d[len1][len2] as f64
    }

    fn unrestricted_damerau(&self, sequences: &[Vec<String>]) -> f64 {
        let s1 = &sequences[0];
        let s2 = &sequences[1];
        let len1 = s1.len();
        let len2 = s2.len();

        let max_dist = len1 + len2;
        let mut d: Vec<Vec<usize>> = vec![vec![0; len2 + 2]; len1 + 2];

        d[0][0] = max_dist;
        for i in 0..=len1 {
            d[i + 1][1] = i;
            d[i + 1][0] = max_dist;
        }
        for j in 0..=len2 {
            d[1][j + 1] = j;
            d[0][j + 1] = max_dist;
        }

        // Last position of each character
        use std::collections::HashMap;
        let mut last_row: HashMap<String, usize> = HashMap::new();

        for i in 1..=len1 {
            let mut last_match_col = 0;
            let mut last_match_row;
            let ch1 = &s1[i - 1];

            for j in 1..=len2 {
                let ch2 = &s2[j - 1];
                last_match_row = *last_row.get(ch2).unwrap_or(&0);
                let cost = if ch1 == ch2 { 0 } else { 1 };

                let mut val = min(min(d[i][j + 1] + 1, d[i + 1][j] + 1), d[i][j] + cost);

                if last_match_row > 0 && last_match_col > 0 {
                    let k = last_match_row - 1;
                    let l = last_match_col - 1;
                    val = min(val, d[k][l] + (i - k - 1) + 1 + (j - l - 1));
                }

                d[i + 1][j + 1] = val;

                if cost == 0 {
                    last_match_col = j;
                }
            }
            last_row.insert(ch1.clone(), i);
        }

        d[len1 + 1][len2 + 1] as f64
    }
}

// ── Jaro ──

pub struct Jaro;

impl Algorithm for Jaro {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if let Some(result) = self.quick_answer(sequences) {
            return if self.is_similarity() {
                self.maximum(sequences) - result
            } else {
                result
            };
        }
        let s1 = &sequences[0];
        let s2 = &sequences[1];

        let len1 = s1.len();
        let len2 = s2.len();

        if len1 == 0 || len2 == 0 {
            return 0.0;
        }

        let match_distance = (len1.max(len2) / 2).saturating_sub(1);

        let mut s1_matches = vec![false; len1];
        let mut s2_matches = vec![false; len2];
        let mut matches = 0;

        for i in 0..len1 {
            let start = if i >= match_distance {
                i - match_distance
            } else {
                0
            };
            let end = (i + match_distance + 1).min(len2);

            for j in start..end {
                if !s2_matches[j] && s1[i] == s2[j] {
                    s1_matches[i] = true;
                    s2_matches[j] = true;
                    matches += 1;
                    break;
                }
            }
        }

        if matches == 0 {
            return 0.0;
        }

        // Transpositions
        let mut transpositions = 0;
        let mut k = 0;
        for i in 0..len1 {
            if !s1_matches[i] {
                continue;
            }
            while !s2_matches[k] {
                k += 1;
            }
            if s1[i] != s2[k] {
                transpositions += 1;
            }
            k += 1;
        }

        let jaro = ((matches as f64 / len1 as f64)
            + (matches as f64 / len2 as f64)
            + ((matches - transpositions / 2) as f64 / matches as f64))
            / 3.0;

        jaro
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        1.0
    }

    fn is_similarity(&self) -> bool {
        true
    }
}

// ── JaroWinkler ──

pub struct JaroWinkler {
    pub winklerize: bool,
}

impl JaroWinkler {
    pub fn new(winklerize: bool) -> Self {
        JaroWinkler { winklerize }
    }
}

impl Default for JaroWinkler {
    fn default() -> Self {
        JaroWinkler { winklerize: true }
    }
}

impl Algorithm for JaroWinkler {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        let jaro = Jaro.compute(sequences);
        if !self.winklerize || jaro == 0.0 {
            return jaro;
        }

        let s1 = &sequences[0];
        let s2 = &sequences[1];

        // Prefix length (max 4)
        let prefix_len = s1
            .iter()
            .zip(s2.iter())
            .take(4)
            .take_while(|(a, b)| a == b)
            .count();

        // Scaling factor = 0.1
        jaro + (prefix_len as f64 * 0.1 * (1.0 - jaro))
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        1.0
    }

    fn is_similarity(&self) -> bool {
        true
    }
}

// ── NeedlemanWunsch ──

pub struct NeedlemanWunsch {
    pub gap_cost: f64,
    pub gap_extension_cost: f64,
    pub sim_func: fn(&str, &str) -> f64,
}

impl NeedlemanWunsch {
    pub fn new(gap_cost: f64, gap_extension_cost: f64, sim_func: fn(&str, &str) -> f64) -> Self {
        NeedlemanWunsch {
            gap_cost,
            gap_extension_cost,
            sim_func,
        }
    }
}

impl Default for NeedlemanWunsch {
    fn default() -> Self {
        NeedlemanWunsch {
            gap_cost: 1.0,
            gap_extension_cost: 0.5,
            sim_func: |a, b| if a == b { 1.0 } else { -1.0 },
        }
    }
}

impl Algorithm for NeedlemanWunsch {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if sequences.len() < 2 {
            return 0.0;
        }
        let s1 = &sequences[0];
        let s2 = &sequences[1];
        let len1 = s1.len();
        let len2 = s2.len();

        let mut d: Vec<Vec<f64>> = vec![vec![0.0; len2 + 1]; len1 + 1];

        // Initialize gaps
        for i in 1..=len1 {
            d[i][0] = d[i - 1][0] - self.gap_cost;
        }
        for j in 1..=len2 {
            d[0][j] = d[0][j - 1] - self.gap_cost;
        }

        for i in 1..=len1 {
            for j in 1..=len2 {
                let match_val = d[i - 1][j - 1] + (self.sim_func)(&s1[i - 1], &s2[j - 1]);
                let delete = d[i - 1][j] - self.gap_cost;
                let insert = d[i][j - 1] - self.gap_cost;
                d[i][j] = match_val.max(delete).max(insert);
            }
        }
        d[len1][len2]
    }

    fn distance(&self, sequences: &[Vec<String>]) -> f64 {
        -self.compute(sequences)
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        0.0
    }

    fn is_similarity(&self) -> bool {
        true
    }
}

// ── SmithWaterman ──

pub struct SmithWaterman {
    pub gap_cost: f64,
    pub sim_func: fn(&str, &str) -> f64,
}

impl SmithWaterman {
    pub fn new(gap_cost: f64, sim_func: fn(&str, &str) -> f64) -> Self {
        SmithWaterman { gap_cost, sim_func }
    }
}

impl Default for SmithWaterman {
    fn default() -> Self {
        SmithWaterman {
            gap_cost: 1.0,
            sim_func: |a, b| if a == b { 1.0 } else { -1.0 / 3.0 },
        }
    }
}

impl Algorithm for SmithWaterman {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if sequences.len() < 2 {
            return 0.0;
        }
        let s1 = &sequences[0];
        let s2 = &sequences[1];
        let len1 = s1.len();
        let len2 = s2.len();

        let mut d: Vec<Vec<f64>> = vec![vec![0.0; len2 + 1]; len1 + 1];
        let mut max_val = 0.0f64;

        for i in 1..=len1 {
            for j in 1..=len2 {
                let match_val = d[i - 1][j - 1] + (self.sim_func)(&s1[i - 1], &s2[j - 1]);
                let delete = d[i - 1][j] - self.gap_cost;
                let insert = d[i][j - 1] - self.gap_cost;
                d[i][j] = 0.0f64.max(match_val).max(delete).max(insert);
                if d[i][j] > max_val {
                    max_val = d[i][j];
                }
            }
        }
        max_val
    }

    fn maximum(&self, sequences: &[Vec<String>]) -> f64 {
        let min_len = sequences.iter().map(|s| s.len()).min().unwrap_or(0) as f64;
        min_len
    }

    fn is_similarity(&self) -> bool {
        true
    }
}

// ── Gotoh ──

pub struct Gotoh {
    pub gap_open: f64,
    pub gap_extend: f64,
    pub sim_func: fn(&str, &str) -> f64,
}

impl Gotoh {
    pub fn new(gap_open: f64, gap_extend: f64, sim_func: fn(&str, &str) -> f64) -> Self {
        Gotoh {
            gap_open,
            gap_extend,
            sim_func,
        }
    }
}

impl Default for Gotoh {
    fn default() -> Self {
        Gotoh {
            gap_open: 1.0,
            gap_extend: 0.5,
            sim_func: |a, b| if a == b { 1.0 } else { -1.0 },
        }
    }
}

impl Algorithm for Gotoh {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if sequences.len() < 2 {
            return 0.0;
        }
        let s1 = &sequences[0];
        let s2 = &sequences[1];
        let len1 = s1.len();
        let len2 = s2.len();

        let mut d: Vec<Vec<f64>> = vec![vec![0.0; len2 + 1]; len1 + 1];
        let mut p: Vec<Vec<f64>> = vec![vec![0.0; len2 + 1]; len1 + 1];
        let mut q: Vec<Vec<f64>> = vec![vec![0.0; len2 + 1]; len1 + 1];

        let neg_inf = f64::NEG_INFINITY;

        for i in 1..=len1 {
            d[i][0] = -self.gap_open - self.gap_extend * (i as f64 - 1.0);
            p[i][0] = neg_inf;
            q[i][0] = neg_inf;
        }
        for j in 1..=len2 {
            d[0][j] = -self.gap_open - self.gap_extend * (j as f64 - 1.0);
            p[0][j] = neg_inf;
            q[0][j] = neg_inf;
        }

        for i in 1..=len1 {
            for j in 1..=len2 {
                let sim = (self.sim_func)(&s1[i - 1], &s2[j - 1]);
                p[i][j] = (d[i - 1][j] - self.gap_open).max(p[i - 1][j] - self.gap_extend);
                q[i][j] = (d[i][j - 1] - self.gap_open).max(q[i][j - 1] - self.gap_extend);
                d[i][j] = (d[i - 1][j - 1] + sim).max(p[i][j]).max(q[i][j]);
            }
        }
        d[len1][len2]
    }

    fn maximum(&self, sequences: &[Vec<String>]) -> f64 {
        let min_len = sequences.iter().map(|s| s.len()).min().unwrap_or(0) as f64;
        min_len
    }

    fn is_similarity(&self) -> bool {
        true
    }
}

// ── StrCmp95 ──

pub struct StrCmp95;

impl Algorithm for StrCmp95 {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if sequences.len() < 2 {
            return 0.0;
        }
        let s1 = &sequences[0];
        let s2 = &sequences[1];
        let s1_str: String = s1.join("");
        let s2_str: String = s2.join("");

        let len1 = s1_str.len();
        let len2 = s2_str.len();

        if len1 == 0 && len2 == 0 {
            return 1.0;
        }

        let sp = |c: char| -> bool { c.is_ascii_whitespace() || c.is_ascii_punctuation() };

        // Find span of first non-space/punctuation chars
        let mut span1 = 0;
        for c in s1_str.chars() {
            if sp(c) {
                break;
            }
            span1 += 1;
        }
        let mut span2 = 0;
        for c in s2_str.chars() {
            if sp(c) {
                break;
            }
            span2 += 1;
        }

        let mut score = 0.0;
        // Longest common substring
        let lcs = {
            let mut max_len = 0;
            for i in 0..len1 {
                for j in 0..len2 {
                    let mut k = 0;
                    while i + k < len1
                        && j + k < len2
                        && s1_str.as_bytes()[i + k] == s2_str.as_bytes()[j + k]
                    {
                        k += 1;
                    }
                    if k > max_len {
                        max_len = k;
                    }
                }
            }
            max_len
        };

        if lcs > 0 {
            let m = 2.0 * lcs as f64 / (len1 + len2) as f64;
            score = m + m * 0.1 * (span1.min(span2) as f64 / m.max(1.0) as f64);
        }
        score.min(1.0)
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        1.0
    }

    fn is_similarity(&self) -> bool {
        true
    }
}

// ── MLIPNS ──

pub struct MLIPNS {
    pub threshold: f64,
}

impl MLIPNS {
    pub fn new(threshold: f64) -> Self {
        MLIPNS { threshold }
    }
}

impl Default for MLIPNS {
    fn default() -> Self {
        MLIPNS { threshold: 0.5 }
    }
}

impl Algorithm for MLIPNS {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if sequences.len() < 2 {
            return 0.0;
        }
        let s1 = &sequences[0];
        let s2 = &sequences[1];
        let _max_len = s1.len().max(s2.len());
        let mut distance = 0usize;
        let mut prev_match = false;

        for (i, c1) in s1.iter().enumerate() {
            if i < s2.len() && c1 == &s2[i] {
                prev_match = true;
            } else if !prev_match || i >= s2.len() {
                distance += 1;
                prev_match = false;
            }
        }

        // Add remaining unmatched chars
        if s2.len() > s1.len() {
            distance += s2.len() - s1.len();
        }

        distance as f64
    }

    fn maximum(&self, sequences: &[Vec<String>]) -> f64 {
        sequences.iter().map(|s| s.len()).max().unwrap_or(0) as f64
    }

    fn is_similarity(&self) -> bool {
        false
    }
}
