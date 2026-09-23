//! On-the-fly visual feedback.
//!
//! Keypad buttons get a full 128×128 SVG key image via `set_image`. Encoders
//! (Stream Deck+ dials) use the native `$B1` touchstrip layout, for which this
//! module builds the `setFeedback` value+bar payload (see [`bar_feedback`]); the
//! icon and title stay the dial's own OpenDeck configuration.

use crate::color::BarColors;
use base64::Engine;

/// Build a `data:image/svg+xml;base64,...` URI suitable for `Instance::set_image`.
fn data_uri(svg: &str) -> String {
	let b64 = base64::engine::general_purpose::STANDARD.encode(svg.as_bytes());
	format!("data:image/svg+xml;base64,{b64}")
}

/// A bold slash drawn corner-to-corner across the whole key to signal mute, in
/// the configured mute colour. Overlaid last (on top of the label/bar) by the
/// keypad renderers when muted; empty when unmuted.

/// A labelled level-bar key/encoder: a custom label with a level bar (no
/// percentage), shared by the device / input / app volume actions.
pub fn label_bar_key(
	label: &str,
	known: bool,
	volume_cubic: f32,
	muted: bool,
	colors: &BarColors,
) -> String {
	let label = escape(&label.chars().take(18).collect::<String>());
	let pct = (volume_cubic * 100.0).round().clamp(0.0, 150.0);

	// 100% is the top-center of the gauge.
	// 0–100 occupies the left half; 100–150 occupies the right half.
	let value_angle = |value: f32| -> f32 {
		if value <= 100.0 {
			180.0 - (value / 100.0) * 90.0
		} else {
			90.0 - ((value - 100.0) / 50.0) * 90.0
		}
	};

	// 200×100 encoder display.
	let cx = 100.0;
	let cy = 86.0;
	let radius = 58.0;

	// Build a correctly positioned colored arc segment.
	let arc_segment = |start: f32, end: f32, color: &str| -> String {
		let start_angle = value_angle(start).to_radians();
		let end_angle = value_angle(end).to_radians();

		let x1 = cx + radius * start_angle.cos();
		let y1 = cy - radius * start_angle.sin();
		let x2 = cx + radius * end_angle.cos();
		let y2 = cy - radius * end_angle.sin();

		format!(
			r##"<path d="M {x1:.2} {y1:.2} A {radius} {radius} 0 0 1 {x2:.2} {y2:.2}"
			fill="none" stroke="{color}" stroke-width="9"
			stroke-linecap="butt"/>"##
		)
	};

	// Red → yellow → green → yellow → red.
	let arc = if known && !muted {
		[
			arc_segment(0.0, 40.0, "#ff2020"),
			arc_segment(40.0, 79.0, "#ffd21f"),
			arc_segment(79.0, 110.0, "#20e83f"),
			arc_segment(110.0, 130.0, "#ffd21f"),
			arc_segment(130.0, 150.0, "#ff2020"),
		]
		.join("")
	} else {
		arc_segment(0.0, 150.0, "#555555")
	};

	// Transition marks only:
	// 40, 80, 100, 130.
	let marks = [40.0, 79.0, 100.0, 110.0, 130.0];

	let mut mark_svg = String::new();

	for value in marks {
		let angle = value_angle(value).to_radians();

		let tick_inner = 53.0;
		let tick_outer = 63.0;

		let tx1 = cx + tick_inner * angle.cos();
		let ty1 = cy - tick_inner * angle.sin();
		let tx2 = cx + tick_outer * angle.cos();
		let ty2 = cy - tick_outer * angle.sin();

		mark_svg.push_str(&format!(
			r##"<line x1="{tx1:.2}" y1="{ty1:.2}" x2="{tx2:.2}" y2="{ty2:.2}"
			stroke="#ffffff" stroke-width="2.5" stroke-linecap="round"/>"##
		));
	}

	// Current volume indicator.
	let indicator_angle = value_angle(pct).to_radians();

	let indicator_inner = 47.0;
	let indicator_outer = 63.0;

	let x1 = cx + indicator_inner * indicator_angle.cos();
	let y1 = cy - indicator_inner * indicator_angle.sin();
	let x2 = cx + indicator_outer * indicator_angle.cos();
	let y2 = cy - indicator_outer * indicator_angle.sin();

	let indicator = format!(
		r##"<line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}"
		stroke="#ffffff" stroke-width="3.5" stroke-linecap="round"/>"##
	);
	let mute_label = if muted {
		format!(
			r##"<text x="100" y="92" font-family="Noto Sans"
			font-size="15" font-weight="bold"
			fill="{}" text-anchor="middle">MUTED</text>"##,
			colors.mute()
		)
	} else {
		String::new()
	};
	let svg = format!(
		r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
		<rect width="200" height="100" fill="#0b0b0b"/>

		{arc}

		{mark_svg}

		{indicator}

		<!-- Current volume -->
		<text x="100" y="68" font-family="Noto Sans"
		font-size="24" font-weight="bold"
		fill="#ffffff" text-anchor="middle">{pct:.0}%</text>

		<!-- Active application -->
		<text x="100" y="15" font-family="Noto Sans"
		font-size="14" font-weight="bold"
		fill="#ffffff" text-anchor="middle">{label}</text>

		{mute_label}
		</svg>"##,
	);

	data_uri(&svg)
}

/// Minimal XML text escaping for untrusted strings (e.g. application names).
fn escape(s: &str) -> String {
	s.replace('&', "&amp;")
		.replace('<', "&lt;")
		.replace('>', "&gt;")
}

// --- Encoder touchstrip feedback (Stream Deck+) ------------------------------
