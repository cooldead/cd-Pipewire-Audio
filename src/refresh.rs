//! Live key re-rendering for Active Application Volume.
//!
//! The active application's display follows the focused KDE/KWin window PID
//! and the corresponding PipeWire stream state.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use openaction::*;

use crate::active_window::ActiveWindowHandle;
use crate::color::BarColors;
use crate::pw::PwHandle;

/// Shared, cheaply-cloneable store of per-instance Active Application Volume
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

/// Redraw every visible Active Application Volume instance from live state.
pub async fn refresh_all(pw: &PwHandle, refresher: &Refresher, active: &ActiveWindowHandle) {
	use crate::display;

	let active_colors = refresher.active_colors.lock().unwrap().clone();

	for inst in
		visible_instances(crate::actions::active_app_volume::ActiveAppVolumeAction::UUID).await
	{
		let colors = active_colors
			.get(&inst.instance_id)
			.cloned()
			.unwrap_or_default();

		if active.pid() != 0 {
			if let Some((name, vol, mute)) = pw.app_pid_state(active.pid()) {
				let _ = display::app(&inst, Some(&name), vol, mute, &colors).await;
				continue;
			}
		}

		let _ = display::app(&inst, None, 0.0, false, &colors).await;
	}
}
