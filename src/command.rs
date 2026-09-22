//! Intents sent by Active Application Volume to the PipeWire backend.

#[derive(Debug)]
pub enum Command {
	/// Change the volume of every stream belonging to a process id.
	AdjustAppPidVolume(u32, f32),

	/// Set mute on every stream belonging to a process id. `None` toggles.
	SetAppPidMute(u32, Option<bool>),
}
