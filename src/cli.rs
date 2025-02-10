use std::path::PathBuf;

use clap::Parser;
use iced::Theme;

/// Power menu for Wayland made with iced_layershell
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    pub config: Option<PathBuf>,
    #[arg(short, long, value_parser = crate::config::parse_theme_str)]
    pub theme: Option<Theme>,

    #[arg(long)]
    pub quiet: bool,
    #[arg(long)]
    pub debug: bool,
    #[arg(long)]
    pub trace: bool,
}
