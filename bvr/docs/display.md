# Onboard Display

The BVR can optionally connect a 7" HDMI display to the Jetson for local status monitoring. This displays the web dashboard in kiosk mode, providing at-a-glance telemetry without requiring remote access.

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

### 1. Configure Auto-Login

Edit GDM configuration:

```bash
sudo nano /etc/gdm3/custom.conf
```

Add under `[daemon]`:

```ini
AutomaticLoginEnable=true
AutomaticLogin=cam
```

### 2. Install Kiosk Service

Copy the service file:

```bash
sudo cp /etc/bvr/kiosk.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable kiosk
```

The service file (`firmware/config/kiosk.service`):

```ini
[Unit]
Description=BVR Dashboard Kiosk
After=bvrd.service graphical.target
Wants=bvrd.service

[Service]
Type=simple
User=cam
Environment=DISPLAY=:0
ExecStartPre=/bin/sleep 5
ExecStart=/usr/bin/firefox --kiosk http://localhost:8080
Restart=on-failure
RestartSec=5

[Install]
WantedBy=graphical.target
```

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

1. Auto-login to desktop
2. Wait for `bvrd` to start
3. Launch Firefox in fullscreen kiosk mode showing the dashboard

## Troubleshooting

### Display stays black

- Check HDMI connection and adapter
- Verify display is powered (USB or 12V depending on model)
- Check `journalctl -u kiosk` for errors

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

## Optional: Wayland

If using Wayland instead of X11, modify the kiosk service:

```ini
Environment=MOZ_ENABLE_WAYLAND=1
ExecStart=/usr/bin/firefox --kiosk http://localhost:8080
```

## Deploy Script Integration

The kiosk service is installed automatically when using:

```bash
./deploy.sh frog-0 --all
```

Or manually:

```bash
./deploy.sh frog-0 --services
```

