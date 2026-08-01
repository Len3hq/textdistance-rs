use crate::utils;
use crate::Algorithm;

// ── LCSSeq (Longest Common Subsequence) ──

pub struct LCSSeq;

impl Algorithm for LCSSeq {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if sequences.is_empty() {
            return 0.0;
        }
        if sequences.len() == 1 {
            return sequences[0].len() as f64;
        }
        // Use dynamic programming for 2 sequences
        let lcs_len = self.lcs_dynamic(&sequences[0], &sequences[1]);
        lcs_len as f64
    }

    fn is_similarity(&self) -> bool {
        true
    }
}

impl LCSSeq {
    fn lcs_dynamic(&self, s1: &[String], s2: &[String]) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        let mut lengths = vec![vec![0usize; len2 + 1]; len1 + 1];

        for i in 1..=len1 {
            for j in 1..=len2 {
                if s1[i - 1] == s2[j - 1] {
                    lengths[i][j] = lengths[i - 1][j - 1] + 1;
                } else {
                    lengths[i][j] = lengths[i - 1][j].max(lengths[i][j - 1]);
                }
            }
        }
        lengths[len1][len2]
    }
}

// ── LCSStr (Longest Common Substring) ──

pub struct LCSStr;

impl Algorithm for LCSStr {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if sequences.is_empty() {
            return 0.0;
        }
        if sequences.len() == 1 {
            return sequences[0].len() as f64;
        }
        if !sequences.iter().all(|s| !s.is_empty()) {
            return 0.0;
        }

        // For 2 sequences under 200 chars, use standard DP
        let max_len = sequences.iter().map(|s| s.len()).max().unwrap_or(0);
        if sequences.len() == 2 && max_len < 200 {
            return self.lcs_dynamic(&sequences[0], &sequences[1]) as f64;
        }

        // Custom: sliding window from shortest
        self.lcs_custom(sequences) as f64
    }

    fn is_similarity(&self) -> bool {
        true
    }
}

impl LCSStr {
    fn lcs_dynamic(&self, s1: &[String], s2: &[String]) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        let mut lengths = vec![vec![0usize; len2 + 1]; len1 + 1];
        let mut max_len = 0;

        for i in 1..=len1 {
            for j in 1..=len2 {
                if s1[i - 1] == s2[j - 1] {
                    lengths[i][j] = lengths[i - 1][j - 1] + 1;
                    if lengths[i][j] > max_len {
                        max_len = lengths[i][j];
                    }
                }
            }
        }
        max_len
    }

    fn lcs_custom(&self, sequences: &[Vec<String>]) -> usize {
        // Find shortest sequence
        let short = sequences.iter().min_by_key(|s| s.len()).unwrap();
        let short_len = short.len();

        for n in (1..=short_len).rev() {
            let ngrams = utils::find_ngrams_from_vec(short, n);
            for ngram in &ngrams {
                let ngram_chars: Vec<String> = ngram.chars().map(|c| c.to_string()).collect();
                let found = sequences.iter().all(|seq| {
                    seq.windows(ngram_chars.len())
                        .any(|w| w == ngram_chars.as_slice())
                });
                if found {
                    return ngram.len();
                }
            }
        }
        0
    }
}

// ── RatcliffObershelp ──

pub struct RatcliffObershelp;

impl Algorithm for RatcliffObershelp {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if let Some(result) = self.quick_answer(sequences) {
            return result;
        }
        let scount = sequences.len() as f64;
        let ecount: usize = sequences.iter().map(|s| s.len()).sum();
        let matching = self.find_matching(sequences);
        scount * matching as f64 / ecount as f64
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        1.0
    }

    fn is_similarity(&self) -> bool {
        true
    }
}

impl RatcliffObershelp {
    /// Find longest common substring and recurse on left and right parts.
    fn find_matching(&self, sequences: &[Vec<String>]) -> usize {
        if sequences.is_empty() {
            return 0;
        }

        // Find longest common substring using LCSStr
        let lcs = LCSStr;
        let subseq_len = lcs.compute(sequences) as usize;
        if subseq_len == 0 {
            return 0;
        }

        // Reconstruct the actual substring
        let s1 = &sequences[0];
        let s1_str: String = s1.join("");
        let s2 = &sequences[1];
        let s2_str: String = s2.join("");

        let lcs_str = self.find_lcs_str(s1, s2, subseq_len);

        // Split on the first occurrence
        let pos1 = s1_str.find(&lcs_str).unwrap_or(0);
        let pos2 = s2_str.find(&lcs_str).unwrap_or(0);

        let before1: Vec<String> = s1_str[..pos1].chars().map(|c| c.to_string()).collect();
        let before2: Vec<String> = s2_str[..pos2].chars().map(|c| c.to_string()).collect();
        let after1: Vec<String> = s1_str[pos1 + lcs_str.len()..]
            .chars()
            .map(|c| c.to_string())
            .collect();
        let after2: Vec<String> = s2_str[pos2 + lcs_str.len()..]
            .chars()
            .map(|c| c.to_string())
            .collect();

        subseq_len + self.find_matching(&[before1, before2]) + self.find_matching(&[after1, after2])
    }

    fn find_lcs_str(&self, s1: &[String], s2: &[String], _len: usize) -> String {
        let len1 = s1.len();
        let len2 = s2.len();
        let mut lengths = vec![vec![0usize; len2 + 1]; len1 + 1];
        let mut max_len = 0;
        let mut end_pos = 0;

        for i in 1..=len1 {
            for j in 1..=len2 {
                if s1[i - 1] == s2[j - 1] {
                    lengths[i][j] = lengths[i - 1][j - 1] + 1;
                    if lengths[i][j] > max_len {
                        max_len = lengths[i][j];
                        end_pos = i;
                    }
                }
            }
        }

        s1[end_pos - max_len..end_pos].join("")
    }
}
