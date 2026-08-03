use crate::Algorithm;
use std::collections::HashMap;

// ── MRA (Match Rating Approach) ──

pub struct MRA;

impl Algorithm for MRA {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if !sequences.iter().all(|s| !s.is_empty()) {
            return 0.0;
        }
        let words: Vec<String> = sequences.iter().map(|s| s.join("")).collect();
        let coded: Vec<Vec<char>> = words.iter().map(|w| self.calc_mra(w)).collect();
        let lengths: Vec<usize> = coded.iter().map(|c| c.len()).collect();
        let max_length = *lengths.iter().max().unwrap_or(&0);
        let min_length = *lengths.iter().min().unwrap_or(&0);
        let count = lengths.len();

        if max_length.abs_diff(min_length) > count {
            return 0.0;
        }

        let mut sequences = coded;
        let mut lengths = lengths;

        for _ in 0..count {
            let minlen = *lengths.iter().min().unwrap_or(&0);
            let mut new_sequences: Vec<Vec<char>> = Vec::new();

            for i in 0..minlen {
                let chars: Vec<char> = sequences.iter().map(|s| s[i]).collect();
                if !chars.windows(2).all(|w| w[0] == w[1]) {
                    for (si, s) in sequences.iter().enumerate() {
                        if new_sequences.len() <= si {
                            new_sequences.push(Vec::new());
                        }
                        new_sequences[si].push(s[i]);
                    }
                }
            }

            let mut updated = Vec::new();
            for (si, s) in sequences.iter().enumerate() {
                let mut combined;
                if si < new_sequences.len() {
                    combined = new_sequences[si].clone();
                } else {
                    combined = Vec::new();
                }
                combined.extend_from_slice(&s[minlen..]);
                updated.push(combined);
            }
            sequences = updated;
            lengths = sequences.iter().map(|s| s.len()).collect();
        }

        if lengths.is_empty() {
            return max_length as f64;
        }
        (max_length - *lengths.iter().max().unwrap_or(&0)) as f64
    }

    fn maximum(&self, sequences: &[Vec<String>]) -> f64 {
        let words: Vec<String> = sequences.iter().map(|s| s.join("")).collect();
        let coded: Vec<Vec<char>> = words.iter().map(|w| self.calc_mra(w)).collect();
        coded.iter().map(|c| c.len()).max().unwrap_or(0) as f64
    }

    fn is_similarity(&self) -> bool {
        true
    }
}

impl MRA {
    fn calc_mra(&self, word: &str) -> Vec<char> {
        if word.is_empty() {
            return vec![];
        }
        let upper: Vec<char> = word.to_uppercase().chars().collect();
        let first = upper[0];
        // Remove vowels after first char
        let vowels = ['A', 'E', 'I', 'O', 'U'];
        let no_vowels: Vec<char> = std::iter::once(first)
            .chain(upper[1..].iter().filter(|c| !vowels.contains(c)).copied())
            .collect();
        // Remove consecutive duplicates
        let mut uniq: Vec<char> = Vec::new();
        for &c in &no_vowels {
            if uniq.last() != Some(&c) {
                uniq.push(c);
            }
        }
        // Truncate to 6 chars: first 3 + last 3
        if uniq.len() > 6 {
            let mut result = uniq[..3].to_vec();
            result.extend_from_slice(&uniq[uniq.len() - 3..]);
            result
        } else {
            uniq
        }
    }
}

// ── Editex ──

pub struct Editex {
    pub local: bool,
    pub match_cost: usize,
    pub group_cost: usize,
    pub mismatch_cost: usize,
}

impl Editex {
    pub fn new(local: bool, match_cost: usize, group_cost: usize, mismatch_cost: usize) -> Self {
        let group_cost = group_cost.max(match_cost);
        let mismatch_cost = mismatch_cost.max(group_cost);
        Editex {
            local,
            match_cost,
            group_cost,
            mismatch_cost,
        }
    }

