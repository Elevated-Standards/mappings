// Modified: 2025-09-22

//! Text preprocessing utilities for fuzzy matching
//!
//! This module provides text preprocessing functionality to normalize strings
//! before fuzzy matching, including abbreviation expansion, case normalization,
//! and special character handling.

use std::collections::HashMap;
use unicode_normalization::{UnicodeNormalization, is_nfc};

/// Text preprocessor for normalizing strings before fuzzy matching
#[derive(Debug, Clone)]
pub struct TextPreprocessor {
    /// Common abbreviations and their expansions
    abbreviations: HashMap<String, String>,
    /// Stop words to remove
    stop_words: std::collections::HashSet<String>,
    /// Whether to normalize Unicode
    normalize_unicode: bool,
}

impl Default for TextPreprocessor {
    fn default() -> Self {
        let mut abbreviations = HashMap::new();
        
        // Common FedRAMP abbreviations
        abbreviations.insert("poc".to_string(), "point of contact".to_string());
        abbreviations.insert("poa&m".to_string(), "plan of action and milestones".to_string());
        abbreviations.insert("poam".to_string(), "plan of action and milestones".to_string());
        abbreviations.insert("ssp".to_string(), "system security plan".to_string());
        abbreviations.insert("ato".to_string(), "authority to operate".to_string());
        abbreviations.insert("csp".to_string(), "cloud service provider".to_string());
        abbreviations.insert("3pao".to_string(), "third party assessment organization".to_string());
        abbreviations.insert("jab".to_string(), "joint authorization board".to_string());
        abbreviations.insert("pmo".to_string(), "program management office".to_string());
        abbreviations.insert("isso".to_string(), "information system security officer".to_string());
        abbreviations.insert("isse".to_string(), "information system security engineer".to_string());
        abbreviations.insert("ca".to_string(), "certificate authority".to_string());
        abbreviations.insert("cac".to_string(), "common access card".to_string());
        abbreviations.insert("piv".to_string(), "personal identity verification".to_string());
        abbreviations.insert("fisma".to_string(), "federal information security management act".to_string());
        abbreviations.insert("nist".to_string(), "national institute of standards and technology".to_string());
        abbreviations.insert("sp".to_string(), "special publication".to_string());
        abbreviations.insert("rmf".to_string(), "risk management framework".to_string());
        abbreviations.insert("sca".to_string(), "supply chain assessment".to_string());
        abbreviations.insert("pen".to_string(), "penetration".to_string());
        abbreviations.insert("vuln".to_string(), "vulnerability".to_string());
        abbreviations.insert("cve".to_string(), "common vulnerabilities and exposures".to_string());
        abbreviations.insert("cvss".to_string(), "common vulnerability scoring system".to_string());
        abbreviations.insert("cwe".to_string(), "common weakness enumeration".to_string());
        abbreviations.insert("cpe".to_string(), "common platform enumeration".to_string());
        abbreviations.insert("ccb".to_string(), "configuration control board".to_string());
        abbreviations.insert("cm".to_string(), "configuration management".to_string());
        abbreviations.insert("ia".to_string(), "information assurance".to_string());
        abbreviations.insert("ir".to_string(), "incident response".to_string());
        abbreviations.insert("bc".to_string(), "business continuity".to_string());
        abbreviations.insert("dr".to_string(), "disaster recovery".to_string());
        abbreviations.insert("bcp".to_string(), "business continuity plan".to_string());
        abbreviations.insert("drp".to_string(), "disaster recovery plan".to_string());
        abbreviations.insert("rpo".to_string(), "recovery point objective".to_string());
        abbreviations.insert("rto".to_string(), "recovery time objective".to_string());
        abbreviations.insert("mtd".to_string(), "maximum tolerable downtime".to_string());
        abbreviations.insert("sla".to_string(), "service level agreement".to_string());
        abbreviations.insert("slo".to_string(), "service level objective".to_string());
        abbreviations.insert("kpi".to_string(), "key performance indicator".to_string());
        abbreviations.insert("kri".to_string(), "key risk indicator".to_string());
        
        let mut stop_words = std::collections::HashSet::new();
        stop_words.insert("the".to_string());
        stop_words.insert("and".to_string());
        stop_words.insert("or".to_string());
        stop_words.insert("of".to_string());
        stop_words.insert("in".to_string());
        stop_words.insert("on".to_string());
        stop_words.insert("at".to_string());
        stop_words.insert("to".to_string());
        stop_words.insert("for".to_string());
        stop_words.insert("with".to_string());
        stop_words.insert("by".to_string());
        stop_words.insert("from".to_string());
        stop_words.insert("as".to_string());
        stop_words.insert("is".to_string());
        stop_words.insert("are".to_string());
        stop_words.insert("was".to_string());
        stop_words.insert("were".to_string());
        stop_words.insert("be".to_string());
        stop_words.insert("been".to_string());
        stop_words.insert("have".to_string());
        stop_words.insert("has".to_string());
        stop_words.insert("had".to_string());
        stop_words.insert("will".to_string());
        stop_words.insert("would".to_string());
        stop_words.insert("could".to_string());
        stop_words.insert("should".to_string());
        stop_words.insert("may".to_string());
        stop_words.insert("might".to_string());
        stop_words.insert("can".to_string());
        stop_words.insert("must".to_string());
        stop_words.insert("shall".to_string());
        
        Self {
            abbreviations,
            stop_words,
            normalize_unicode: true,
        }
    }
}

