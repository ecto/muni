# SF Depot

San Francisco depot deployment at Founders Inc. Temporary setup using Jetson AGX Orin
as the compute platform (intended to become a rover, repurposed for depot during bringup).

## Hardware

| Component | Model | Notes |
|-----------|-------|-------|
| Compute | NVIDIA Jetson AGX Orin 64GB | Ubuntu 20.04, JetPack 5.x |
| Router | Ubiquiti UCG Max | UniFi gateway, cloud-managed |
| Display | HDMI monitor | Kiosk mode via Firefox |

## Network Topology

The depot acts as a NAT gateway, sharing the building WiFi connection to the UCG Max router:

```
Internet
    │
    ▼
Founders Guest WiFi (10.104.x.x)
    │
    │ wlan0
    ▼
┌─────────────────────────────────────────────────────────────────┐
│  Jetson AGX Orin (depot)                                        │
│                                                                 │
│  wlan0: 10.104.12.233 (DHCP from building)                     │
│  eth0:  192.168.2.1   (static, gateway for UCG)                │
│                                                                 │
│  Services:                                                      │
│    - NAT (iptables MASQUERADE wlan0 → eth0)                    │
│    - DHCP server (dnsmasq on eth0)                             │
│    - Docker stack (console, discovery, dispatch, etc.)         │
│    - Kiosk display (Firefox fullscreen)                        │
│                                                                 │
│  Tailscale: depot (100.71.209.42)                              │
└─────────────────────────────────────────────────────────────────┘
    │
    │ eth0 → UCG WAN
    ▼
┌─────────────────────────────────────────────────────────────────┐
│  UCG Max                                                        │
│                                                                 │
│  WAN:  192.168.2.121 (DHCP from depot)                         │
│  LAN:  192.168.1.1                                             │
│                                                                 │
│  UniFi cloud access enabled                                     │
└─────────────────────────────────────────────────────────────────┘
    │
    │ LAN ports
    ▼
  Local devices (rovers, laptops, etc.)
```

## Access

| Method | Address | Notes |
|--------|---------|-------|
| Tailscale SSH | `ssh depot@depot` | Primary access |
| Console UI | http://depot/ | Via Tailscale, no auth |
| Grafana | http://depot/grafana/ | Via console proxy |
| UniFi Console | https://unifi.ui.com | Cloud management |

## System Configuration

### NAT Gateway

IP forwarding enabled persistently:

```bash
# /etc/sysctl.d/99-ip-forward.conf
net.ipv4.ip_forward=1
```

iptables NAT rule restored on boot via systemd:

```bash
# /etc/iptables/rules.v4
*nat
:PREROUTING ACCEPT
:INPUT ACCEPT
:OUTPUT ACCEPT
:POSTROUTING ACCEPT
-A POSTROUTING -o wlan0 -j MASQUERADE
COMMIT
```

### DHCP Server (dnsmasq)

```bash
# /etc/dnsmasq.d/eth0-dhcp.conf
interface=eth0
bind-interfaces
dhcp-range=192.168.2.100,192.168.2.200,255.255.255.0,12h
dhcp-option=option:router,192.168.2.1
dhcp-option=option:dns-server,8.8.8.8,1.1.1.1
```

### Static IP on eth0

Configured via NetworkManager:

```bash
nmcli connection modify "Wired connection 1" \
  ipv4.method manual \
  ipv4.addresses 192.168.2.1/24 \
  ipv4.never-default yes
```

## Kiosk Mode

The depot display runs in unattended kiosk mode, showing the console web UI fullscreen
without any desktop chrome, screensaver, or screen lock.

### Components

| Component | Purpose | Config File |
|-----------|---------|-------------|
| GDM auto-login | Boot straight to desktop | `/etc/gdm3/custom.conf` |
| GNOME autostart | Launch kiosk script on login | `~/.config/autostart/kiosk.desktop` |
| Kiosk script | Configure display + launch Firefox | `~/kiosk.sh` |
| unclutter | Hide mouse cursor when idle | Installed via apt |

### GDM Auto-Login

Automatically logs in the `depot` user on boot, bypassing the login screen:

```ini
# /etc/gdm3/custom.conf
[daemon]
AutomaticLoginEnable = true
AutomaticLogin = depot
```

### Autostart Entry

GNOME runs this desktop entry after login, which executes the kiosk script:

```ini
# ~/.config/autostart/kiosk.desktop
[Desktop Entry]
Type=Application
Name=Depot Kiosk
Exec=/home/depot/kiosk.sh
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
```

### Kiosk Script

The main kiosk configuration script. Disables all screen blanking/locking at multiple
layers (GNOME and X11), hides the cursor, and launches Firefox fullscreen:

```bash
#!/bin/bash
# ~/kiosk.sh

# Wait for Docker services to start
sleep 10

# Disable GNOME screensaver and lock
gsettings set org.gnome.desktop.screensaver idle-activation-enabled false
gsettings set org.gnome.desktop.screensaver lock-enabled false
gsettings set org.gnome.desktop.screensaver ubuntu-lock-on-suspend false
gsettings set org.gnome.desktop.session idle-delay 0
gsettings set org.gnome.settings-daemon.plugins.power idle-dim false

# Disable X11 screen blanking and power management
xset s off
xset s noblank
xset -dpms

# Hide cursor after 3 seconds of inactivity
unclutter -idle 3 &

# Launch Firefox in kiosk mode
firefox --kiosk http://localhost &
```

