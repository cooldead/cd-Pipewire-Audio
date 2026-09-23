use crate::active_window::ActiveWindowHandle;
use crate::app_match::FocusedApp;
use crate::color::BarColors;
use crate::command::Command;
use crate::pw::PwHandle;
use openaction::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ActiveAppVolumeSettings {
	pub step: u8,
	pub mode: super::KeyMode,
	#[serde(flatten)]
	pub colors: BarColors,
}

impl Default for ActiveAppVolumeSettings {
	fn default() -> Self {
		Self {
			step: 5,
			mode: super::KeyMode::Up,
			colors: BarColors::default(),
		}
	}
}

pub struct ActiveAppVolumeAction {
	pub pw: PwHandle,
	pub active: ActiveWindowHandle,
	pub refresher: crate::refresh::Refresher,
}

#[async_trait]
impl Action for ActiveAppVolumeAction {
	const UUID: &'static str = super::action_uuid!("activeappvolume");
	type Settings = ActiveAppVolumeSettings;

	async fn key_down(&self, i: &Instance, s: &Self::Settings) -> OpenActionResult<()> {
		let d = super::step_fraction(s.step);
		match s.mode {
			super::KeyMode::Up => self.adjust(i, s, d).await,
			super::KeyMode::Down => self.adjust(i, s, -d).await,
			super::KeyMode::Mute => self.toggle_mute(i, s).await,
		}
	}

	async fn dial_rotate(
		&self,
		i: &Instance,
		s: &Self::Settings,
		t: i16,
		pressed: bool,
	) -> OpenActionResult<()> {
		log::info!(
			"ActiveAppVolume: dial_rotate instance={} ticks={} pressed={} pid={}",
			i.instance_id,
			t,
			pressed,
			self.active.pid()
		);
		self.adjust(i, s, t as f32 * super::step_fraction(s.step))
			.await
	}

	async fn dial_down(&self, i: &Instance, s: &Self::Settings) -> OpenActionResult<()> {
		self.toggle_mute(i, s).await
	}

	async fn touch_tap(
		&self,
		i: &Instance,
		s: &Self::Settings,
		_: (u16, u16),
		_: bool,
	) -> OpenActionResult<()> {
		self.toggle_mute(i, s).await
	}

	async fn will_appear(&self, i: &Instance, s: &Self::Settings) -> OpenActionResult<()> {
		self.refresher.set_active_colors(&i.instance_id, &s.colors);
		self.render(i, s).await
	}

	async fn will_disappear(&self, i: &Instance, _s: &Self::Settings) -> OpenActionResult<()> {
		self.refresher.forget_active_colors(&i.instance_id);
		Ok(())
	}

	async fn did_receive_settings(&self, i: &Instance, s: &Self::Settings) -> OpenActionResult<()> {
		self.refresher.set_active_colors(&i.instance_id, &s.colors);
		self.render(i, s).await
	}
}

impl ActiveAppVolumeAction {
	/// Exact focused PID first, followed by descendants.
	fn focused_pids(&self) -> Vec<u32> {
		let focused = self.active.pid();
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

	fn state(&self) -> Option<(u32, String, f32, bool)> {
		let pids = self.focused_pids();
		self.pw.app_pids_state(&pids)
	}

	async fn adjust(
		&self,
		i: &Instance,
		s: &ActiveAppVolumeSettings,
		d: f32,
	) -> OpenActionResult<()> {
		let pids = self.focused_pids();
		if pids.is_empty() {
			return self.render(i, s).await;
		}

		if let Some((_matched_pid, name, cur, mute)) = self.pw.app_pids_state(&pids) {
			if mute {
				self.pw
					.send(Command::SetAppPidsMute(pids.clone(), Some(false)));
			}
			self.pw.send(Command::AdjustAppPidsVolume(pids, d));
			return crate::display::app(
				i,
				Some(&name),
				(cur + d).clamp(0.0, 1.5),
				false,
				&s.colors,
			)
			.await;
		}

		self.pw.send(Command::AdjustAppPidsVolume(pids, d));
		self.render(i, s).await
	}

	async fn toggle_mute(&self, i: &Instance, s: &ActiveAppVolumeSettings) -> OpenActionResult<()> {
		let pids = self.focused_pids();
		if pids.is_empty() {
			return self.render(i, s).await;
		}

		if let Some((_matched_pid, name, vol, mute)) = self.pw.app_pids_state(&pids) {
			self.pw.send(Command::SetAppPidsMute(pids, None));
			return crate::display::app(i, Some(&name), vol, !mute, &s.colors).await;
		}

		self.pw.send(Command::SetAppPidsMute(pids, None));
		self.render(i, s).await
	}

	async fn render(&self, i: &Instance, s: &ActiveAppVolumeSettings) -> OpenActionResult<()> {
		match self.state() {
			Some((_p, n, v, m)) => crate::display::app(i, Some(&n), v, m, &s.colors).await,
			None => crate::display::app(i, None, 0.0, false, &s.colors).await,
		}
	}
}
