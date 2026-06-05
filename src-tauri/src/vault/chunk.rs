//! Token-aware chunking for markdown notes. We approximate tokens as
//! whitespace-separated words; sqlite-vec embeddings tolerate small drift.

const CHUNK_TOKENS: usize = 400;
const OVERLAP_TOKENS: usize = 50;

pub fn chunk_text(s: &str) -> Vec<String> {
    let words: Vec<&str> = s.split_whitespace().collect();
    if words.is_empty() {
        return vec![];
    }
    if words.len() <= CHUNK_TOKENS {
        return vec![words.join(" ")];
    }
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < words.len() {
        let end = (start + CHUNK_TOKENS).min(words.len());
        chunks.push(words[start..end].join(" "));
        if end == words.len() {
            break;
        }
        start = end - OVERLAP_TOKENS;
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_text_one_chunk() {
        let chunks = chunk_text("hello world from argus");
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn long_text_overlaps() {
        let s = (0..1500).map(|i| i.to_string()).collect::<Vec<_>>().join(" ");
        let chunks = chunk_text(&s);
        assert!(chunks.len() >= 4);
        // Verify overlap exists between successive chunks.
        for w in chunks.windows(2) {
            let last_words: Vec<&str> = w[0].split_whitespace().rev().take(OVERLAP_TOKENS).collect();
            let next_first: Vec<&str> = w[1].split_whitespace().take(OVERLAP_TOKENS).collect();
            // The two sets should share words (forward order vs reverse → at least intersect).
            let a: std::collections::HashSet<_> = last_words.into_iter().collect();
            let b: std::collections::HashSet<_> = next_first.into_iter().collect();
            assert!(!a.is_disjoint(&b));
        }
    }
}
