// Modified: 2025-09-22

//! Fuzzy matching algorithms implementation
//!
//! This module contains implementations of various fuzzy string matching algorithms
//! including Levenshtein distance, Jaro-Winkler, N-gram similarity, and Soundex.

use std::collections::HashMap;
use super::types::FuzzyAlgorithm;

/// Levenshtein distance algorithm implementation
#[derive(Debug, Clone)]
pub struct LevenshteinAlgorithm;

impl FuzzyAlgorithm for LevenshteinAlgorithm {
    fn name(&self) -> &'static str {
        "levenshtein"
    }

    fn similarity(&self, s1: &str, s2: &str) -> f64 {
        if s1 == s2 {
            return 1.0;
        }
        
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();
        
        if len1 == 0 || len2 == 0 {
            return 0.0;
        }
        
        let distance = self.levenshtein_distance(s1, s2);
        let max_len = len1.max(len2) as f64;
        
        1.0 - (distance as f64 / max_len)
    }
}

impl LevenshteinAlgorithm {
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        let len1 = chars1.len();
        let len2 = chars2.len();
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }
        
        matrix[len1][len2]
    }
}

/// Jaro-Winkler algorithm implementation
#[derive(Debug, Clone)]
pub struct JaroWinklerAlgorithm;

impl FuzzyAlgorithm for JaroWinklerAlgorithm {
    fn name(&self) -> &'static str {
        "jaro_winkler"
    }

    fn similarity(&self, s1: &str, s2: &str) -> f64 {
        self.jaro_winkler_similarity(s1, s2)
    }
}

impl JaroWinklerAlgorithm {
    fn jaro_winkler_similarity(&self, s1: &str, s2: &str) -> f64 {
        if s1 == s2 {
            return 1.0;
        }
        
        let jaro = self.jaro_similarity(s1, s2);
        if jaro < 0.7 {
            return jaro;
        }
        
        let prefix_len = s1.chars()
            .zip(s2.chars())
            .take(4)
            .take_while(|(c1, c2)| c1 == c2)
            .count();
        
        jaro + (0.1 * prefix_len as f64 * (1.0 - jaro))
    }
    
    fn jaro_similarity(&self, s1: &str, s2: &str) -> f64 {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        let len1 = chars1.len();
        let len2 = chars2.len();
        
        if len1 == 0 && len2 == 0 {
            return 1.0;
        }
        if len1 == 0 || len2 == 0 {
            return 0.0;
        }
        
        let match_window = (len1.max(len2) / 2).saturating_sub(1);
        let mut s1_matches = vec![false; len1];
        let mut s2_matches = vec![false; len2];
        
        let mut matches = 0;
        
        // Find matches
        for i in 0..len1 {
            let start = i.saturating_sub(match_window);
            let end = (i + match_window + 1).min(len2);
            
            for j in start..end {
                if s2_matches[j] || chars1[i] != chars2[j] {
                    continue;
                }
                s1_matches[i] = true;
                s2_matches[j] = true;
                matches += 1;
                break;
            }
        }
        
        if matches == 0 {
            return 0.0;
        }
        
        // Count transpositions
        let mut transpositions = 0;
        let mut k = 0;
        for i in 0..len1 {
            if !s1_matches[i] {
                continue;
            }
            while !s2_matches[k] {
                k += 1;
            }
            if chars1[i] != chars2[k] {
                transpositions += 1;
            }
            k += 1;
        }
        
        let jaro = (matches as f64 / len1 as f64 + 
                   matches as f64 / len2 as f64 + 
                   (matches as f64 - transpositions as f64 / 2.0) / matches as f64) / 3.0;
        
        jaro
    }
}

/// N-gram similarity algorithm implementation
#[derive(Debug, Clone)]
pub struct NgramAlgorithm {
    n: usize,
}

impl NgramAlgorithm {
    pub fn new(n: usize) -> Self {
        Self { n }
    }
    
    fn generate_ngrams(&self, s: &str) -> HashMap<String, usize> {
        let chars: Vec<char> = s.chars().collect();
        let mut ngrams = HashMap::new();
        
        if chars.len() < self.n {
            ngrams.insert(s.to_string(), 1);
            return ngrams;
        }
        
        for i in 0..=chars.len() - self.n {
            let ngram: String = chars[i..i + self.n].iter().collect();
            *ngrams.entry(ngram).or_insert(0) += 1;
        }
        
        ngrams
    }
}

impl Default for NgramAlgorithm {
    fn default() -> Self {
        Self::new(2) // Bigrams by default
    }
}

impl FuzzyAlgorithm for NgramAlgorithm {
    fn name(&self) -> &'static str {
        "ngram"
    }

    fn similarity(&self, s1: &str, s2: &str) -> f64 {
        if s1 == s2 {
            return 1.0;
        }
        
        let ngrams1 = self.generate_ngrams(s1);
        let ngrams2 = self.generate_ngrams(s2);
        
        if ngrams1.is_empty() && ngrams2.is_empty() {
            return 1.0;
        }
        if ngrams1.is_empty() || ngrams2.is_empty() {
            return 0.0;
        }
        
        let mut intersection = 0;
        let mut union = 0;
        
        let all_ngrams: std::collections::HashSet<_> = ngrams1.keys()
            .chain(ngrams2.keys())
            .collect();
        
        for ngram in all_ngrams {
            let count1 = ngrams1.get(ngram).unwrap_or(&0);
            let count2 = ngrams2.get(ngram).unwrap_or(&0);
            intersection += count1.min(count2);
            union += count1.max(count2);
        }
        
        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
}

/// Soundex algorithm implementation
#[derive(Debug, Clone)]
pub struct SoundexAlgorithm;

impl SoundexAlgorithm {
    fn soundex(&self, s: &str) -> String {
        if s.is_empty() {
            return "0000".to_string();
        }
        
        let chars: Vec<char> = s.to_uppercase().chars().collect();
        let mut result = String::new();
        
        // First character
        result.push(chars[0]);
        
        let mut prev_code = self.get_soundex_code(chars[0]);
        
        for &ch in &chars[1..] {
            let code = self.get_soundex_code(ch);
            if code != '0' && code != prev_code {
                result.push(code);
                if result.len() == 4 {
                    break;
                }
            }
            if code != '0' {
                prev_code = code;
            }
        }
        
        // Pad with zeros
        while result.len() < 4 {
            result.push('0');
        }
        
        result
    }
    
    fn get_soundex_code(&self, ch: char) -> char {
        match ch {
            'B' | 'F' | 'P' | 'V' => '1',
            'C' | 'G' | 'J' | 'K' | 'Q' | 'S' | 'X' | 'Z' => '2',
            'D' | 'T' => '3',
            'L' => '4',
            'M' | 'N' => '5',
            'R' => '6',
            _ => '0',
        }
    }
}

impl FuzzyAlgorithm for SoundexAlgorithm {
    fn name(&self) -> &'static str {
        "soundex"
    }

    fn similarity(&self, s1: &str, s2: &str) -> f64 {
        if s1 == s2 {
            return 1.0;
        }
        
        let soundex1 = self.soundex(s1);
        let soundex2 = self.soundex(s2);
        
        if soundex1 == soundex2 {
            0.8 // High similarity for same soundex, but not perfect
        } else {
            // Calculate partial similarity based on matching positions
            let matches = soundex1.chars()
                .zip(soundex2.chars())
                .filter(|(c1, c2)| c1 == c2)
                .count();
            
            matches as f64 / 4.0 * 0.6 // Scale down since soundex is less precise
        }
    }
    
    fn needs_preprocessing(&self) -> bool {
        false // Soundex handles its own normalization
    }
}
