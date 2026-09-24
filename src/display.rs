//! Surface updates for CD-Active App Volume.
use crate::color::BarColors;
use crate::render;
use openaction::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BarStyle {
	Original,
	#[default]
	#[serde(other)]
	Gauge,
}

impl BarStyle {
	pub fn layout(self) -> &'static str {
		match self {
			Self::Gauge => "layouts/active-app-volume.json",
			Self::Original => "layouts/original-volume.json",
		}
	}
}

pub async fn app(
	instance: &Instance,
	app: Option<&str>,
	vol: f32,
	muted: bool,
	colors: &BarColors,
	style: BarStyle,
) -> OpenActionResult<()> {
	let label = app.unwrap_or("No Focused App");
	if instance.controller == "Encoder" {
		// Layout changes are handled by Refresher only on appearance/style changes.
		let feedback = match style {
			BarStyle::Gauge => {
				json!({"full-canvas": render::label_bar_key(label, app.is_some(), vol, muted, colors)})
			}
			BarStyle::Original => {
				render::original_bar_feedback(label, app.is_some(), vol, muted, colors)
			}
		};
		instance.set_feedback(&feedback).await
	} else if app.is_none() {
		instance.set_title(Some("—"), None).await
	} else {
		instance
			.set_image(
				Some(render::label_bar_key(label, true, vol, muted, colors)),
				None,
			)
			.await
	}
}
