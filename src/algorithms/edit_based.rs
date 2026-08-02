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

        // Standard optimal string alignment with full transposition support.
        // Based on: https://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance
        let mut d = vec![vec![0usize; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            d[i][0] = i;
        }
        for j in 0..=len2 {
            d[0][j] = j;
        }

        use std::collections::HashMap;
        let mut last_row: HashMap<String, usize> = HashMap::new();

        for i in 1..=len1 {
            let mut db = 0usize;
            for j in 1..=len2 {
                let i1 = *last_row.get(&s2[j - 1]).unwrap_or(&0);
                let j1 = db;

                let cost = if s1[i - 1] == s2[j - 1] { 0 } else { 1 };
                if cost == 0 {
                    db = j;
                }

                d[i][j] = (d[i - 1][j - 1] + cost)
                    .min(d[i][j - 1] + 1)
                    .min(d[i - 1][j] + 1);

                if i1 > 0 && j1 > 0 {
                    d[i][j] = d[i][j].min(d[i1 - 1][j1 - 1] + (i - i1 - 1) + 1 + (j - j1 - 1));
                }
            }
            last_row.insert(s1[i - 1].clone(), i);
        }

        d[len1][len2] as f64
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
        let s1_str: String = sequences[0].join("");
        let s2_str: String = sequences[1].join("");

        let s1: Vec<char> = s1_str.trim().to_uppercase().chars().collect();
        let s2: Vec<char> = s2_str.trim().to_uppercase().chars().collect();

        if let Some(result) = self.quick_answer_str(&s1, &s2) {
            return result;
        }

        strcmp95_impl(&s1, &s2)
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        1.0
    }

    fn is_similarity(&self) -> bool {
        true
    }
}

impl StrCmp95 {
    fn quick_answer_str(&self, s1: &[char], s2: &[char]) -> Option<f64> {
        if s1.is_empty() && s2.is_empty() {
            return Some(1.0);
        }
        if s1.is_empty() || s2.is_empty() {
            return Some(0.0);
        }
        if s1 == s2 {
            return Some(1.0);
        }
        None
    }
}

