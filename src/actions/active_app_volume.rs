use crate::active_window::ActiveWindowHandle;
use crate::app_match::focused_pids;
use crate::color::BarColors;
use crate::command::Command;
use crate::pw::PwHandle;
use openaction::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ActiveAppVolumeSettings {
	pub step: u8,
	pub bar_style: crate::display::BarStyle,
	pub mode: super::KeyMode,
	#[serde(flatten)]
	pub colors: BarColors,
}

impl Default for ActiveAppVolumeSettings {
	fn default() -> Self {
		Self {
			step: 5,
			bar_style: crate::display::BarStyle::default(),
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
		log::debug!(
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
		self.refresher
			.set_appearance(&i.instance_id, &s.colors, s.bar_style)
			.await;
		self.render(i, s).await
	}

	async fn will_disappear(&self, i: &Instance, _s: &Self::Settings) -> OpenActionResult<()> {
		self.refresher.forget_appearance(&i.instance_id).await;
		Ok(())
	}

	async fn did_receive_settings(&self, i: &Instance, s: &Self::Settings) -> OpenActionResult<()> {
		self.refresher
			.set_appearance(&i.instance_id, &s.colors, s.bar_style)
			.await;
		self.render(i, s).await
	}
}

impl ActiveAppVolumeAction {
	fn state(&self) -> Option<(u32, String, f32, bool)> {
		let pids = focused_pids(self.active.pid());
		self.pw.app_pids_state(&pids)
	}

	async fn adjust(
		&self,
		i: &Instance,
		s: &ActiveAppVolumeSettings,
		d: f32,
	) -> OpenActionResult<()> {
		let pids = focused_pids(self.active.pid());
		if pids.is_empty() {
			return self.render(i, s).await;
		}

		if let Some((_, _, _, true)) = self.pw.app_pids_state(&pids) {
			self.pw
				.send(Command::SetAppPidsMute(pids.clone(), Some(false)));
		}

		self.pw.send(Command::AdjustAppPidsVolume(pids, d));
		// PipeWire's notification triggers the redraw. An optimistic frame here
		// can race with that refresh and overwrite it with an older value.
		Ok(())
	}

	async fn toggle_mute(&self, i: &Instance, s: &ActiveAppVolumeSettings) -> OpenActionResult<()> {
		let pids = focused_pids(self.active.pid());
		if pids.is_empty() {
			return self.render(i, s).await;
		}

		self.pw.send(Command::SetAppPidsMute(pids, None));
		Ok(())
	}

	async fn render(&self, i: &Instance, _s: &ActiveAppVolumeSettings) -> OpenActionResult<()> {
		self.refresher.draw(i, self.state().as_ref()).await
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::display::BarStyle;

	#[test]
	fn existing_settings_keep_gauge_and_original_style_round_trips() {
		let old: ActiveAppVolumeSettings =
			serde_json::from_str(r#"{"step":7,"mode":"down"}"#).unwrap();
		assert_eq!(old.bar_style, BarStyle::Gauge);
		let original: ActiveAppVolumeSettings =
			serde_json::from_str(r##"{"bar_style":"original","mute_color":"#123456"}"##).unwrap();
		assert_eq!(original.bar_style, BarStyle::Original);
		assert_eq!(original.step, 5);
		let saved = serde_json::to_value(&original).unwrap();
		assert_eq!(saved["bar_style"], "original");
		assert_eq!(saved["mute_color"], "#123456");
		let unknown: ActiveAppVolumeSettings =
			serde_json::from_str(r#"{"bar_style":"future"}"#).unwrap();
		assert_eq!(unknown.bar_style, BarStyle::Gauge);
	}
}
