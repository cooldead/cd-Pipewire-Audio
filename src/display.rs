//! Surface updates for Active Application Volume.

use openaction::*;

use crate::color::BarColors;
use crate::render;

fn is_encoder(instance: &Instance) -> bool {
	instance.controller == "Encoder"
}

/// Render a volume-style surface: the `$B1` value+bar on an encoder touchstrip,
/// or an SVG key image on a keypad.
async fn bar(
	instance: &Instance,
	title: &str,
	known: bool,
	volume_cubic: f32,
	muted: bool,
	colors: &BarColors,
	keypad: impl FnOnce() -> String,
) -> OpenActionResult<()> {
	if is_encoder(instance) {
		instance
			.set_feedback(&render::bar_feedback(
				title,
				known,
				volume_cubic,
				muted,
				colors,
			))
			.await
	} else if !known {
		instance.set_title(Some("—"), None).await
	} else {
		instance.set_image(Some(keypad()), None).await
	}
}

/// Active Application Volume surface.
pub async fn app(
	instance: &Instance,
	app: Option<&str>,
	vol: f32,
	muted: bool,
	colors: &BarColors,
) -> OpenActionResult<()> {
	let label = app.unwrap_or("App");
	bar(instance, label, app.is_some(), vol, muted, colors, || {
		render::label_bar_key(label, vol, muted, colors)
	})
	.await
}