### Screen Blanking Prevention

GNOME has multiple layers that can blank or lock the screen. All must be disabled:

| Layer | Setting | Purpose |
|-------|---------|---------|
| GNOME Screensaver | `idle-activation-enabled false` | Prevents screensaver overlay |
| GNOME Screensaver | `lock-enabled false` | Prevents screen lock |
| GNOME Screensaver | `ubuntu-lock-on-suspend false` | Prevents lock after suspend |
| GNOME Session | `idle-delay 0` | Disables idle detection entirely |
| GNOME Power | `idle-dim false` | Prevents screen dimming |
| X11 DPMS | `xset s off` | Disables X11 screen saver |
| X11 DPMS | `xset s noblank` | Prevents screen blanking |
| X11 DPMS | `xset -dpms` | Disables display power management |

### Exiting Kiosk Mode

If you need to access the desktop (e.g., for troubleshooting):

1. **Keyboard shortcut**: Press `Alt+F4` to close Firefox, revealing the desktop
2. **SSH**: Connect via `ssh depot@depot` and run commands remotely
3. **Kill Firefox**: `pkill firefox` via SSH, then use VNC or physical keyboard

To restart kiosk mode after exiting:

```bash
~/kiosk.sh
```

### Troubleshooting

**Screen goes blank despite settings:**
```bash
# Re-apply all settings
gsettings set org.gnome.desktop.screensaver idle-activation-enabled false
gsettings set org.gnome.desktop.screensaver lock-enabled false
gsettings set org.gnome.desktop.session idle-delay 0
gsettings set org.gnome.settings-daemon.plugins.power idle-dim false
xset s off && xset s noblank && xset -dpms
```

**Firefox not launching:**
```bash
# Check if Firefox is running
pgrep firefox

# Check kiosk script logs (runs at login, no persistent log)
# Manually test:
firefox --kiosk http://localhost &
```

**Console shows auth prompt:**
```bash
# Ensure CONSOLE_PASSWORD is empty in .env
grep CONSOLE_PASSWORD ~/depot/.env

# Force recreate container to pick up env change
cd ~/depot && docker compose up -d --force-recreate console
```

**Display not detected:**
```bash
# Check connected displays
xrandr

# HDMI may need to be connected before boot on Jetson
```

## Docker Stack

Services deployed via `~/depot/docker-compose.yml`:

| Service | Port | Status | Notes |
|---------|------|--------|-------|
| console | 80 | Running | Web UI, no auth |
| discovery | 4860 | Running | Rover registration |
| dispatch | 4890 | Running | Mission planning |
| influxdb | 8086 | Running | Metrics storage |
| postgres | 5432 | Running | Dispatch database |
| grafana | 3000 | Running | Dashboards |
| map-api | 4870 | Running | Map serving |
| mapper | - | Running | Map processing |
| sftp | 2222 | Stopped | x86-only image |

### Environment

Key settings in `~/depot/.env`:

```bash
DEPOT_NAME=sf
CONSOLE_PASSWORD=        # Empty = no auth
CONSOLE_USERNAME=admin
```

### Management Commands

```bash
# View logs
cd ~/depot && docker compose logs -f console

# Restart stack
cd ~/depot && docker compose restart

# Rebuild after code changes
cd ~/depot && docker compose up -d --build

# Force recreate (to pick up env changes)
cd ~/depot && docker compose up -d --force-recreate console
```

## Known Issues

1. **SFTP container fails** - `atmoz/sftp:alpine` is x86-only. Need ARM64 alternative or
   build from source. Non-critical for now.

2. **JetPack version** - Running JetPack 5.x (L4T 35.x). Upgrade to 6.x would require
   USB-A to USB-C data cable for recovery mode flashing.

3. **WiFi dependency** - Depot internet depends on "Founders Guest" WiFi. If WiFi goes
   down, depot loses internet but local Docker services continue running.

## Maintenance

### Updating Code

```bash
# From development machine
rsync -avz --exclude node_modules --exclude target \
  ~/Developer/muni/depot/ depot@depot:~/depot/

# On depot
cd ~/depot && docker compose up -d --build
```

### Checking Services

```bash
# Container status
docker ps

# Service health
curl -s http://localhost/health
curl -s http://localhost:4860/health
curl -s http://localhost:4890/health

# Network status
ip addr show wlan0
ip addr show eth0
tailscale status
```

### Rebooting

```bash
sudo reboot
```

After reboot:
- GDM auto-logs in depot user
- Kiosk script launches Firefox
- Docker services auto-start (restart: unless-stopped)
- NAT rules restored via systemd

## Future Improvements

- [ ] Replace Jetson with dedicated depot hardware (RPi5 or mini PC)
- [ ] Add RTK base station (ZED-F9P + antenna)
- [ ] Build ARM64 SFTP container
- [ ] Add UPS for power resilience
- [ ] Configure Tailscale Serve for external access
