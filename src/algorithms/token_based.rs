use crate::Algorithm;

// ── Jaccard ──

pub struct Jaccard {
    pub qval: Option<usize>,
    pub as_set: bool,
}

impl Jaccard {
    pub fn new(qval: Option<usize>, as_set: bool) -> Self {
        Jaccard { qval, as_set }
    }
}

impl Default for Jaccard {
    fn default() -> Self {
        Jaccard {
            qval: Some(1),
            as_set: false,
        }
    }
}

impl Algorithm for Jaccard {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if let Some(result) = self.quick_answer(sequences) {
            return result;
        }
        let prepared = self.prepare(sequences, self.qval);
        let counters = self.to_counters(&prepared);
        let intersection = self.intersect_counters(&counters);
        let intersection_count = self.count_counter(&intersection, self.as_set);
        let union = self.union_counters(&counters);
        let union_count = self.count_counter(&union, self.as_set);
        intersection_count as f64 / union_count as f64
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        1.0
    }

    fn is_similarity(&self) -> bool {
        true
    }
}

// ── Sorensen (Dice) ──

pub struct Sorensen {
    pub qval: Option<usize>,
    pub as_set: bool,
}

impl Sorensen {
    pub fn new(qval: Option<usize>, as_set: bool) -> Self {
        Sorensen { qval, as_set }
    }
}

impl Default for Sorensen {
    fn default() -> Self {
        Sorensen {
            qval: Some(1),
            as_set: false,
        }
    }
}

impl Algorithm for Sorensen {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if let Some(result) = self.quick_answer(sequences) {
            return result;
        }
        let prepared = self.prepare(sequences, self.qval);
        let counters = self.to_counters(&prepared);
        let total: usize = counters
            .iter()
            .map(|c| self.count_counter(c, self.as_set))
            .sum();
        let intersection = self.intersect_counters(&counters);
        let intersection_count = self.count_counter(&intersection, self.as_set);
        2.0 * intersection_count as f64 / total as f64
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        1.0
    }

    fn is_similarity(&self) -> bool {
        true
    }
}

// ── Tversky ──

pub struct Tversky {
    pub qval: Option<usize>,
    pub ks: Vec<f64>,
    pub bias: Option<f64>,
    pub as_set: bool,
}

impl Tversky {
    pub fn new(qval: Option<usize>, ks: Vec<f64>, bias: Option<f64>, as_set: bool) -> Self {
        Tversky {
            qval,
            ks,
            bias,
            as_set,
        }
    }
}

impl Default for Tversky {
    fn default() -> Self {
        Tversky {
            qval: Some(1),
            ks: vec![1.0],
            bias: None,
            as_set: false,
        }
    }
}

impl Algorithm for Tversky {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if let Some(result) = self.quick_answer(sequences) {
            return result;
        }
        let prepared = self.prepare(sequences, self.qval);
        let counters = self.to_counters(&prepared);
        let intersection = self.intersect_counters(&counters);
        let intersection_count = self.count_counter(&intersection, self.as_set) as f64;
        let seq_counts: Vec<f64> = counters
            .iter()
            .map(|c| self.count_counter(c, self.as_set) as f64)
            .collect();

        if seq_counts.len() != 2 || self.bias.is_none() {
            let mut result = intersection_count;
            let ks: Vec<f64> = self
                .ks
                .iter()
                .cycle()
                .take(seq_counts.len())
                .copied()
                .collect();
            for (k, &s) in ks.iter().zip(&seq_counts) {
                result += k * (s - intersection_count);
            }
            return intersection_count / result;
        }

        // 2-sequence with bias
        let alpha = *self.ks.first().unwrap_or(&1.0);
        let beta = *self.ks.get(1).unwrap_or(&1.0);
        let a_val = seq_counts[0].min(seq_counts[1]);
        let b_val = seq_counts[0].max(seq_counts[1]);
        let c_val = intersection_count + self.bias.unwrap_or(0.0);
        let result = alpha * beta * (a_val - b_val) + b_val * beta;
        c_val / (result + c_val)
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        1.0
    }

    fn is_similarity(&self) -> bool {
        true
    }
}

