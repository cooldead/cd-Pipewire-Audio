//! Gauge rendering and original upstream horizontal touchstrip feedback.

use crate::color::BarColors;
use base64::Engine;

/// Build a `data:image/svg+xml;base64,...` URI suitable for `Instance::set_image`.
fn data_uri(svg: &str) -> String {
	let b64 = base64::engine::general_purpose::STANDARD.encode(svg.as_bytes());
	format!("data:image/svg+xml;base64,{b64}")
}

/// The existing semicircular gauge, shared by keys and gauge-style encoders.
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

	// User-configurable sections, defaulting to red/yellow/green/yellow/red.
	let arc = if known && !muted {
		[
			arc_segment(0.0, 40.0, colors.gauge_low_color()),
			arc_segment(40.0, 79.0, colors.gauge_lower_mid_color()),
			arc_segment(79.0, 115.0, colors.gauge_normal_color()),
			arc_segment(115.0, 135.0, colors.gauge_boost_color()),
			arc_segment(135.0, 150.0, colors.gauge_high_color()),
		]
		.join("")
	} else {
		arc_segment(0.0, 150.0, "#555555")
	};

	// Color transitions plus the 100% reference mark. Boost marks follow
	// their actual values, rather than mirroring the lower-volume side.
	let marks = [40.0, 79.0, 100.0, 115.0, 135.0];

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

	let indicator_inner = 44.0;
	let indicator_outer = 66.0;

	let x1 = cx + indicator_inner * indicator_angle.cos();
	let y1 = cy - indicator_inner * indicator_angle.sin();
	let x2 = cx + indicator_outer * indicator_angle.cos();
	let y2 = cy - indicator_outer * indicator_angle.sin();

	let indicator = format!(
		r##"<line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}"
		stroke="#0b0b0b" stroke-width="8" stroke-linecap="round"/>
		<line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}"
		stroke="#ff2020" stroke-width="4.5" stroke-linecap="round"/>"##
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

/// Matches sjourdois/opendeck-pipewire's native volume layout and feedback.
/// The bar fills at 100%; the number continues to show boosted volume to 150%.
pub fn original_bar_feedback(
	title: &str,
	known: bool,
	volume: f32,
	muted: bool,
	colors: &BarColors,
) -> serde_json::Value {
	let pct = (volume * 100.0).round().clamp(0.0, 150.0);
	let (text, text_color, bar_color, value) = if !known {
		("n/a".to_owned(), "#eab308", "#eab308", 100.0)
	} else if muted {
		(
			"muted".to_owned(),
			colors.mute(),
			colors.mute(),
			pct.min(100.0),
		)
	} else {
		(
			format!("{pct:.0}%"),
			"#ffffff",
			colors.active(),
			pct.min(100.0),
		)
	};
	serde_json::json!({
		"title": title,
		"value": {"value": text, "color": text_color},
		"indicator": {"value": value, "bar_fill_c": bar_color},
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn original_bar_shows_level_boost_mute_and_unavailable() {
		let colors = BarColors {
			unmute_color: Some("#123456".into()),
			mute_color: Some("#abcdef".into()),
			..BarColors::default()
		};
		for (volume, text, bar) in [(0.0, "0%", 0.0), (0.72, "72%", 72.0), (1.5, "150%", 100.0)] {
			let fb = original_bar_feedback("Brave", true, volume, false, &colors);
			assert_eq!(fb["title"], "Brave");
			assert_eq!(fb["value"]["value"], text);
			assert_eq!(fb["indicator"]["value"], bar);
			assert_eq!(fb["indicator"]["bar_fill_c"], "#123456");
		}
		let muted = original_bar_feedback("Brave", true, 0.72, true, &colors);
		assert_eq!(muted["value"]["value"], "muted");
		assert_eq!(muted["value"]["color"], "#abcdef");
		assert_eq!(muted["indicator"]["bar_fill_c"], "#abcdef");
		let missing = original_bar_feedback("No Focused App", false, 0.0, true, &colors);
		assert_eq!(missing["value"]["value"], "n/a");
		assert_eq!(missing["indicator"]["value"], 100.0);
		assert_eq!(missing["indicator"]["bar_fill_c"], "#eab308");
	}
}
