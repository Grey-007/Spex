pub mod config_check;
pub mod engine_check;
pub mod hook_check;
pub mod template_check;

use std::error::Error;
use std::path::Path;

use crate::template::config::TemplateConfig;

pub fn run_doctor(config_override: Option<&Path>) -> Result<bool, Box<dyn Error>> {
    println!("Running spex diagnostics...");
    println!();

    let mut issues = 0usize;

    let config_result = config_check::check_config(config_override)?;
    issues += config_result.issues;

    let mut loaded_config: Option<TemplateConfig> = config_result.config;

    if let Some(config) = loaded_config.take() {
        let template_result = template_check::check_templates(&config)?;
        issues += template_result.issues;

        let hook_issues = hook_check::check_hooks(&config);
        issues += hook_issues;
    } else {
        println!("[WARN] Skipping template and hook checks (config missing)");
        issues += 1;
    }

    let engine_issues = engine_check::check_color_engine();
    issues += engine_issues;

    println!();
    if issues == 0 {
        println!("All checks passed.");
        Ok(true)
    } else {
        println!("Diagnostics found {issues} issue(s).");
        Ok(false)
    }
}
