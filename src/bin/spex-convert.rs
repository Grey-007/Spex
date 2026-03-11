use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

use clap::Parser;
use spex::convert::analyzer::{analyze_tokens, print_analysis};
use spex::convert::classifier::classify_token;
use spex::convert::detect::{TemplateSystem, detect_template_system};
use spex::convert::extractor::extract_tokens;
use spex::convert::mapper::map_category;
use spex::convert::rewriter::rewrite_template;

#[derive(Debug, Parser)]
#[command(name = "spex-convert", version, about = "Convert external templates to spex syntax")]
struct Cli {
    #[arg(value_name = "TEMPLATE_FILE")]
    template_file: PathBuf,

    #[arg(long, action = clap::ArgAction::SetTrue)]
    analyze: bool,

    #[arg(long, value_name = "FILE")]
    output: Option<PathBuf>,

    #[arg(long, action = clap::ArgAction::SetTrue)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    if !cli.template_file.exists() {
        return Err(format!("Unreadable file: {}", cli.template_file.display()).into());
    }

    let input = std::fs::read_to_string(&cli.template_file).map_err(|err| {
        format!(
            "Unreadable file: {} ({err})",
            cli.template_file.display()
        )
    })?;

    let extracted = extract_tokens(&input).map_err(|err| format!("Invalid syntax: {err}"))?;
    if extracted.is_empty() {
        println!("No tokens found in template.");
        return Ok(());
    }

    let detected = detect_template_system(&extracted);
    if cli.verbose {
        println!("Detected template system: {}", system_name(detected));
    }

    if cli.analyze {
        let analysis = analyze_tokens(&extracted);
        print_analysis(&analysis);
        return Ok(());
    }

    let mut mappings = HashMap::<String, String>::new();
    let mut unknown = Vec::new();
    for token in &extracted {
        if mappings.contains_key(&token.normalized) {
            continue;
        }

        let category = classify_token(&token.normalized);
        if let Some(mapped) = map_category(&category) {
            mappings.insert(token.normalized.clone(), mapped);
        } else {
            unknown.push(token.normalized.clone());
        }
    }

    if !unknown.is_empty() {
        println!("Unknown token(s) detected:");
        for token in unknown {
            println!("- {token}");
        }
    }

    let output = rewrite_template(&input, &mappings, &extracted);
    let output_path = cli
        .output
        .unwrap_or_else(|| PathBuf::from("converted_template.spex"));
    std::fs::write(&output_path, output)?;
    println!("Converted template written to: {}", output_path.display());

    Ok(())
}

fn system_name(system: TemplateSystem) -> &'static str {
    match system {
        TemplateSystem::Pywal => "pywal",
        TemplateSystem::Matugen => "matugen",
        TemplateSystem::CssVariable => "css-variable",
        TemplateSystem::Unknown => "unknown",
    }
}
