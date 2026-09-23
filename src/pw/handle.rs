//! PipeWire handle used by CD-Active App Volume.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::command::Command;

use super::Channels;
use super::backend::run_loop;

/// Cheap, cloneable handle used by CD-Active App Volume.
#[derive(Clone)]
pub struct PwHandle {
	tx: Arc<Mutex<pipewire::channel::Sender<Command>>>,
	process_apps: Arc<Mutex<HashMap<u32, (String, f32, bool)>>>,
	notify: Arc<tokio::sync::watch::Sender<u64>>,
}

impl PwHandle {
	/// Send an intent to the PipeWire backend thread.
	pub fn send(&self, cmd: Command) {
		let _ = self.tx.lock().unwrap().send(cmd);
	}

	/// Live aggregate (application name, volume, mute) for an OS process id.
	pub fn app_pid_state(&self, pid: u32) -> Option<(String, f32, bool)> {
		self.process_apps.lock().unwrap().get(&pid).cloned()
	}

	/// Subscribe to PipeWire state-change notifications.
	pub fn subscribe(&self) -> tokio::sync::watch::Receiver<u64> {
		self.notify.subscribe()
	}
}

/// Start the PipeWire backend thread. Returns immediately.
pub fn start() -> Result<PwHandle> {
	let process_apps = Arc::new(Mutex::new(HashMap::new()));

	let (notify_tx, _) = tokio::sync::watch::channel(0u64);
	let notify = Arc::new(notify_tx);

	let chans = Channels {
		process_apps: process_apps.clone(),
		notify: notify.clone(),
	};

	let (tx, rx) = pipewire::channel::channel();

	std::thread::Builder::new()
		.name("cooldeadpipewire-pw".into())
		.spawn(move || {
			if let Err(e) = run_loop(rx, chans) {
				log::error!("PipeWire backend stopped: {e:#}");
			}
		})?;

	Ok(PwHandle {
		tx: Arc::new(Mutex::new(tx)),
		process_apps,
		notify,
	})
}
