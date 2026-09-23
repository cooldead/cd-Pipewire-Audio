# CD-Active App Volume for KDE

CD-Active App Volume for KDE is a Linux PipeWire audio plugin for OpenDeck that controls the volume of the application currently focused on KDE Plasma Wayland.

It is based on [OpenDeck PipeWire](https://github.com/sjourdois/opendeck-pipewire) by sjourdois, with CD-Active App Volume focused on automatic per-application audio control and KDE Plasma Wayland integration.

> [!IMPORTANT]
> ## KDE ACTIVE-WINDOW BRIDGE REQUIRED
> Installing the OpenDeck plugin ZIP **is not enough by itself**. You must also install the **KDE active-window bridge** or CD-Active App Volume cannot determine which application is currently focused.
>
> After installing the plugin ZIP through OpenDeck, follow the **Install the KDE active-window bridge** section below.

## CD-Active App Volume

CD-Active App Volume provides a single OpenDeck action for controlling the audio of the currently focused application. There is no need to manually select an application before adjusting its volume.

The plugin starts with the PID reported for the focused KDE/Wayland window and matches it to the corresponding PipeWire output stream. It can also follow related processes when the window and audio stream are owned by different processes.

This expanded matching improves support for:

- Native Linux applications
- Multi-process Chromium-based applications such as Brave and Chromium
- Applications whose focused-window PID differs from their PipeWire audio PID
- Steam games running through Wine/Proton
- Games and applications that expose their PipeWire process information through the owning PipeWire client

Compatibility can still vary between applications and Wine/Proton configurations, but the plugin is no longer limited to games whose focused PID directly owns the audio stream.

### Encoder controls

When used with an encoder:

- **Rotate:** Adjusts the focused application's volume.
- **Press:** Performs the configured press action.

The press action can be configured to:

- **Volume up**
- **Volume down**
- **Toggle mute**

The volume adjustment step can be configured from **1% to 20%**.

### Custom encoder display

CD-Active App Volume does not use the standard OpenDeck volume bar.

Instead, the encoder displays a custom half-circle volume gauge with the focused application's title above it and the current volume percentage below it. The display updates as the focused application or its PipeWire volume/mute state changes.

The gauge uses the following ranges:

| Volume | Gauge |
|---|---|
| 0–40% | Red |
| 40–80% | Yellow |
| 80–110% | Green |
| 110–130% | Yellow |
| 130–150% | Red |

The display allows volume levels above 100% to be shown when PipeWire permits the application stream to be amplified.

When no usable focused application/audio stream is found, the display shows **No Focused App** and the gauge is grey.

When the focused application is muted, the gauge becomes grey and **MUTED** is displayed.

The muted-state text uses the configurable **Muted color** setting from the Property Inspector.

## KDE Plasma Wayland integration

CD-Active App Volume for KDE uses a KDE KWin active-window bridge to report the PID of the currently focused window to the plugin.

The plugin uses that PID, related processes, and PipeWire client information to determine which PipeWire application stream should be controlled.

The integration uses a dedicated D-Bus interface:

```text
org.cooldeadpipewire.PipeWire.ActiveWindow
```

This feature is specifically intended for KDE Plasma Wayland.

> [!IMPORTANT]
> **The KDE active-window bridge is required.** Installing the OpenDeck plugin ZIP does **not** install the bridge. Install it separately using the instructions below before using Active Application Volume.

## Proton / Wine game support

CD-Active App Volume supports many games running through Wine/Proton when their audio streams are exposed through PipeWire.

For Proton/Wine games, the process associated with the focused game window and the process reported by PipeWire are not always identical. Some streams also expose the useful OS process ID on their owning PipeWire client rather than directly on the stream node.

CD-Active App Volume accounts for these cases by matching the focused process and related processes against the PipeWire stream/client relationship instead of requiring the PipeWire application name or direct PID to exactly match the focused window.

This substantially improves support for Windows games running through Steam/Proton, although compatibility may still vary by game and compatibility-tool configuration.

## Installation

### Requirements

CD-Active App Volume for KDE currently targets Linux systems using:

- PipeWire
- WirePlumber
- OpenDeck 7.x
- KDE Plasma Wayland

For KDE Plasma Wayland, `qdbus6` is also required for installing/reloading the active-window bridge.

### Recommended: install the release ZIP through OpenDeck

1. Download **`CD-Active-App-Volume-for-KDE.sdPlugin.zip`** from the latest GitHub release.
2. Open **OpenDeck**.
3. Use OpenDeck's plugin installation/import option to install the downloaded `.sdPlugin.zip` file.
4. Restart OpenDeck if it does not reload the plugin automatically.
5. **REQUIRED: Install the KDE active-window bridge using the instructions immediately below.** The ZIP only installs the OpenDeck plugin; without the bridge, Active Application Volume cannot know which KDE window is focused.

After both components are installed, add the **CD-Active App Volume** action to an encoder in OpenDeck.

> [!IMPORTANT]
> ## REQUIRED: Install the KDE active-window bridge
> **Do not skip this step.** The active-window bridge is required even when you install the plugin using the release ZIP.

The active-window bridge is part of this repository, not the OpenDeck ZIP installation. Clone or download the repository, then run the installer from the repository directory:

```bash
git clone https://github.com/cooldead/cd-Pipewire-Audio.git
cd cd-Pipewire-Audio
./install-cooldeadpipewire-active-window.sh
```

The installer:

1. Installs the CD-Active App Volume for KDE KWin script.
2. Enables the script in KDE.
3. Reloads KWin so the bridge becomes active immediately.

The bridge is registered as a KDE KWin script and should automatically load when KDE starts. It does not need to be reinstalled after every reboot.

### Arch Linux / CachyOS build dependencies

If you want to build the plugin yourself instead of using the release ZIP:

```bash
sudo pacman -S --needed pipewire wireplumber rust deno qt6-tools
```

### Build and install from source

Clone the repository:

```bash
git clone https://github.com/cooldead/cd-Pipewire-Audio.git
cd cd-Pipewire-Audio
```

Build the plugin:

```bash
deno run -A build.ts dist x86_64-unknown-linux-gnu
```

Install it into OpenDeck:

```bash
rm -rf ~/.config/opendeck/plugins/com.cooldeadpipewire.sdPlugin
cp -r dist/. ~/.config/opendeck/plugins/com.cooldeadpipewire.sdPlugin
```

Then install the active-window bridge:

```bash
./install-cooldeadpipewire-active-window.sh
```

Restart OpenDeck after installing from source so it loads the new plugin build.

## Updating

### Release ZIP users

Download the new `CD-Active-App-Volume-for-KDE.sdPlugin.zip` from the latest release and install/import it through OpenDeck again.

The KDE active-window bridge normally does not need to be reinstalled unless the bridge script itself has changed. Check the release notes when updating.

### Source users

From the repository directory:

```bash
git pull
```

Then rebuild and reinstall:

```bash
deno run -A build.ts dist x86_64-unknown-linux-gnu
rm -rf ~/.config/opendeck/plugins/com.cooldeadpipewire.sdPlugin
cp -r dist/. ~/.config/opendeck/plugins/com.cooldeadpipewire.sdPlugin
```

Restart OpenDeck after updating so that it loads the new plugin build.

If the KWin bridge itself is updated, run:

```bash
./install-cooldeadpipewire-active-window.sh
```

## Uninstalling

Remove the OpenDeck plugin through OpenDeck's plugin management UI, or remove the plugin directory manually:

```bash
rm -rf ~/.config/opendeck/plugins/com.cooldeadpipewire.sdPlugin
```

Remove the KDE active-window bridge:

```bash
kpackagetool6 --type=KWin/Script --remove cooldeadpipewire-active-window
```

Disable the bridge in KDE:

```bash
kwriteconfig6 --file ~/.config/kwinrc --group Plugins --key cooldeadpipewire-active-windowEnabled false
```

Restart KDE if necessary.

## AI-assisted development disclosure

This fork contains code and documentation developed with assistance from OpenAI's ChatGPT.

AI assistance was used for portions of the development process, including code changes, debugging, troubleshooting, documentation, and development guidance.

The project maintainer reviewed, tested, and integrated the resulting changes. In particular, the CD-Active App Volume functionality, KDE active-window integration, PipeWire process matching, and custom encoder display were tested in the author's Linux/KDE environment.

AI assistance does not imply that the original OpenDeck PipeWire project's author or contributors were involved in, reviewed, or endorsed these changes.

CD-Active App Volume for KDE is an independent fork of the original project.

## Credits

CD-Active App Volume for KDE is based on:

**OpenDeck PipeWire**  
https://github.com/sjourdois/opendeck-pipewire

Original project by **sjourdois**.

Please refer to the original project for its license and upstream history.
