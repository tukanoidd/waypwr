mod app;
mod cli;
mod config;

use app::App;
use clap::Parser;
use cli::Cli;
use config::Config;
use iced_fonts::{NERD_FONT_BYTES, REQUIRED_FONT_BYTES};
use iced_layershell::{
    reexport::{Anchor, KeyboardInteractivity, Layer},
    settings::{LayerShellSettings, Settings, StartMode},
    Application,
};
use snafu::{report, ResultExt, Whatever};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[report]
pub fn main() -> Result<(), Whatever> {
    let Cli {
        config,
        theme,

        quiet,
        debug,
        trace,
    } = Cli::parse();

    init_logging(quiet, debug, trace)?;

    let mut config = Config::open(config)?;

    if let Some(theme) = theme {
        config.theme = theme;
    }

    App::run(Settings {
        layer_settings: LayerShellSettings {
            layer: Layer::Overlay,
            anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
            start_mode: StartMode::Active,
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            ..Default::default()
        },
        flags: config,
        fonts: vec![REQUIRED_FONT_BYTES.into(), NERD_FONT_BYTES.into()],
        ..Default::default()
    })
    .whatever_context("Failed to run the app")?;

    Ok(())
}

fn init_logging(quiet: bool, debug: bool, trace: bool) -> Result<(), Whatever> {
    let level = quiet
        .then_some(Level::WARN)
        .or_else(|| (debug || cfg!(debug_assertions)).then_some(Level::DEBUG))
        .or_else(|| trace.then_some(Level::TRACE))
        .unwrap_or(Level::INFO);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(tracing_subscriber::filter::LevelFilter::from(level))
        .try_init()
        .whatever_context("Failed to initialize logging")?;

    Ok(())
}
