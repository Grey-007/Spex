use std::path::PathBuf;
use std::{io, io::Write};

use clap::{ArgAction, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{self, Shell};

#[derive(Debug, Parser)]
#[command(
    name = "spex",
    version,
    about = "Extract, preview, and export image-based color themes"
)]
pub struct Cli {
    /// Input image path (shortcut for `spex generate <IMAGE>`)
    #[arg(value_name = "IMAGE")]
    pub image: Option<PathBuf>,

    /// Number of palette colors to generate
    #[arg(long, global = true, default_value_t = 16, value_name = "N")]
    pub colors: usize,

    /// Theme mode for role assignment and ordering
    #[arg(long, global = true, default_value_t = ThemeArg::Dark, value_enum, value_name = "MODE")]
    pub theme: ThemeArg,

    /// Export generated theme palette
    #[arg(long, global = true, value_enum, value_name = "FORMAT")]
    pub export: Option<ExportArg>,

    /// Palette extraction method
    #[arg(long, global = true, default_value_t = ExtractorArg::Kmeans, value_enum, value_name = "METHOD")]
    pub extractor: ExtractorArg,

    /// Override config.toml path used by template engine
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Render templates but do not write files or execute hooks
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub dry_run: bool,

    /// Print additional debugging output
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub verbose: bool,

    /// Print template semantic role resolution during rendering
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub debug_theme: bool,

    /// Print palette metrics, Delta-E distances, and final role diagnostics
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub debug_colors: bool,

    /// Print final extracted palette metrics and quality checks
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub debug_palette: bool,

    /// Print extractor internals such as LAB centroids, cluster sizes, and fallback usage
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub debug_extractor: bool,

    /// Disable terminal palette preview output
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub no_preview: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ThemeArg {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ExportArg {
    Json,
    Css,
    Terminal,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ExtractorArg {
    Kmeans,
    Mediancut,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Generate palette, assign roles, render templates, run hooks
    Generate {
        /// Input image path
        #[arg(value_name = "IMAGE")]
        image: PathBuf,
    },
    /// Preview palette in terminal without rendering templates
    Preview {
        /// Input image path
        #[arg(value_name = "IMAGE")]
        image: PathBuf,
    },
    /// Start daemon mode (reserved for future implementation)
    Daemon,
    /// Print shell completion script
    Completions {
        /// Target shell for completion script
        #[arg(value_name = "SHELL")]
        shell: Shell,
    },
    /// Print resolved config path information
    Config,
    /// Run diagnostics for config, templates, hooks, and color engine
    Doctor,
}

pub fn print_completions(shell: Shell) -> io::Result<()> {
    let mut command = Cli::command();
    let mut buffer = Vec::new();
    clap_complete::generate(shell, &mut command, "spex", &mut buffer);

    match io::stdout().write_all(&buffer) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(err) => Err(err),
    }
}
