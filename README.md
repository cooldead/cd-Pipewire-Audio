# CD-Active App Volume for KDE

CD-Active App Volume for KDE is a Linux PipeWire audio plugin for OpenDeck designed to control the audio volume of the currently focused game.

It is based on [OpenDeck PipeWire](https://github.com/sjourdois/opendeck-pipewire) by sjourdois, with the CD-Active App Volume functionality focused on per-game audio control and KDE Plasma Wayland integration.

## CD-Active App Volume

CD-Active App Volume provides a single OpenDeck action for controlling the audio of the currently focused game.

The action uses the PID of the focused KDE/Wayland window to identify the corresponding PipeWire audio stream. The game does not need to be manually selected.

### Encoder controls

When used with an encoder:

- **Rotate:** Adjusts the focused game's volume.
- **Press:** Performs the configured press action.

The press action can be configured to:

- **Volume up**
- **Volume down**
- **Toggle mute**

The volume adjustment step can be configured from **1% to 20%**.

### Custom encoder display

CD-Active App Volume does not use the standard OpenDeck volume bar.

Instead, the encoder displays a custom half-circle volume gauge with the focused game's title above it and the current volume percentage below it.

The gauge uses the following ranges:

| Volume | Gauge |
|---|---|
| 0–40% | Red |
| 40–80% | Yellow |
| 80–110% | Green |
| 110–130% | Yellow |
| 130–150% | Red |

The display allows volume levels above 100% to be shown when PipeWire permits the application stream to be amplified.

When no usable focused game/audio stream is found, the display shows **No Focused App** and the gauge is grey.

When the focused game is muted, the gauge becomes grey and **MUTED** is displayed.

The muted-state text uses the configurable **Muted color** setting from the Property Inspector.

### Current limitation

The current implementation is focused on games.

It is designed to control the audio stream associated with the currently focused game window under KDE Plasma Wayland. General desktop applications such as web browsers or Discord are not currently the target of this plugin and compatibility with them is not guaranteed.

## KDE Plasma Wayland integration

CD-Active App Volume for KDE uses a KDE KWin script to report the PID of the currently focused window to the plugin.

The plugin uses that PID to determine which PipeWire application stream should be controlled.

The integration uses a dedicated D-Bus interface:

```text
org.cooldeadpipewire.PipeWire.ActiveWindow
```

This feature is specifically intended for KDE Plasma Wayland.

## Proton / Wine game support

CD-Active App Volume works with games running through Wine/Proton when their audio streams are exposed through PipeWire.

For Proton/Wine games, the process shown by PipeWire may be different from the executable visible in the game launcher.

For example, a game may appear as:

```text
application.name = "Schedule I.exe"
application.process.binary = "wine64-preloader"
```

CD-Active App Volume uses the process relationship rather than requiring the PipeWire application name to exactly match the focused window title.

This allows supported Windows games running through Proton/Wine to be controlled based on the currently focused game window.

## Installation

### Requirements

CD-Active App Volume for KDE currently targets Linux systems using:

- PipeWire
- WirePlumber
- OpenDeck 7.x
- KDE Plasma Wayland

For KDE Plasma Wayland, `qdbus6` is also required for installing the active-window bridge.

### Arch Linux / CachyOS

Install the required packages:

```bash
sudo pacman -S --needed pipewire wireplumber rust deno qt6-tools
```

### Build and install

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

### Install the KDE active-window bridge

For KDE Plasma Wayland:

```bash
./install-cooldeadpipewire-active-window.sh
```

The installer:

1. Installs the CD-Active App Volume for KDE KWin script.
2. Enables the script in KDE.
3. Reloads KWin so the bridge becomes active immediately.

The bridge is registered as a KDE KWin script and should automatically load when KDE starts. It does not need to be reinstalled after every reboot.

### Restart OpenDeck

**Restart OpenDeck after installing the plugin.**

OpenDeck needs to be restarted so that it loads the newly installed plugin.

The CD-Active App Volume action should then appear in OpenDeck.

## Updating

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

**Restart OpenDeck after updating** so that it loads the new plugin build.

The KDE active-window bridge normally does not need to be reinstalled unless its script has changed.

If the KWin bridge itself is updated, run:

```bash
./install-cooldeadpipewire-active-window.sh
```

## Uninstalling

Remove the OpenDeck plugin:

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
