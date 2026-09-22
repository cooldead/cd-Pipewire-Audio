use std::sync::{
	Arc,
	atomic::{AtomicU32, Ordering},
};
use tokio::sync::watch;
use zbus::interface;
const SERVICE: &str = "org.cooldeadpipewire.PipeWire";
const PATH: &str = "/org/cooldeadpipewire/PipeWire";
#[derive(Clone)]
pub struct ActiveWindowHandle {
	pid: Arc<AtomicU32>,
	notify: watch::Sender<u64>,
}
impl ActiveWindowHandle {
	pub fn pid(&self) -> u32 {
		self.pid.load(Ordering::Acquire)
	}
	pub fn subscribe(&self) -> watch::Receiver<u64> {
		self.notify.subscribe()
	}
}
struct ActiveWindowService {
	pid: Arc<AtomicU32>,
	notify: watch::Sender<u64>,
}
#[interface(name = "org.cooldeadpipewire.PipeWire.ActiveWindow")]
impl ActiveWindowService {
	async fn set_active_pid(&self, pid: &str) {
		let parsed = pid.parse::<u32>().unwrap_or(0);
		self.pid.store(parsed, Ordering::Release);
		self.notify.send_modify(|v| *v = v.wrapping_add(1));
	}
	#[zbus(property)]
	fn active_pid(&self) -> u32 {
		self.pid.load(Ordering::Acquire)
	}
}
pub fn start() -> ActiveWindowHandle {
	let pid = Arc::new(AtomicU32::new(0));
	let (notify, _) = watch::channel(0u64);
	let handle = ActiveWindowHandle {
		pid: pid.clone(),
		notify: notify.clone(),
	};
	tokio::spawn(async move {
		let connection = match zbus::Connection::session().await {
			Ok(c) => c,
			Err(e) => {
				log::warn!("Could not connect to the KDE session bus: {e}");
				return;
			}
		};
		if let Err(e) = connection.request_name(SERVICE).await {
			log::warn!("Could not own {SERVICE}: {e}");
			return;
		}
		if let Err(e) = connection
			.object_server()
			.at(PATH, ActiveWindowService { pid, notify })
			.await
		{
			log::warn!("Could not register KWin active-window D-Bus object: {e}");
			return;
		}
		std::future::pending::<()>().await;
	});
	handle
}