impl TextPreprocessor {
    /// Create a new preprocessor with custom settings
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a custom abbreviation
    pub fn add_abbreviation(&mut self, abbrev: String, expansion: String) {
        self.abbreviations.insert(abbrev.to_lowercase(), expansion.to_lowercase());
    }
    
    /// Add a stop word
    pub fn add_stop_word(&mut self, word: String) {
        self.stop_words.insert(word.to_lowercase());
    }
    
    /// Preprocess a string for fuzzy matching
    pub fn preprocess(&self, text: &str) -> (String, Vec<String>) {
        let mut processed = text.to_string();
        let mut steps = Vec::new();
        
        // Unicode normalization
        if self.normalize_unicode && !is_nfc(text) {
            processed = processed.nfc().collect();
            steps.push("unicode_normalization".to_string());
        }
        
        // Convert to lowercase
        processed = processed.to_lowercase();
        steps.push("lowercase".to_string());
        
        // Remove special characters and normalize whitespace
        processed = processed
            .chars()
            .map(|c| if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");
        steps.push("normalize_chars".to_string());
        
        // Expand abbreviations
        let original_len = processed.len();
        for (abbrev, expansion) in &self.abbreviations {
            if processed.contains(abbrev) {
                processed = processed.replace(abbrev, expansion);
            }
        }
        if processed.len() != original_len {
            steps.push("expand_abbreviations".to_string());
        }
        
        // Remove stop words
        let words: Vec<&str> = processed.split_whitespace().collect();
        let filtered_words: Vec<&str> = words
            .into_iter()
            .filter(|word| !self.stop_words.contains(&word.to_lowercase()))
            .collect();
        
        if filtered_words.len() != processed.split_whitespace().count() {
            processed = filtered_words.join(" ");
            steps.push("remove_stop_words".to_string());
        }
        
        // Final whitespace normalization
        processed = processed.trim().to_string();
        
        (processed, steps)
    }
    
    /// Quick preprocessing for performance-critical operations
    pub fn quick_preprocess(&self, text: &str) -> String {
        text.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }
    
    /// Check if two strings are equivalent after preprocessing
    pub fn are_equivalent(&self, s1: &str, s2: &str) -> bool {
        let (processed1, _) = self.preprocess(s1);
        let (processed2, _) = self.preprocess(s2);
        processed1 == processed2
    }
}
