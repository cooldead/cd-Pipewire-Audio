//! Live key re-rendering for CD-Active App Volume.
//!
//! The active application's display follows the focused KDE/KWin window PID
//! and the corresponding PipeWire stream state.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use openaction::*;

use crate::active_window::ActiveWindowHandle;
use crate::app_match::FocusedApp;
use crate::color::BarColors;
use crate::pw::PwHandle;

/// Shared, cheaply-cloneable store of per-instance CD-Active App Volume
/// settings needed for background redraws.
#[derive(Clone, Default)]
pub struct Refresher {
	active_colors: Arc<Mutex<HashMap<String, BarColors>>>,
}

impl Refresher {
	pub fn set_active_colors(&self, instance_id: &str, colors: &BarColors) {
		self.active_colors
			.lock()
			.unwrap()
			.insert(instance_id.to_owned(), colors.clone());
	}

	pub fn forget_active_colors(&self, instance_id: &str) {
		self.active_colors.lock().unwrap().remove(instance_id);
	}
}

/// Build the same focused-process candidate list used by ActiveAppVolumeAction.
///
/// The exact KWin PID is always first. Descendants follow so applications such
/// as Brave/Chromium can resolve their audio-owning child process.
fn focused_pids(active: &ActiveWindowHandle) -> Vec<u32> {
	let focused = active.pid();

	if focused == 0 {
		return Vec::new();
	}

	let Some(app) = FocusedApp::from_pid(focused) else {
		return vec![focused];
	};

	let mut related: Vec<u32> = app.related_pids().filter(|&pid| pid != focused).collect();

	related.sort_unstable();

	let mut pids = Vec::with_capacity(related.len() + 1);
	pids.push(focused);
	pids.extend(related);

	pids
}

/// Redraw every visible CD-Active App Volume instance from live state.
pub async fn refresh_all(pw: &PwHandle, refresher: &Refresher, active: &ActiveWindowHandle) {
	use crate::display;

	let active_colors = refresher.active_colors.lock().unwrap().clone();

	let pids = focused_pids(active);

	let state = if pids.is_empty() {
		None
	} else {
		pw.app_pids_state(&pids)
	};
	for inst in
		visible_instances(crate::actions::active_app_volume::ActiveAppVolumeAction::UUID).await
	{
		let colors = active_colors
			.get(&inst.instance_id)
			.cloned()
			.unwrap_or_default();

		match &state {
			Some((_pid, name, vol, mute)) => {
				let _ = display::app(&inst, Some(name), *vol, *mute, &colors).await;
			}

			None => {
				let _ = display::app(&inst, None, 0.0, false, &colors).await;
			}
		}
	}
}
