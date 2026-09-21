use openaction::*;
use serde::{Deserialize, Serialize};
use crate::active_window::ActiveWindowHandle;
use crate::color::BarColors;
use crate::command::Command;
use crate::pw::PwHandle;
#[derive(Serialize,Deserialize,Clone)]
#[serde(default)]
pub struct ActiveAppVolumeSettings { pub step:u8, pub mode:super::KeyMode, #[serde(flatten)] pub colors:BarColors }
impl Default for ActiveAppVolumeSettings { fn default()->Self{Self{step:5,mode:super::KeyMode::Up,colors:BarColors::default()}} }
pub struct ActiveAppVolumeAction { pub pw:PwHandle, pub active:ActiveWindowHandle, pub refresher:crate::refresh::Refresher }
#[async_trait]
impl Action for ActiveAppVolumeAction {
	const UUID:&'static str=super::action_uuid!("activeappvolume"); type Settings=ActiveAppVolumeSettings;
	async fn key_down(&self,i:&Instance,s:&Self::Settings)->OpenActionResult<()>{log::info!("ActiveAppVolume: key_down instance={} step={}",i.instance_id,s.step);let d=super::step_fraction(s.step);match s.mode{super::KeyMode::Up=>self.adjust(i,s,d).await,super::KeyMode::Down=>self.adjust(i,s,-d).await,super::KeyMode::Mute=>self.toggle_mute(i,s).await}}
	async fn dial_rotate(&self,i:&Instance,s:&Self::Settings,t:i16,pressed:bool)->OpenActionResult<()>{log::info!("ActiveAppVolume: dial_rotate instance={} ticks={} pressed={} pid={}",i.instance_id,t,pressed,self.active.pid());self.adjust(i,s,t as f32*super::step_fraction(s.step)).await}
	async fn dial_down(&self,i:&Instance,s:&Self::Settings)->OpenActionResult<()>{log::info!("ActiveAppVolume: dial_down instance={} pid={}",i.instance_id,self.active.pid());self.toggle_mute(i,s).await}
	async fn touch_tap(&self,i:&Instance,s:&Self::Settings,_:(u16,u16),_:bool)->OpenActionResult<()>{log::info!("ActiveAppVolume: touch_tap instance={} pid={}",i.instance_id,self.active.pid());self.toggle_mute(i,s).await}
	async fn will_appear(&self,i:&Instance,s:&Self::Settings)->OpenActionResult<()>{log::info!("ActiveAppVolume: will_appear instance={} pid={}",i.instance_id,self.active.pid());self.refresher.set_active_colors(&i.instance_id,&s.colors);self.render(i,s).await}
	async fn will_disappear(&self,i:&Instance,_s:&Self::Settings)->OpenActionResult<()>{self.refresher.forget_active_colors(&i.instance_id);Ok(())}
	async fn did_receive_settings(&self,i:&Instance,s:&Self::Settings)->OpenActionResult<()>{self.refresher.set_active_colors(&i.instance_id,&s.colors);self.render(i,s).await}
}
impl ActiveAppVolumeAction {
	fn state(&self)->Option<(u32,String,f32,bool)>{let pid=self.active.pid();if pid==0{return None;}self.pw.app_pid_state(pid).map(|(n,v,m)|(pid,n,v,m))}

	async fn adjust(&self,i:&Instance,s:&ActiveAppVolumeSettings,d:f32)->OpenActionResult<()>{
		let pid=self.active.pid();
		if pid==0{return self.render(i,s).await;}

		// Do not gate the command on the cached process_apps state. The cache is
		// only a UI snapshot and can briefly lag behind PipeWire discovery (in
		// particular when a Wine/Proton process starts). The backend already has
		// the authoritative live NodeRefs and matches the PID directly.
		if let Some((_pid,name,cur,mute))=self.state(){
			if mute{self.pw.send(Command::SetAppPidMute(pid,Some(false)));}
			self.pw.send(Command::AdjustAppPidVolume(pid,d));
			return crate::display::app(i,Some(&name),(cur+d).clamp(0.0,1.5),false,&s.colors).await;
		}

		self.pw.send(Command::AdjustAppPidVolume(pid,d));
		self.render(i,s).await
	}

	async fn toggle_mute(&self,i:&Instance,s:&ActiveAppVolumeSettings)->OpenActionResult<()>{
		let pid=self.active.pid();
		if pid==0{return self.render(i,s).await;}

		// As with volume adjustment, always send the PID command. The cached
		// process state is not a prerequisite for controlling the live stream.
		if let Some((_pid,name,vol,mute))=self.state(){
			self.pw.send(Command::SetAppPidMute(pid,None));
			return crate::display::app(i,Some(&name),vol,!mute,&s.colors).await;
		}

		self.pw.send(Command::SetAppPidMute(pid,None));
		self.render(i,s).await
	}
	async fn render(&self,i:&Instance,s:&ActiveAppVolumeSettings)->OpenActionResult<()>{match self.state(){Some((_p,n,v,m))=>crate::display::app(i,Some(&n),v,m,&s.colors).await,None=>crate::display::app(i,None,0.0,false,&s.colors).await}}
}
