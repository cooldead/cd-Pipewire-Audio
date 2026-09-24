//! Building and sending SPA Props PODs for application audio streams.
//!
//! The helper takes an already-linear volume (`linear = cubic³`, see the parent
//! module) and a mute flag; either may be `None` to leave that attribute alone.

/// Build and send a Props POD setting channelVolumes and/or mute on a node.
pub(super) fn set_node_props(
	node: &pipewire::node::Node,
	linear_volume: Option<f32>,
	mute: Option<bool>,
) {
	use pipewire::spa::param::ParamType;
	use pipewire::spa::pod::{
		Object, Property, PropertyFlags, Value, ValueArray, serialize::PodSerializer,
	};

	let mut properties: Vec<Property> = Vec::new();
	if let Some(v) = linear_volume {
		// Apply to a stereo pair; PipeWire fans out / matches channel count.
		properties.push(Property {
			key: pipewire::spa::sys::SPA_PROP_channelVolumes,
			flags: PropertyFlags::empty(),
			value: Value::ValueArray(ValueArray::Float(vec![v, v])),
		});
	}
	if let Some(m) = mute {
		properties.push(Property {
			key: pipewire::spa::sys::SPA_PROP_mute,
			flags: PropertyFlags::empty(),
			value: Value::Bool(m),
		});
	}
	if properties.is_empty() {
		return;
	}

	let object = Value::Object(Object {
		type_: pipewire::spa::sys::SPA_TYPE_OBJECT_Props,
		id: pipewire::spa::sys::SPA_PARAM_Props,
		properties,
	});

	let mut bytes = Vec::new();
	if PodSerializer::serialize(std::io::Cursor::new(&mut bytes), &object).is_err() {
		log::warn!("Failed to serialize Props POD");
		return;
	}
	match pipewire::spa::pod::Pod::from_bytes(&bytes) {
		Some(pod) => node.set_param(ParamType::Props, 0, pod),
		None => log::warn!("Failed to build Props POD from bytes"),
	}
}
