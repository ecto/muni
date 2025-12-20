# Onboard Display

The BVR can optionally connect a 7" HDMI display to the Jetson for local status monitoring. This displays the web dashboard in kiosk mode, providing at-a-glance telemetry without requiring remote access.

> **Note:** The display is always internal-facing, intended for operational and debugging purposes during service. It is not visible from outside the shell. For external status indication, see [Status LEDs](#status-leds) below.

## What's Displayed

The dashboard shows real-time rover status:

- **Mode** — Current state (Disabled, Idle, Teleop, Autonomous, E-Stop, Fault)
- **Battery** — Voltage, current draw, charge level bar
- **Velocity** — Linear and angular velocity
- **Motor Temps** — Temperature for all 4 drive motors
- **Tool Status** — Active tool name, position, current draw

The dashboard auto-refreshes at 5 Hz and indicates connection status.

## Hardware

| Part               | Source | Notes                       |
| ------------------ | ------ | --------------------------- |
| 7" HDMI display    | Amazon | 1024×600 or 800×480         |
| Micro HDMI adapter | Amazon | Jetson Orin uses micro HDMI |
| USB power cable    | —      | Power display from 5V rail  |

Mount the display where it's visible during field service but protected from impact.

## Software Setup

### Prerequisites

- Ubuntu Desktop installed (not Server)
- `bvrd.service` enabled and running
- User account for auto-login (e.g., `cam`)
- Firefox installed (comes with Ubuntu Desktop)

### 1. Configure Auto-Login

Edit GDM configuration:

```bash
sudo nano /etc/gdm3/custom.conf
```

Set the `[daemon]` section to:

```ini
[daemon]
WaylandEnable=false
AutomaticLoginEnable=true
AutomaticLogin=cam
```

### 2. Create Autostart Entry

We use a desktop autostart file instead of a systemd service. Systemd services
can't easily access the X display session due to authentication requirements.

```bash
# Create autostart directory
mkdir -p ~/.config/autostart

# Create autostart entry (includes screen blanking prevention)
cat > ~/.config/autostart/bvr-kiosk.desktop << 'EOF'
[Desktop Entry]
Type=Application
Name=BVR Dashboard Kiosk
Exec=bash -c 'xset s off && xset -dpms && xset s noblank && sleep 10 && firefox --kiosk http://localhost:8080'
X-GNOME-Autostart-enabled=true
EOF
```

The `xset` commands disable:

- `s off` — screen saver
- `-dpms` — display power management (prevents standby/suspend/off)
- `s noblank` — screen blanking

### 3. Disable Screen Blanking

Prevent the display from sleeping:

```bash
# Disable screen blanking
gsettings set org.gnome.desktop.screensaver lock-enabled false
gsettings set org.gnome.desktop.session idle-delay 0

# Disable power management
gsettings set org.gnome.settings-daemon.plugins.power sleep-inactive-ac-type 'nothing'
```

### 4. Reboot and Verify

```bash
sudo reboot
```

After reboot, the display should:

1. GDM auto-logs in as `cam`
2. GNOME desktop starts
3. After 10 seconds, Firefox launches in kiosk mode showing the dashboard

## Troubleshooting

### Display stays black

- Check HDMI connection and adapter
- Verify display is powered (USB or 12V depending on model)
- Check if GDM is running: `systemctl status gdm`

### Still showing login screen

Auto-login is not configured. Verify `/etc/gdm3/custom.conf` has:

```ini
[daemon]
AutomaticLoginEnable=true
AutomaticLogin=cam
```

### Firefox doesn't start

Check if the autostart file exists:

```bash
cat ~/.config/autostart/bvr-kiosk.desktop
```

Check GNOME autostart is working:

```bash
# View running processes after login
ps aux | grep firefox
```

### Dashboard shows "Disconnected"

- Verify bvrd is running: `systemctl status bvrd`
- Check dashboard port: `curl http://localhost:8080`
- Review bvrd logs: `journalctl -u bvrd -f`

### Firefox shows session restore dialog

Clear the previous session:

```bash
rm -rf ~/.mozilla/firefox/*.default*/sessionstore*
```

### Wrong display resolution

Create or edit `/etc/X11/xorg.conf.d/10-monitor.conf`:

```
Section "Monitor"
    Identifier "HDMI-1"
    Modeline "1024x600_60" 49.00 1024 1072 1168 1312 600 603 613 624 -hsync +vsync
    Option "PreferredMode" "1024x600_60"
EndSection
```

## Why Not a Systemd Service?

A systemd service (`kiosk.service`) seems cleaner but doesn't work reliably because:

1. System services run before the user's graphical session starts
2. They can't access the X display without the user's `XAUTHORITY` token
3. The `XAUTHORITY` path varies and may not exist at service start time

The autostart desktop file runs within the user's GNOME session, so it
automatically has display access.

## Remote Setup via SSH

To configure the kiosk on a new rover via SSH:

```bash
ssh cam@<rover-hostname>

# 1. Configure auto-login
sudo sed -i '/^\[daemon\]/a AutomaticLoginEnable=true\nAutomaticLogin=cam' /etc/gdm3/custom.conf

# 2. Create autostart entry (includes screen blanking prevention)
mkdir -p ~/.config/autostart
cat > ~/.config/autostart/bvr-kiosk.desktop << 'EOF'
[Desktop Entry]
Type=Application
Name=BVR Dashboard Kiosk
Exec=bash -c 'xset s off && xset -dpms && xset s noblank && sleep 10 && firefox --kiosk http://localhost:8080'
X-GNOME-Autostart-enabled=true
EOF

# 3. Reboot
sudo reboot
```

---

## Status LEDs

For external status indication, a 12V addressable LED strip is mounted around the base of the rover shell. This provides at-a-glance status visible from any angle without requiring a window or external display.

### Hardware

| Part                  | Source | Notes                          |
| --------------------- | ------ | ------------------------------ |
| 12V addressable strip | Amazon | WS2811 or similar, IP65+ rated |
| Level shifter         | —      | 3.3V GPIO → 5V/12V data signal |
| Power                 | —      | Tapped from 12V rail           |

### Status Patterns

| State       | Pattern                        |
| ----------- | ------------------------------ |
| Boot        | Chase animation (white)        |
| Idle        | Slow pulse (green)             |
| Teleop      | Solid (blue)                   |
| Autonomous  | Solid (purple)                 |
| E-Stop      | Fast blink (red)               |
| Fault       | Alternating blink (red/yellow) |
| Low Battery | Slow blink (orange)            |

The LED strip is controlled directly by `bvrd` via GPIO, independent of the optional internal display.
