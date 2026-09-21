print("CooldeadPipeWire Active Window script loaded");

const SERVICE = "org.cooldeadpipewire.PipeWire";
const PATH = "/org/cooldeadpipewire/PipeWire";
const INTERFACE = "org.cooldeadpipewire.PipeWire.ActiveWindow";

function report(window) {
    const pid = window ? (window.pid || 0) : 0;
    print("CooldeadPipeWire Active Window: PID = " + pid);
    callDBus(SERVICE, PATH, INTERFACE, "SetActivePid", String(pid));
}

workspace.windowActivated.connect(report);
report(workspace.activeWindow);
