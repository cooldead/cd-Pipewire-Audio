//! Focused-application process matching.
//!
//! KDE gives us the PID belonging to the focused window. Audio applications may
//! create their PipeWire stream from that process or from a descendant process
//! in the same application process tree (Chromium/Brave, Wine/Proton, etc.).

use std::collections::{HashMap, HashSet};
use std::fs;

/// Exact focused PID first, followed by sorted descendants. Read a fresh table
/// each time so newly spawned audio processes are immediately discoverable.
pub fn focused_pids(pid: u32) -> Vec<u32> {
	if pid == 0 {
		return Vec::new();
	}
	candidates(pid, &process_table())
}

fn candidates(pid: u32, processes: &HashMap<u32, u32>) -> Vec<u32> {
	// Index children once instead of scanning the full table for every parent.
	let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
	for (&child, &parent) in processes {
		children.entry(parent).or_default().push(child);
	}
	let mut seen = HashSet::from([pid]);
	let mut pending = vec![pid];
	let mut descendants = Vec::new();
	while let Some(parent) = pending.pop() {
		if let Some(children) = children.get(&parent) {
			for &child in children {
				if seen.insert(child) {
					descendants.push(child);
					pending.push(child);
				}
			}
		}
	}
	descendants.sort_unstable();
	let mut result = Vec::with_capacity(descendants.len() + 1);
	result.push(pid);
	result.extend(descendants);
	result
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn exact_pid_precedes_sorted_descendants_and_excludes_unrelated_processes() {
		let table = HashMap::from([(90, 10), (5, 90), (30, 10), (70, 1)]);
		assert_eq!(candidates(10, &table), vec![10, 5, 30, 90]);
		assert_eq!(candidates(999, &table), vec![999]);
		assert!(focused_pids(0).is_empty());
	}

	#[test]
	fn cycles_and_deep_trees_do_not_recurse_or_duplicate_candidates() {
		assert_eq!(
			candidates(10, &HashMap::from([(10, 20), (20, 10)])),
			vec![10, 20]
		);
		let table = (2..=10_000).map(|pid| (pid, pid - 1)).collect();
		assert_eq!(candidates(1, &table), (1..=10_000).collect::<Vec<_>>());
	}
}
