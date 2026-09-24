//! The PipeWire main-loop thread.
//!
//! Everything here runs on the single dedicated PipeWire thread (not `Send`):
//! application audio streams, handling commands from the actions, and reading back
//! node Props to keep the published state current.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use anyhow::Result;
use std::time::Duration;

use crate::command::Command;

use super::pod::set_node_props;
const RECONNECT_DELAY: Duration = Duration::from_secs(1);
use super::Channels;

struct NodeRef {
	/// Per-node application stream state tracked by process id.
	is_stream: bool,
	/// For an application output stream: its `application.name`.
	app_name: Option<String>,
	/// PID exposed directly by the PipeWire node, when available.
	process_id: Option<u32>,
	/// PipeWire client that owns this node.
	client_id: Option<u32>,
	mute: bool,
	volume_cubic: f32,
	proxy: pipewire::node::Node,
	// Listener must be kept alive for as long as the proxy.
	_listener: pipewire::node::NodeListener,
}

struct ClientRef {
	// Keep both alive so PipeWire continues sending Client info events.
	_proxy: pipewire::client::Client,
	_listener: pipewire::client::ClientListener,
}

struct Inner {
	nodes: HashMap<u32, NodeRef>,
	/// PipeWire client ID -> OS process ID.
	clients: HashMap<u32, u32>,
	/// Bound PipeWire clients kept alive for their info listeners.
	client_refs: HashMap<u32, ClientRef>,
	chans: Channels,
}

impl Inner {
	fn new(chans: Channels) -> Self {
		Self {
			nodes: HashMap::new(),
			clients: HashMap::new(),
			client_refs: HashMap::new(),
			chans,
		}
	}
	fn node_pid(&self, node: &NodeRef) -> Option<u32> {
		node.process_id.or_else(|| {
			node.client_id
				.and_then(|client_id| self.clients.get(&client_id).copied())
		})
	}
	fn refresh_apps(&self) {
		use std::collections::BTreeMap;

		let mut by_pid: BTreeMap<u32, (String, f32, u32, bool)> = BTreeMap::new();

		for n in self.nodes.values().filter(|n| n.is_stream) {
			let Some(app) = n.app_name.as_deref().filter(|s| !s.is_empty()) else {
				continue;
			};
			let Some(pid) = self.node_pid(n) else {
				continue;
			};

			let entry = by_pid.entry(pid).or_insert((app.to_owned(), 0.0, 0, true));
			entry.1 += n.volume_cubic;
			entry.2 += 1;
			entry.3 &= n.mute;
		}
		*self.chans.process_apps.lock().unwrap() = by_pid
			.into_iter()
			.map(|(pid, (name, sum, count, mute))| (pid, (name, sum / count as f32, mute)))
			.collect();

		self.bump();
	}

	fn bump(&self) {
		self.chans.notify.send_modify(|v| *v = v.wrapping_add(1));
	}

	fn reset(&mut self) {
		self.nodes.clear();
		self.clients.clear();
		self.client_refs.clear();
		self.refresh_apps();
	}
}
pub(super) fn run_loop(rx: pipewire::channel::Receiver<Command>, chans: Channels) -> Result<()> {
	use pipewire::context::ContextRc;
	use pipewire::main_loop::MainLoopRc;

	pipewire::init();

	let main_loop = MainLoopRc::new(None)?;
	let inner = Rc::new(RefCell::new(Inner::new(chans)));

	// --- Commands from actions, delivered inside this loop ---
	// Attached once for the thread's whole life: the receiver survives reconnects,
	// so commands keep flowing after the PipeWire daemon is restarted.
	let inner_c = inner.clone();
	let _recv = rx.attach(main_loop.loop_(), move |cmd| {
		on_command(&inner_c, cmd);
	});

	// Reconnect loop: `systemctl --user restart pipewire` (or any daemon crash)
	// severs the connection and fires a fatal core error, which quits the inner
	// `run()`. We then drop the dead connection, clear the stale object state, and
	// dial back in — rediscovering everything from the fresh registry.
	loop {
		let context = ContextRc::new(&main_loop, None)?;
		let core = match context.connect_rc(None) {
			Ok(core) => core,
			Err(error) => {
				log::warn!("PipeWire connect failed: {error}; retrying");
				std::thread::sleep(RECONNECT_DELAY);
				continue;
			}
		};
		// A ref-counted registry so the `global` callback below can bind new objects.
		let registry = core.get_registry_rc()?;

		// A fatal error on the core object (id 0) — most often the daemon going
		// away — is our disconnect signal: quit the loop so we reconnect below.
		let ml = main_loop.clone();
		let _core_listener = core
			.add_listener_local()
			.error(move |id, _seq, res, message| {
				log::warn!("core error (id {id}, res {res}): {message}");
				// `-ENOENT` on the core means we touched an object that had already
				// gone: a node removed mid-bind, or one the session manager hid from
				// us (WirePlumber's `hide-parent` revokes permissions on nodes we may
				// have seen a moment earlier). The connection itself is healthy, and
				// reconnecting would only replay the same race forever.
				const ENOENT: i32 = -2;
				if id == pipewire::core::PW_ID_CORE && res != ENOENT {
					ml.quit();
				}
			})
			.register();

		// --- Registry: discover application audio streams ---
		let reg_for_cb = registry.clone();
		let inner_g = inner.clone();
		let inner_r = inner.clone();
		let _registry_listener = registry
			.add_listener_local()
			.global(move |global| on_global(&inner_g, &reg_for_cb, global))
			.global_remove(move |id| on_global_remove(&inner_r, id))
			.register();

		log::info!("PipeWire backend connected; entering main loop");
		main_loop.run();

		// `run()` only returns once the core error quit it, i.e. the connection
		// dropped. Clear the stale nodes while their (dead) core
		// still exists, then let the connection objects drop and reconnect.
		inner.borrow_mut().reset();
		log::warn!("PipeWire connection lost; reconnecting in {RECONNECT_DELAY:?}");
		std::thread::sleep(RECONNECT_DELAY);
	}
}

