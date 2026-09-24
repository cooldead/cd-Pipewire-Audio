//! PipeWire handle used by CD-Active App Volume.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::command::Command;

use super::backend::run_loop;
use super::{Channels, ProcessApps};

/// Cheap, cloneable handle used by CD-Active App Volume.
#[derive(Clone)]
pub struct PwHandle {
	tx: Arc<Mutex<pipewire::channel::Sender<Command>>>,
	process_apps: ProcessApps,
	notify: Arc<tokio::sync::watch::Sender<u64>>,
}

impl PwHandle {
	pub fn send(&self, cmd: Command) {
		let _ = self.tx.lock().unwrap().send(cmd);
	}
	/// Prefer the exact focused PID; if it has no audio, return the first
	/// related PID that currently owns a PipeWire output stream.
	pub fn app_pids_state(&self, pids: &[u32]) -> Option<(u32, String, f32, bool)> {
		let apps = self.process_apps.lock().unwrap();
		for &pid in pids {
			if let Some((name, volume, mute)) = apps.get(&pid) {
				return Some((pid, name.clone(), *volume, *mute));
			}
		}
		None
	}

	pub fn subscribe(&self) -> tokio::sync::watch::Receiver<u64> {
		self.notify.subscribe()
	}
}

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
		.name("cd-active-app-volume-pw".into())
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
