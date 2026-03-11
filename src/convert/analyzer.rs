use std::collections::BTreeMap;

use super::classifier::{SemanticCategory, classify_token};
use super::extractor::ExtractedToken;
use super::mapper::map_category;

pub struct AnalysisResult {
    pub rows: Vec<AnalysisRow>,
}

pub struct AnalysisRow {
    pub token: String,
    pub category: SemanticCategory,
    pub mapped: Option<String>,
}

pub fn analyze_tokens(tokens: &[ExtractedToken]) -> AnalysisResult {
    let mut unique = BTreeMap::<String, AnalysisRow>::new();

    for token in tokens {
        let category = classify_token(&token.normalized);
        let mapped = map_category(&category);
        unique.entry(token.normalized.clone()).or_insert(AnalysisRow {
            token: token.normalized.clone(),
            category,
            mapped,
        });
    }

    AnalysisResult {
        rows: unique.into_values().collect(),
    }
}

pub fn print_analysis(result: &AnalysisResult) {
    println!("Detected tokens:");
    println!();
    for row in &result.rows {
        println!("{}", row.token);
    }

    println!();
    println!("Suggested mappings:");
    println!();
    for row in &result.rows {
        match &row.mapped {
            Some(mapped) => println!("{} -> {}", row.token, short_name(mapped)),
            None => println!("{} -> <unknown>", row.token),
        }
    }
}

fn short_name(mapped: &str) -> String {
    mapped
        .trim_start_matches("{{")
        .trim_end_matches("}}")
        .to_string()
}
