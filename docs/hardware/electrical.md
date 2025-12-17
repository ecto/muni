# Electrical System

## Power Distribution

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            48V Battery Pack                                  │
│                         (13S LiPo, 39-54.6V)                                │
└───────────────────────────────────┬─────────────────────────────────────────┘
                                    │
                              100A Main Fuse
                                    │
         ┌──────────────────────────┼──────────────────────────────┐
         │                          │                              │
    ┌────┴────┐              ┌─────┴─────┐                  ┌─────┴─────┐
    │ VESCs   │              │  E-Stop   │                  │   DCDC    │
    │ (×4)    │              │  Contactor│                  │  48→12V   │
    └─────────┘              └───────────┘                  └─────┬─────┘
                                                                  │
                                                           ┌──────┴──────┐
                                                           │             │
                                                      ┌────┴────┐  ┌────┴────┐
                                                      │ Jetson  │  │  Tools  │
                                                      │ (12V)   │  │  (12V)  │
                                                      └─────────┘  └─────────┘
```

## Main Components

| Component | Spec                      | Notes                       |
| --------- | ------------------------- | --------------------------- |
| Battery   | 13S4P Li-ion, 48V 20Ah    | With BMS                    |
| Main Fuse | 100A ANL                  | At battery positive         |
| E-Stop    | Normally closed contactor | Cuts 48V to VESCs           |
| DCDC      | 48V→12V, 20A              | Powers Jetson + accessories |
| VESCs     | 4× VESC 6 or similar      | 60A continuous each         |

## Wiring

### CAN Bus

```
Jetson ─── VESC1 ─── VESC2 ─── VESC3 ─── VESC4 ─── Tool MCU
  │                                                    │
 120Ω                                                120Ω
```

- Twisted pair (CANH/CANL)
- 120Ω termination at each end
- 500 kbps

### Power Connectors

| Connector | Type         | Use              |
| --------- | ------------ | ---------------- |
| Battery   | XT90         | Main power       |
| Motor     | 5.5mm bullet | Phase wires      |
| 12V       | XT30         | Accessories      |
| Signal    | JST-XH       | Sensors, buttons |

## E-Stop Circuit

```
        48V+
          │
    ┌─────┴─────┐
    │  NC Relay │◄── E-Stop signal (GPIO)
    │  100A     │
    └─────┬─────┘
          │
      To VESCs
```

- Normally closed relay
- Physical E-Stop button wired in series
- Software can trigger via GPIO
- Fail-safe: loss of signal = stop

## Jetson Power

- Input: 12V from DCDC
- Power adapter: 12V barrel jack or direct wiring
- Enable proper shutdown sequence on low battery

## Jetson CAN Setup (reComputer J4012)

The reComputer J4012 (Jetson Orin NX) exposes CAN TX/RX at TTL levels on header J16. An external transceiver is required.

### Hardware

**J16 CAN Header Pinout:**

| Pin | Signal |
| --- | ------ |
| 1   | 3V3    |
| 2   | GND    |
| 3   | CAN RX |
| 4   | CAN TX |

**Wiring to SN65HVD230 Transceiver:**

```
J16 (Jetson)              SN65HVD230              VESC
────────────              ──────────              ────
3V3  ───────────────────► VCC
GND  ───────────────────► GND ◄───────────────── GND
CAN TX ─────────────────► TX (D)
CAN RX ◄──────────────── RX (R)
                          CANH ────────────────► CANH
                          CANL ────────────────► CANL
```

**Notes:**

- Ensure common ground between Jetson, transceiver, and VESCs
- 120Ω termination at each end of bus (VESC has built-in termination option)
- Keep CAN wires as twisted pair

### Software Setup

**1. Load kernel modules:**

```bash
sudo modprobe can
sudo modprobe can_raw
sudo modprobe can-dev
sudo modprobe mttcan
```

**2. Configure and bring up interface:**

```bash
sudo ip link set can0 type can bitrate 500000
sudo ip link set can0 up
```

**3. Test with can-utils:**

```bash
sudo apt install can-utils
candump can0          # Listen for VESC status messages
cansend can0 000#00   # Send a test frame
```

### Persistent Configuration

Create a systemd service to configure CAN on boot:

```bash
sudo tee /etc/systemd/system/can0.service << 'EOF'
[Unit]
Description=CAN0 interface
After=network.target

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStartPre=/sbin/modprobe can
ExecStartPre=/sbin/modprobe can_raw
ExecStartPre=/sbin/modprobe can-dev
ExecStartPre=/sbin/modprobe mttcan
ExecStart=/sbin/ip link set can0 type can bitrate 500000
ExecStart=/sbin/ip link set can0 up
ExecStop=/sbin/ip link set can0 down

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable can0.service
sudo systemctl start can0.service
```

### Verifying CAN Bus

Use the `bvr` CLI tool to scan for connected VESCs:

```bash
bvr scan
```

Expected output with all 4 VESCs configured:

```
┌─────────┬────────┬─────────┬─────────┬──────────┬──────────┬──────────┐
│   ID    │  ERPM  │ Current │  Duty   │ FET Temp │ Mot Temp │ Voltage  │
├─────────┼────────┼─────────┼─────────┼──────────┼──────────┼──────────┤
│   0 (0x00) │      0 │   0.0 A │   0.0%  │   19.3°C │   22.4°C │   48.0 V │
│   1 (0x01) │      0 │   0.0 A │   0.0%  │   18.2°C │   22.5°C │   48.0 V │
│   2 (0x02) │      0 │   0.0 A │   0.0%  │   19.0°C │   22.2°C │   48.0 V │
│   3 (0x03) │      0 │   0.0 A │   0.0%  │   18.4°C │   22.7°C │   48.0 V │
└─────────┴────────┴─────────┴─────────┴──────────┴──────────┴──────────┘
```

### Troubleshooting

| Issue                | Solution                                           |
| -------------------- | -------------------------------------------------- |
| `can0` not appearing | Check `lsmod \| grep mttcan`, load `can-dev` first |
| No VESC messages     | Verify wiring, check VESC CAN mode enabled         |
| Bus errors           | Check termination, verify 500kbps on both ends     |
| Interface disappears | Reload mttcan module, check `dmesg` for errors     |
| Missing VESCs        | Check CAN wiring, verify unique IDs in VESC Tool   |

## VESC Configuration

Each VESC must be configured via VESC Tool (USB connection):

**App Settings → General:**

| Setting         | Value    |
| --------------- | -------- |
| Controller ID   | 0-3      |
| CAN Mode        | VESC     |
| CAN Baud Rate   | CAN_500K |
| Send CAN Status | Enabled  |
| CAN Status Rate | 50 Hz    |

**Recommended ID assignment:**

| Position    | CAN ID |
| ----------- | ------ |
| Front Left  | 0      |
| Front Right | 1      |
| Rear Left   | 2      |
| Rear Right  | 3      |

**Termination:** Enable 120Ω termination on the first and last VESC in the chain (App Settings → General → CAN Termination).

## Tool Connector

Standard connector for tools includes:

| Pin | Signal   |
| --- | -------- |
| 1   | 12V      |
| 2   | GND      |
| 3   | CANH     |
| 4   | CANL     |
| 5   | Reserved |
| 6   | Reserved |

Connector type: Deutsch DT06-6S (weatherproof)