fn on_global(
	inner: &Rc<RefCell<Inner>>,
	registry: &pipewire::registry::Registry,
	global: &pipewire::registry::GlobalObject<&pipewire::spa::utils::dict::DictRef>,
) {
	use pipewire::types::ObjectType;

	let Some(props) = global.props else { return };

	match global.type_ {
		ObjectType::Client => {
			let id = global.id;

			// Bind the client. Some applications (especially Wine/Proton games)
			// don't expose application.process.id in the registry announcement,
			// but do expose it later through the Client info event.
			let client: pipewire::client::Client = match registry.bind(global) {
				Ok(client) => client,
				Err(e) => {
					log::warn!("Failed to bind client {id}: {e}");
					return;
				}
			};

			let inner_info = inner.clone();

			let listener = client
				.add_listener_local()
				.info(move |info| {
					let Some(props) = info.props() else {
						return;
					};

					let process_id = props
						.get("application.process.id")
						.and_then(|s| s.parse::<u32>().ok());

					if let Some(pid) = process_id {
						let mut b = inner_info.borrow_mut();
						b.clients.insert(id, pid);
						b.refresh_apps();
					}
				})
				.register();

			// Seed the PID immediately when the registry already provides it.
			if let Some(pid) = props
				.get("application.process.id")
				.and_then(|s| s.parse::<u32>().ok())
			{
				inner.borrow_mut().clients.insert(id, pid);
			}

			inner.borrow_mut().client_refs.insert(
				id,
				ClientRef {
					_proxy: client,
					_listener: listener,
				},
			);

			inner.borrow().refresh_apps();
		}
		ObjectType::Node => {
			let media_class = props.get("media.class").unwrap_or("");
			let app_name: Option<String> = props.get("application.name").map(str::to_owned);
			let process_id = props
				.get("application.process.id")
				.and_then(|s| s.parse::<u32>().ok());
			let client_id = props.get("client.id").and_then(|s| s.parse::<u32>().ok());
			let is_stream = media_class == "Stream/Output/Audio" && app_name.is_some();

			// Bind the node so we can read its Props (volume/mute) and set them.
			let node: pipewire::node::Node = match registry.bind(global) {
				Ok(n) => n,
				Err(e) => {
					log::warn!("Failed to bind node {}: {e}", global.id);
					return;
				}
			};

			let id = global.id;
			let inner_param = inner.clone();
			let inner_info = inner.clone();
			let listener = node
				.add_listener_local()
				.info(move |info| {
					if let Some(props) = info.props() {
						let pid = props
							.get("application.process.id")
							.and_then(|s| s.parse::<u32>().ok());

						if let Some(n) = inner_info.borrow_mut().nodes.get_mut(&id)
							&& pid.is_some()
						{
							n.process_id = pid;
						}
					}
				})
				.param(move |_seq, param_type, _index, _next, pod| {
					if param_type == pipewire::spa::param::ParamType::Props
						&& let Some(pod) = pod
					{
						update_node_from_props(&inner_param, id, pod);
					}
				})
				.register();

			node.subscribe_params(&[pipewire::spa::param::ParamType::Props]);

			log::info!(
				"tracking stream {id} {} pid={:?}",
				app_name.as_deref().unwrap_or(""),
				process_id,
			);

			inner.borrow_mut().nodes.insert(
				id,
				NodeRef {
					is_stream,
					app_name,
					process_id,
					client_id,
					volume_cubic: 0.0,
					mute: false,
					proxy: node,
					_listener: listener,
				},
			);

			inner.borrow().refresh_apps();
		}
		_ => {}
	}
}