    fn r_cost(&self, a: &str, b: &str, groups: &[Vec<char>]) -> usize {
        if a == b {
            return self.match_cost;
        }
        let a_char = a.chars().next().unwrap_or('\0');
        let b_char = b.chars().next().unwrap_or('\0');

        for group in groups {
            if group.contains(&a_char) && group.contains(&b_char) {
                return self.group_cost;
            }
        }
        self.mismatch_cost
    }

    fn d_cost(&self, a: &str, b: &str, groups: &[Vec<char>], ungrouped: &[char]) -> usize {
        if a != b && ungrouped.contains(&a.chars().next().unwrap_or('\0')) {
            return self.group_cost;
        }
        self.r_cost(a, b, groups)
    }
}

impl Default for Editex {
    fn default() -> Self {
        Editex {
            local: false,
            match_cost: 0,
            group_cost: 1,
            mismatch_cost: 2,
        }
    }
}

impl Algorithm for Editex {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if let Some(result) = self.quick_answer(sequences) {
            return result;
        }
        let groups: Vec<Vec<char>> = vec![
            vec!['A', 'E', 'I', 'O', 'U', 'Y'],
            vec!['B', 'P'],
            vec!['C', 'K', 'Q'],
            vec!['D', 'T'],
            vec!['L', 'R'],
            vec!['M', 'N'],
            vec!['G', 'J'],
            vec!['F', 'P', 'V'],
            vec!['S', 'X', 'Z'],
            vec!['C', 'S', 'Z'],
        ];
        let ungrouped: Vec<char> = vec!['H', 'W'];

        let s1_str = sequences[0].join("");
        let s2_str = sequences[1].join("");

        let max_length = self.maximum(sequences) as usize;
        let s1 = " ".to_string() + &s1_str.to_uppercase();
        let s2 = " ".to_string() + &s2_str.to_uppercase();
        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();
        let len_s1 = s1_chars.len() - 1;
        let len_s2 = s2_chars.len() - 1;

        let mut d_mat: HashMap<(usize, usize), usize> = HashMap::new();

        if !self.local {
            for i in 1..=len_s1 {
                let prev = *d_mat.get(&(i - 1, 0)).unwrap_or(&0);
                let cost = self.d_cost(
                    &s1_chars[i - 1].to_string(),
                    &s1_chars[i].to_string(),
                    &groups,
                    &ungrouped,
                );
                d_mat.insert((i, 0), prev + cost);
            }
        }
        for j in 1..=len_s2 {
            let prev = *d_mat.get(&(0, j - 1)).unwrap_or(&0);
            let cost = self.d_cost(
                &s2_chars[j - 1].to_string(),
                &s2_chars[j].to_string(),
                &groups,
                &ungrouped,
            );
            d_mat.insert((0, j), prev + cost);
        }

        for i in 1..=len_s1 {
            for j in 1..=len_s2 {
                let delete = *d_mat.get(&(i - 1, j)).unwrap_or(&0)
                    + self.d_cost(
                        &s1_chars[i - 1].to_string(),
                        &s1_chars[i].to_string(),
                        &groups,
                        &ungrouped,
                    );
                let insert = *d_mat.get(&(i, j - 1)).unwrap_or(&0)
                    + self.d_cost(
                        &s2_chars[j - 1].to_string(),
                        &s2_chars[j].to_string(),
                        &groups,
                        &ungrouped,
                    );
                let replace = *d_mat.get(&(i - 1, j - 1)).unwrap_or(&0)
                    + self.r_cost(&s1_chars[i].to_string(), &s2_chars[j].to_string(), &groups);
                let val = delete.min(insert).min(replace);
                d_mat.insert((i, j), val);
            }
        }

        let distance = *d_mat.get(&(len_s1, len_s2)).unwrap_or(&0);
        distance.min(max_length) as f64
    }

    fn maximum(&self, sequences: &[Vec<String>]) -> f64 {
        sequences.iter().map(|s| s.len()).max().unwrap_or(0) as f64 * self.mismatch_cost as f64
    }

    fn is_similarity(&self) -> bool {
        false
    }
}