// ── Overlap ──

pub struct Overlap {
    pub qval: Option<usize>,
    pub as_set: bool,
}

impl Overlap {
    pub fn new(qval: Option<usize>, as_set: bool) -> Self {
        Overlap { qval, as_set }
    }
}

impl Default for Overlap {
    fn default() -> Self {
        Overlap {
            qval: Some(1),
            as_set: false,
        }
    }
}

impl Algorithm for Overlap {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if let Some(result) = self.quick_answer(sequences) {
            return result;
        }
        let prepared = self.prepare(sequences, self.qval);
        let counters = self.to_counters(&prepared);
        let intersection = self.intersect_counters(&counters);
        let intersection_count = self.count_counter(&intersection, self.as_set);
        let min_count = counters
            .iter()
            .map(|c| self.count_counter(c, self.as_set))
            .min()
            .unwrap_or(1);
        intersection_count as f64 / min_count as f64
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        1.0
    }

    fn is_similarity(&self) -> bool {
        true
    }
}

// ── Cosine ──

pub struct Cosine {
    pub qval: Option<usize>,
    pub as_set: bool,
}

impl Cosine {
    pub fn new(qval: Option<usize>, as_set: bool) -> Self {
        Cosine { qval, as_set }
    }
}

impl Default for Cosine {
    fn default() -> Self {
        Cosine {
            qval: Some(1),
            as_set: false,
        }
    }
}

impl Algorithm for Cosine {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        if let Some(result) = self.quick_answer(sequences) {
            return result;
        }
        let prepared = self.prepare(sequences, self.qval);
        let counters = self.to_counters(&prepared);
        let intersection = self.intersect_counters(&counters);
        let intersection_count = self.count_counter(&intersection, self.as_set) as f64;
        let seq_counts: Vec<f64> = counters
            .iter()
            .map(|c| self.count_counter(c, self.as_set) as f64)
            .collect();
        let prod: f64 = seq_counts.iter().product();
        intersection_count / prod.powf(1.0 / seq_counts.len() as f64)
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        1.0
    }

    fn is_similarity(&self) -> bool {
        true
    }
}

// ── Tanimoto ──

pub struct Tanimoto {
    pub qval: Option<usize>,
    pub as_set: bool,
}

impl Tanimoto {
    pub fn new(qval: Option<usize>, as_set: bool) -> Self {
        Tanimoto { qval, as_set }
    }
}

impl Default for Tanimoto {
    fn default() -> Self {
        Tanimoto {
            qval: Some(1),
            as_set: false,
        }
    }
}

impl Algorithm for Tanimoto {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        let jaccard = Jaccard::new(self.qval, self.as_set);
        let result = jaccard.compute(sequences);
        if result == 0.0 {
            f64::NEG_INFINITY
        } else {
            result.log2()
        }
    }

    fn maximum(&self, _sequences: &[Vec<String>]) -> f64 {
        1.0
    }

    fn is_similarity(&self) -> bool {
        true
    }
}

// ── Bag ──

pub struct Bag;

impl Algorithm for Bag {
    fn compute(&self, sequences: &[Vec<String>]) -> f64 {
        let counters = self.to_counters(sequences);
        let intersection = self.intersect_counters(&counters);
        let max_diff = counters
            .iter()
            .map(|c| {
                let mut diff = c.clone();
                for (k, v) in &intersection {
                    if let Some(count) = diff.get_mut(k) {
                        *count = count.saturating_sub(*v);
                    }
                }
                diff.values().sum::<usize>()
            })
            .max()
            .unwrap_or(0);
        max_diff as f64
    }

    fn is_similarity(&self) -> bool {
        false
    }
}
