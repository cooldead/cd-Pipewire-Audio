//! OpenDeck PipeWire audio control plugin — native Rust, no Node.js.
//!
//! The plugin process is launched by OpenDeck and speaks the Elgato/OpenAction
//! WebSocket protocol via the `openaction` crate. All audio control goes through
//! a dedicated PipeWire main-loop thread (see [`pw`]).

mod actions;
mod active_window;
mod color;
mod command;
mod display;
mod pw;
mod refresh;
mod render;

use actions::active_app_volume;
use openaction::*;

#[tokio::main]
async fn main() -> OpenActionResult<()> {
	// Logs are written to stdout, which OpenDeck redirects into the plugin log
	// file (~/.config/opendeck/logs/plugins/<uuid>.log).
	{
		use simplelog::*;
		if let Err(error) = TermLogger::init(
			LevelFilter::Info,
			Config::default(),
			TerminalMode::Stdout,
			ColorChoice::Never,
		) {
			eprintln!("Logger initialization failed: {error}");
		}
	}

	// Spin up the PipeWire backend (its own OS thread; not Send-safe for tokio).
	let pw = match pw::start() {
		Ok(handle) => handle,
		Err(error) => {
			log::error!("Failed to start PipeWire backend: {error}");
			// Still connect so OpenDeck doesn't treat the plugin as crashed.
			return run(std::env::args().collect()).await;
		}
	};

	let refresher = refresh::Refresher::default();
	let active = active_window::start();

	register_action(active_app_volume::ActiveAppVolumeAction {
		pw: pw.clone(),
		active: active.clone(),
		refresher: refresher.clone(),
	})
	.await;

	// Re-render visible keys whenever PipeWire state changes out-of-band (volume
	// changed by wpctl, media keys, another app…). The PipeWire thread signals
	// changes on the watch channel; we coalesce bursts and redraw once each.
	{
		let pw = pw.clone();
		let refresher = refresher.clone();
		let active = active.clone();
		tokio::spawn(async move {
			let mut changes = pw.subscribe();
			let mut active_changes = active.subscribe();
			loop {
				refresh::refresh_all(&pw, &refresher, &active).await;
				tokio::select! {
					result = changes.changed() => if result.is_err() { break; },
					result = active_changes.changed() => if result.is_err() { break; },
				}
			}
		});
	}

	run(std::env::args().collect()).await
}