fn on_global_remove(inner: &Rc<RefCell<Inner>>, id: u32) {
	let mut b = inner.borrow_mut();
	b.nodes.remove(&id);
	b.clients.remove(&id);
	b.client_refs.remove(&id);
	b.refresh_apps();
}
fn on_command(inner: &Rc<RefCell<Inner>>, cmd: Command) {
	log::debug!("command {cmd:?}");

	match cmd {
		Command::AdjustAppPidsVolume(pids, delta) => {
			let b = inner.borrow();

			// Prefer the exact focused PID when it directly owns an audio stream.
			// If it does not, search the focused application's descendant PIDs.
			let exact_pid = pids.first().copied();

			let exact_exists = exact_pid.is_some_and(|pid| {
				b.nodes
					.values()
					.any(|n| n.is_stream && b.node_pid(n) == Some(pid))
			});

			let selected_pids: &[u32] = if exact_exists { &pids[..1] } else { &pids };

			let mut matched = 0;

			for n in b.nodes.values().filter(|n| {
				n.is_stream
					&& b.node_pid(n)
						.is_some_and(|pid| selected_pids.contains(&pid))
			}) {
				let target = (n.volume_cubic + delta).clamp(0.0, 1.5);
				set_node_props(&n.proxy, Some(cubic_to_linear(target)), None);
				matched += 1;
			}

			if matched == 0 {
				log::warn!("No active audio streams for candidate PIDs {pids:?}");
			}
		}

		Command::SetAppPidsMute(pids, value) => {
			let b = inner.borrow();

			let exact_pid = pids.first().copied();

			let exact_exists = exact_pid.is_some_and(|pid| {
				b.nodes
					.values()
					.any(|n| n.is_stream && b.node_pid(n) == Some(pid))
			});

			let selected_pids: &[u32] = if exact_exists { &pids[..1] } else { &pids };

			let streams: Vec<&NodeRef> = b
				.nodes
				.values()
				.filter(|n| {
					n.is_stream
						&& b.node_pid(n)
							.is_some_and(|pid| selected_pids.contains(&pid))
				})
				.collect();

			let currently_muted = !streams.is_empty() && streams.iter().all(|n| n.mute);

			let mute = value.unwrap_or(!currently_muted);

			for n in &streams {
				set_node_props(&n.proxy, None, Some(mute));
			}

			if streams.is_empty() {
				log::warn!("No active audio streams for candidate PIDs {pids:?}");
			}
		}
	}
}
/// Parse a Props POD coming from a node and cache channelVolumes + mute.
fn update_node_from_props(inner: &Rc<RefCell<Inner>>, id: u32, pod: &pipewire::spa::pod::Pod) {
	use pipewire::spa::pod::Value;
	use pipewire::spa::pod::deserialize::PodDeserializer;

	let Ok((_, value)) = PodDeserializer::deserialize_from::<Value>(pod.as_bytes()) else {
		return;
	};
	let Value::Object(obj) = value else { return };

	let mut linear_avg: Option<f32> = None;
	let mut mute: Option<bool> = None;
	for prop in obj.properties {
		match prop.key {
			pipewire::spa::sys::SPA_PROP_channelVolumes => {
				if let Value::ValueArray(pipewire::spa::pod::ValueArray::Float(vals)) = prop.value
					&& !vals.is_empty()
				{
					linear_avg = Some(vals.iter().sum::<f32>() / vals.len() as f32);
				}
			}
			pipewire::spa::sys::SPA_PROP_mute => {
				if let Value::Bool(m) = prop.value {
					mute = Some(m);
				}
			}
			_ => {}
		}
	}

	let mut b = inner.borrow_mut();
	if let Some(n) = b.nodes.get_mut(&id) {
		if let Some(l) = linear_avg {
			n.volume_cubic = linear_to_cubic(l);
		}
		if let Some(m) = mute {
			n.mute = m;
		}
	}
	drop(b);
	inner.borrow().refresh_apps();
}

fn cubic_to_linear(cubic: f32) -> f32 {
	cubic.powi(3)
}
fn linear_to_cubic(linear: f32) -> f32 {
	linear.cbrt()
}
