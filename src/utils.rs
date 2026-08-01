pub fn find_ngrams(text: &str, n: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < n {
        return vec![text.to_string()];
    }
    chars.windows(n).map(|w| w.iter().collect()).collect()
}

pub fn find_ngrams_from_vec(items: &[String], n: usize) -> Vec<String> {
    if items.len() < n {
        return items.to_vec();
    }
    items.windows(n).map(|w| w.join("")).collect()
}
