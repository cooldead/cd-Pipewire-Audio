# CooldeadPipeWire

CooldeadPipeWire is a Linux PipeWire audio plugin for OpenDeck.

It is based on [OpenDeck PipeWire](https://github.com/sjourdois/opendeck-pipewire) by sjourdois, with additional features and fixes focused on per-application audio control and KDE Plasma Wayland integration.

## What CooldeadPipeWire adds

CooldeadPipeWire retains the original plugin's PipeWire functionality while adding the following changes.

### Active Application Volume

Added a new **Active Application Volume** action.

The action controls the volume of the application belonging to the currently focused window.

This makes it possible to use a Stream Deck encoder to control whichever application is currently active without creating a separate Stream Deck action for every application.

For example:

- Focus a game → encoder controls the game's volume
- Focus a browser → encoder controls the browser's volume
- Focus Discord → encoder controls Discord's volume

The action also supports application mute.

### KDE Plasma Wayland active-window integration

Added a KDE KWin script that reports the PID of the currently focused window to CooldeadPipeWire.

This is used by the Active Application Volume action to determine which PipeWire application stream should be controlled.

The integration uses a dedicated D-Bus interface:

```text
org.cooldeadpipewire.PipeWire.ActiveWindow
```

This feature is specifically intended for KDE Plasma Wayland.

### Proton / Wine application support

Active Application Volume works with applications running through Wine/Proton when their audio streams are exposed through PipeWire.

The implementation matches the focused application's process ID against PipeWire application streams rather than relying solely on the displayed application name.

This allows applications such as Windows games running through Proton/Wine to be controlled.

### Output Device icon reliability

Fixed an issue with the Output Device action's custom icon picker.

The original implementation created the file input dynamically without attaching it to the document before opening the file picker. On some systems this caused the file selection callback to fail intermittently.

CooldeadPipeWire attaches the file input to the document before opening the picker, making custom output-device icons reliably selectable.

### Output Device icon persistence

Fixed custom output-device icons being lost when Output Device settings were subsequently saved or when switching between devices.

Custom icons are now included when the action's settings are saved.

### CooldeadPipeWire branding

The fork has its own plugin identity and namespace so it can coexist with the original OpenDeck PipeWire plugin.

The plugin uses:

```text
com.cooldeadpipewire.sdPlugin
```

and actions use the:

```text
com.cooldeadpipewire.pipewire.*
```

namespace.

---

## Installation

### Requirements

CooldeadPipeWire currently targets Linux systems using:

- PipeWire
- WirePlumber
- OpenDeck 7.x
- KDE Plasma Wayland for Active Application Volume

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
cp -r dist ~/.config/opendeck/plugins/com.cooldeadpipewire.sdPlugin
```

### Install the KDE active-window bridge

For KDE Plasma Wayland:

```bash
./install-cooldeadpipewire-active-window.sh
```

Restart OpenDeck after installation.

The CooldeadPipeWire actions should now appear in OpenDeck.

---

## Active Application Volume

Add the **Active Application Volume** action to a Stream Deck key or encoder.

When the action is active:

1. CooldeadPipeWire receives the PID of the currently focused KDE window.
2. The plugin looks for PipeWire audio streams belonging to that PID.
3. Turning the encoder adjusts the application's PipeWire volume.
4. Pressing the encoder can mute/unmute the application.

The application does not need to be manually selected.

### Proton and Wine games

For Proton/Wine games, the process shown by PipeWire may be different from the executable visible in the game launcher.

For example, a game may appear as:

```text
application.name = "Schedule I.exe"
application.process.binary = "wine64-preloader"
```

CooldeadPipeWire uses the process relationship rather than requiring the PipeWire application name to exactly match the focused window title.

---

## Updating

From the repository directory:

```bash
git pull
```

Then rebuild and reinstall:

```bash
deno run -A build.ts dist x86_64-unknown-linux-gnu
rm -rf ~/.config/opendeck/plugins/com.cooldeadpipewire.sdPlugin
cp -r dist ~/.config/opendeck/plugins/com.cooldeadpipewire.sdPlugin
```

Restart OpenDeck after updating.

The KDE active-window bridge normally does not need to be reinstalled unless its script has changed.

---

## Uninstalling

Remove the OpenDeck plugin:

```bash
rm -rf ~/.config/opendeck/plugins/com.cooldeadpipewire.sdPlugin
```

Remove the KDE active-window bridge:

```bash
rm -rf ~/.local/share/kwin/scripts/cooldeadpipewire-active-window
```

Restart OpenDeck and KDE if necessary.

---

## AI-assisted development disclosure

This fork contains code and documentation developed with assistance from OpenAI's ChatGPT.

AI assistance was used for portions of the development process, including code changes, debugging, troubleshooting, documentation, and development guidance.

The project maintainer reviewed, tested, and integrated the resulting changes. In particular, the Active Application Volume functionality, KDE active-window integration, PipeWire process matching, and Output Device icon fixes were tested in the author's Linux/KDE environment.

AI assistance does not imply that the original OpenDeck PipeWire project's author or contributors were involved in, reviewed, or endorsed these changes.

CooldeadPipeWire is an independent fork of the original project.

## Credits

CooldeadPipeWire is based on:

**OpenDeck PipeWire**  
https://github.com/sjourdois/opendeck-pipewire

Original project by **sjourdois**.

Please refer to the original project for its license and upstream history.
