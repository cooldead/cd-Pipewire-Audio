//! Intents sent by CD-Active App Volume to the PipeWire backend.

#[derive(Debug)]
pub enum Command {
	/// Change volume for the focused process family.
	///
	/// The first PID is the exact focused PID. The backend prefers an exact
	/// audio match and only falls back to the remaining related PIDs.
	AdjustAppPidsVolume(Vec<u32>, f32),

	/// Set/toggle mute for the focused process family. `None` toggles.
	SetAppPidsMute(Vec<u32>, Option<bool>),
}
