//! Live key re-rendering for CD-Active App Volume.
//!
//! The active application's display follows the focused KDE/KWin window PID
//! and the corresponding PipeWire stream state.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use openaction::*;

use crate::active_window::ActiveWindowHandle;
use crate::app_match::focused_pids;
use crate::color::BarColors;
use crate::pw::PwHandle;

/// Shared, cheaply-cloneable store of per-instance CD-Active App Volume
/// settings needed for background redraws.
#[derive(Clone, Default)]
pub struct Refresher {
	appearances: Arc<Mutex<HashMap<String, Appearance>>>,
}

#[derive(Default)]
struct Appearance {
	colors: BarColors,
	style: crate::display::BarStyle,
	applied_style: Option<crate::display::BarStyle>,
}

impl Refresher {
	pub async fn set_appearance(
		&self,
		id: &str,
		colors: &BarColors,
		style: crate::display::BarStyle,
	) {
		let mut entries = self.appearances.lock().await;
		let entry = entries.entry(id.to_owned()).or_default();
		entry.colors = colors.clone();
		entry.style = style;
	}

	pub async fn forget_appearance(&self, id: &str) {
		self.appearances.lock().await.remove(id);
	}

	pub async fn draw(
		&self,
		instance: &Instance,
		state: Option<&(u32, String, f32, bool)>,
	) -> OpenActionResult<()> {
		// Serialize layout and feedback together, using the latest saved appearance.
		// Background refreshes must not restore an old layout after a style switch.
		let mut entries = self.appearances.lock().await;
		let Some(entry) = entries.get_mut(&instance.instance_id) else {
			return Ok(());
		};
		if instance.controller == "Encoder" && entry.applied_style != Some(entry.style) {
			instance
				.set_feedback_layout(entry.style.layout().to_owned())
				.await?;
			entry.applied_style = Some(entry.style);
		}
		let (name, volume, mute) = match state {
			Some((_, name, volume, mute)) => (Some(name.as_str()), *volume, *mute),
			None => (None, 0.0, false),
		};
		crate::display::app(instance, name, volume, mute, &entry.colors, entry.style).await
	}
}

/// Redraw every visible CD-Active App Volume instance from live state.
pub async fn refresh_all(pw: &PwHandle, refresher: &Refresher, active: &ActiveWindowHandle) {
	let instances =
		visible_instances(crate::actions::active_app_volume::ActiveAppVolumeAction::UUID).await;
	if instances.is_empty() {
		return;
	}
	let pids = focused_pids(active.pid());

	let state = if pids.is_empty() {
		None
	} else {
		pw.app_pids_state(&pids)
	};
	for inst in instances {
		let _ = refresher.draw(&inst, state.as_ref()).await;
	}
}