fn strcmp95_impl(s1: &[char], s2: &[char]) -> f64 {
    use std::collections::HashMap;

    let len1 = s1.len();
    let len2 = s2.len();

    // Phonetic/keyboard proximity table (sp_mx)
    let sp_mx: [(&str, &str); 42] = [
        ("A", "E"),
        ("A", "I"),
        ("A", "O"),
        ("A", "U"),
        ("B", "V"),
        ("E", "I"),
        ("E", "O"),
        ("E", "U"),
        ("I", "O"),
        ("I", "U"),
        ("O", "U"),
        ("I", "Y"),
        ("E", "Y"),
        ("C", "G"),
        ("E", "F"),
        ("W", "U"),
        ("W", "V"),
        ("X", "K"),
        ("S", "Z"),
        ("X", "S"),
        ("Q", "C"),
        ("U", "V"),
        ("M", "N"),
        ("L", "I"),
        ("Q", "O"),
        ("P", "R"),
        ("I", "J"),
        ("2", "Z"),
        ("5", "S"),
        ("8", "B"),
        ("1", "I"),
        ("1", "L"),
        ("0", "O"),
        ("0", "Q"),
        ("C", "K"),
        ("G", "J"),
        ("E", " "),
        ("Y", " "),
        ("S", " "),
        (" ", "S"),
        (" ", "Y"),
        (" ", "E"),
    ];

    let mut adjwt: HashMap<(String, String), usize> = HashMap::new();
    for (c1, c2) in &sp_mx {
        adjwt.insert((c1.to_string(), c2.to_string()), 3);
        adjwt.insert((c2.to_string(), c1.to_string()), 3);
    }

    let search_range;
    let minv;
    if len1 > len2 {
        search_range = len1;
        minv = len2;
    } else {
        search_range = len2;
        minv = len1;
    }

    let mut s1_flag = vec![0u8; search_range];
    let mut s2_flag = vec![0u8; search_range];
    let sr = if search_range / 2 > 1 {
        search_range / 2 - 1
    } else {
        0
    };

    // Count matched pairs within search range
    let mut num_com = 0i32;
    let yl1 = len2 as i32 - 1;
    for (i, &sc1) in s1.iter().enumerate() {
        let lowlim = 0i32.max(i as i32 - sr as i32) as usize;
        let hilim = (yl1.min(i as i32 + sr as i32) as usize + 1).min(len2);
        for j in lowlim..hilim {
            if s2_flag[j] == 0 && s2[j] == sc1 {
                s2_flag[j] = 1;
                s1_flag[i] = 1;
                num_com += 1;
                break;
            }
        }
    }

    if num_com == 0 {
        return 0.0;
    }

    // Count transpositions
    let mut k = 0usize;
    let mut n_trans = 0i32;
    for (i, &sc1) in s1.iter().enumerate() {
        if s1_flag[i] == 0 {
            continue;
        }
        for j in k..len2 {
            if s2_flag[j] != 0 {
                k = j + 1;
                if sc1 != s2[j] {
                    n_trans += 1;
                }
                break;
            }
        }
    }
    n_trans /= 2;

    // Adjust for similarities in unmatched characters
    let mut n_simi = 0i32;
    if minv > num_com as usize {
        for i in 0..len1 {
            if s1_flag[i] != 0 {
                continue;
            }
            let sc1 = s1[i] as u32;
            if sc1 == 0 || sc1 > 90 {
                continue;
            }
            for j in 0..len2 {
                if s2_flag[j] != 0 {
                    continue;
                }
                let sc2 = s2[j] as u32;
                if sc2 == 0 || sc2 > 90 {
                    continue;
                }
                let key = (s1[i].to_string(), s2[j].to_string());
                if let Some(&wt) = adjwt.get(&key) {
                    n_simi += wt as i32;
                    s2_flag[j] = 2;
                    break;
                }
            }
        }
    }
    let num_sim = n_simi as f64 / 10.0 + num_com as f64;

    // Main weight computation
    let mut weight = num_sim / len1 as f64 + num_sim / len2 as f64;
    weight += (num_com - n_trans) as f64 / num_com as f64;
    weight /= 3.0;

    if weight > 0.7 {
        // Boost for common prefix
        let j = minv.min(4);
        let mut i = 0usize;
        for (sc1, sc2) in s1.iter().zip(s2.iter()) {
            if i >= j {
                break;
            }
            if sc1 != sc2 {
                break;
            }
            if sc1.is_ascii_digit() {
                break;
            }
            i += 1;
        }
        if i > 0 {
            weight += i as f64 * 0.1 * (1.0 - weight);
        }
    }

    weight
}

// ── MLIPNS ──

pub struct MLIPNS {
    pub threshold: f64,
    pub max_mismatches: usize,
}

impl MLIPNS {
    pub fn new(threshold: f64, max_mismatches: usize) -> Self {
        MLIPNS {
            threshold,
            max_mismatches,
        }
    }
}

impl Default for MLIPNS {
    fn default() -> Self {
        MLIPNS {
            threshold: 0.25,
            max_mismatches: 2,
        }
    }
}

impl Algorithm for MLIPNS {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if let Some(result) = self.quick_answer(sequences) {
            return result;
        }
        if sequences.len() < 2 {
            return 0.0;
        }

        let mut mismatches = 0usize;
        let ham_start = hamming_distance(&sequences[0], &sequences[1]);
        let mut ham = ham_start;
        let mut maxlen = sequences.iter().map(|s| s.len()).max().unwrap_or(0);

        loop {
            if sequences.iter().any(|s| s.is_empty()) {
                return 1.0;
            }
            if mismatches > self.max_mismatches {
                break;
            }
            if maxlen == 0 {
                return 1.0;
            }
            if 1.0 - (maxlen as f64 - ham as f64) / maxlen as f64 <= self.threshold {
                return 1.0;
            }
            mismatches += 1;
            ham = ham.saturating_sub(1);
            maxlen = maxlen.saturating_sub(1);
        }

        if maxlen == 0 {
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

fn hamming_distance(s1: &[String], s2: &[String]) -> usize {
    let max_len = s1.len().max(s2.len());
    let min_len = s1.len().min(s2.len());
    let mut dist = max_len - min_len;
    for i in 0..min_len {
        if s1[i] != s2[i] {
            dist += 1;
        }
    }
    dist
}
