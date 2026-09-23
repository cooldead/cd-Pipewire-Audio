//! Focused-application process matching.
//!
//! KDE gives us the PID belonging to the focused window. Audio applications may
//! create their PipeWire stream from that process or from a descendant process
//! in the same application process tree (Chromium/Brave, Wine/Proton, etc.).

use std::collections::{HashMap, HashSet};
use std::fs;

#[derive(Debug, Clone)]
pub struct FocusedApp {
	/// Focused PID plus all descendants.
	related_pids: HashSet<u32>,
}

impl FocusedApp {
	pub fn from_pid(pid: u32) -> Option<Self> {
		if pid == 0 {
			return None;
		}

		let processes = process_table();
		let mut related_pids = HashSet::new();

		related_pids.insert(pid);
		add_descendants(pid, &processes, &mut related_pids);

		Some(Self { related_pids })
	}
	pub fn related_pids(&self) -> impl Iterator<Item = u32> + '_ {
		self.related_pids.iter().copied()
	}
}

fn process_table() -> HashMap<u32, u32> {
	let mut table = HashMap::new();

	let Ok(entries) = fs::read_dir("/proc") else {
		return table;
	};

	for entry in entries.flatten() {
		let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
			continue;
		};

		let Ok(pid) = name.parse::<u32>() else {
			continue;
		};

		if let Some(ppid) = parent_pid(pid) {
			table.insert(pid, ppid);
		}
	}

	table
}

fn parent_pid(pid: u32) -> Option<u32> {
	let status = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;

	status
		.lines()
		.find_map(|line| line.strip_prefix("PPid:"))
		.and_then(|value| value.trim().parse::<u32>().ok())
}

fn add_descendants(parent: u32, processes: &HashMap<u32, u32>, output: &mut HashSet<u32>) {
	let children: Vec<u32> = processes
		.iter()
		.filter_map(|(&pid, &ppid)| (ppid == parent).then_some(pid))
		.collect();

	for child in children {
		if output.insert(child) {
			add_descendants(child, processes, output);
		}
	}
}
