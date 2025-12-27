# Bill of Materials

## Summary

| Category    | Est. Cost   |
| ----------- | ----------- |
| Chassis     | $150        |
| Drivetrain  | $800        |
| Electronics | $1,286      |
| Perception  | $1,800      |
| Power       | $400        |
| Wiring/Misc | $100        |
| **Total**   | **~$4,536** |

Depot base station adds ~$1,540 (rack + mesh). Fixed repeaters: $114-214 each as needed.

## Detailed BOM

### Chassis

| Part                        | Qty | Unit  | Total    | Link        |
| --------------------------- | --- | ----- | -------- | ----------- |
| 2020 extrusion 600mm        | 8   | $5    | $40      |             |
| 2020 corner bracket         | 16  | $2    | $32      |             |
| M5×10 BHCS                  | 100 | $0.10 | $10      |             |
| M5 T-nut                    | 100 | $0.15 | $15      |             |
| Electronics plate (1/4" AL) | 1   | $50   | $50      | SendCutSend |
| **Subtotal**                |     |       | **$147** |             |

### Drivetrain

| Part                      | Qty | Unit | Total    | Link        |
| ------------------------- | --- | ---- | -------- | ----------- |
| Hoverboard hub motor 350W | 4   | $50  | $200     | AliExpress  |
| VESC 6.6                  | 4   | $120 | $480     | Flipsky     |
| Motor mount (custom)      | 4   | $20  | $80      | SendCutSend |
| Wheel spacer (custom)     | 4   | $10  | $40      | SendCutSend |
| **Subtotal**              |     |      | **$800** |             |

### Electronics

| Part                   | Qty | Unit | Total      | Link     |
| ---------------------- | --- | ---- | ---------- | -------- |
| Jetson Orin NX 16GB    | 1   | $600 | $600       | NVIDIA   |
| Jetson carrier board   | 1   | $100 | $100       | Seeed    |
| USB CAN adapter        | 1   | $30  | $30        | Amazon   |
| Proxicast 7-in-1 combo | 1   | $367 | $367       | Amazon   |
| Bullet AC IP67         | 1   | $129 | $129       | UI Store |
| RP-SMA to N adapter    | 1   | $10  | $10        | Amazon   |
| 7" HDMI display        | 1   | $50  | $50        | Amazon   |
| **Subtotal**           |     |      | **$1,286** |          |

Note: Proxicast 7-in-1 includes GPS and 4x4 MIMO 5G antenna ports for LTE/5G
fallback. See [networking.md](networking.md).

### Perception

| Part                      | Qty | Unit   | Total      | Link        |
| ------------------------- | --- | ------ | ---------- | ----------- |
| Livox Mid-360 LiDAR       | 1   | $1,500 | $1,500     | Livox / DJI |
| Insta360 X4               | 1   | $300   | $300       | Amazon      |
| Sensor mount pole (1" AL) | 1   | $20    | $20        | Amazon      |
| **Subtotal**              |     |        | **$1,820** |             |

See [sensors.md](sensors.md) for detailed specifications and integration.

### Power System

| Part                    | Qty | Unit | Total    | Link   |
| ----------------------- | --- | ---- | -------- | ------ |
| 13S4P battery pack 20Ah | 1   | $300 | $300     | Custom |
| 48V→12V DCDC 20A        | 1   | $40  | $40      | Amazon |
| 100A ANL fuse + holder  | 1   | $15  | $15      | Amazon |
| E-Stop relay 100A       | 1   | $25  | $25      | Amazon |
| E-Stop button           | 1   | $15  | $15      | Amazon |
| **Subtotal**            |     |      | **$395** |        |

### Wiring & Connectors

| Part                        | Qty | Unit | Total   | Link   |
| --------------------------- | --- | ---- | ------- | ------ |
| 8 AWG silicone wire (red)   | 2m  | $3/m | $6      | Amazon |
| 8 AWG silicone wire (black) | 2m  | $3/m | $6      | Amazon |
| 14 AWG wire assortment      | 1   | $15  | $15     | Amazon |
| 22 AWG twisted pair         | 5m  | $1/m | $5      | Amazon |
| XT90 connectors (5 pair)    | 1   | $12  | $12     | Amazon |
| XT30 connectors (10 pair)   | 1   | $8   | $8      | Amazon |
| Deutsch DT connector kit    | 1   | $25  | $25     | Amazon |
| Heat shrink kit             | 1   | $12  | $12     | Amazon |
| Cable management            | 1   | $10  | $10     | Amazon |
| **Subtotal**                |     |      | **$99** |        |

## Tool: Snow Auger

| Part                | Qty | Unit | Total    | Link   |
| ------------------- | --- | ---- | -------- | ------ |
| RP2040 Pico W       | 1   | $8   | $8       |        |
| MCP2515 CAN module  | 1   | $5   | $5       |        |
| Linear actuator 12" | 1   | $60  | $60      | Amazon |
| Auger motor 500W    | 1   | $80  | $80      |        |
| Motor controller    | 1   | $30  | $30      |        |
| Auger assembly      | 1   | $150 | $150     |        |
| Mount hardware      | 1   | $30  | $30      |        |
| **Subtotal**        |     |      | **$363** |        |

## bvr1 Optional Upgrades

These components are not required for bvr0 but enable enhanced capabilities.

### RTK GPS (Per Rover)

For centimeter-accurate georeferenced mapping. Same hardware as depot base
station, configured in rover mode.

| Part                        | Qty | Unit | Total    | Link                                                                                 |
| --------------------------- | --- | ---- | -------- | ------------------------------------------------------------------------------------ |
| SparkFun GPS-RTK2 (ZED-F9P) | 1   | $275 | $275     | [Amazon](https://www.amazon.com/SparkFun-GPS-RTK2-Board-ZED-F9P-Qwiic/dp/B07NBPNWNZ) |
| GNSS Multi-Band Antenna     | 1   | $80  | $80      | SparkFun                                                                             |
| SMA cable (1m)              | 1   | $15  | $15      | Amazon                                                                               |
| **Subtotal**                |     |      | **$370** |                                                                                      |

See [rover-rack.md](rover-rack.md) for mounting and [rtk.md](rtk.md) for setup.

### Network Switch (Per Rover)

For debug access and future sensor expansion.

| Part                           | Qty | Unit | Total   | Link   |
| ------------------------------ | --- | ---- | ------- | ------ |
| Netgear GS305 (5-port gigabit) | 1   | $20  | $20     | Amazon |
| Ethernet cables (0.5m)         | 3   | $3   | $9      | Amazon |
| **Subtotal**                   |     |      | **$29** |        |

### Hardware-Synced Camera (Per Rover)

For high-quality Gaussian splatting (replaces Insta360).

| Part                             | Qty | Unit | Total      | Link   |
| -------------------------------- | --- | ---- | ---------- | ------ |
| FLIR Blackfly S (global shutter) | 2   | $500 | $1,000     | B&H    |
| Wide-angle lens (CS mount)       | 2   | $50  | $100       | Amazon |
| Sync cable (Mid-360 PPS)         | 1   | $20  | $20        | Custom |
| **Subtotal**                     |     |      | **$1,120** |        |

## Depot Rack

Complete base station for fleet operations. See [depot.md](depot.md) for full details.

| Part                        | Qty | Unit | Total      | Link      |
| --------------------------- | --- | ---- | ---------- | --------- |
| GeeekPi 10" 6U Rack         | 1   | $80  | $80        | Amazon    |
| GeeekPi 10.1" Touchscreen   | 1   | $110 | $110       | Amazon    |
| Ubiquiti USW-Flex PoE       | 1   | $100 | $100       | UI Store  |
| Raspberry Pi 5 8GB          | 1   | $80  | $80        | PiShop    |
| GeeekPi P31 NVMe PoE+ HAT   | 1   | $35  | $35        | Amazon    |
| WD SN740 2230 256GB         | 1   | $30  | $30        | Amazon    |
| SparkFun GPS-RTK2 (ZED-F9P) | 1   | $275 | $275       | Amazon    |
| Tallysman TW4721 antenna    | 1   | $100 | $100       | Tallysman |
| LMR-400 cable (25ft)        | 1   | $50  | $50        | Amazon    |
| EcoFlow River 3             | 1   | $200 | $200       | Amazon    |
| Misc (shelf, splitter, etc) | 1   | $80  | $80        | Various   |
| **Subtotal**                |     |      | **$1,140** |           |

Note: Requires internet gateway (ISP-provided router, or add UniFi Dream Router
for $199). The USW-Flex is Layer 2 only and cannot provide NAT/DHCP.

### Mesh Base Station (Required)

| Part                  | Qty | Unit | Total    | Link     |
| --------------------- | --- | ---- | -------- | -------- |
| Rocket 5AC Prism      | 1   | $249 | $249     | UI Store |
| AMO-5G13 omni antenna | 1   | $130 | $130     | UI Store |
| Antenna mount + cable | 1   | $50  | $50      | Amazon   |
| **Subtotal**          |     |      | **$429** |          |

Note: Omni provides 360° coverage for rovers dispatching in all directions.
Sector (AM-5G17-90, $100) available if longer range in one direction is needed.

### Patron WiFi (Optional)

| Part             | Qty | Unit | Total   | Link     |
| ---------------- | --- | ---- | ------- | -------- |
| UniFi AP AC Lite | 1   | $99  | $99     | UI Store |
| **Subtotal**     |     |      | **$99** |          |

Provides public WiFi at depot. Requires ISP with redistribution rights (AT&T
Business or Starlink Priority). See [networking.md](networking.md) for details.

### Fixed Repeater (Optional, add where needed)

| Part                      | Qty | Unit | Total    | Link     |
| ------------------------- | --- | ---- | -------- | -------- |
| NanoStation 5AC Loco      | 1   | $49  | $49      | UI Store |
| Outdoor NEMA enclosure    | 1   | $30  | $30      | Amazon   |
| PoE injector 24V          | 1   | $15  | $15      | UI Store |
| Pole mount clamps         | 1   | $20  | $20      | Amazon   |
| Solar kit (if no grid)    | 1   | $100 | $100     | Amazon   |
| **Subtotal (grid power)** |     |      | **$114** |          |
| **Subtotal (solar)**      |     |      | **$214** |          |

Alternatively, use an NTRIP network subscription (~$50/month) instead of
running your own RTK base station.

See [depot.md](depot.md) for rack setup and [rtk.md](rtk.md) for RTK configuration.

## Vendor Notes

- **SendCutSend**: Custom cut aluminum/steel, fast turnaround
- **Flipsky**: VESC clones, good value
- **AliExpress**: Hoverboard motors (2-3 week shipping)
- **Seeed Studio**: Jetson carrier boards
- **Livox/DJI**: Mid-360 LiDAR (direct order, ~1 week)
- **Amazon**: Insta360, mounting hardware (Prime shipping)
- **SparkFun**: GPS-RTK2, GNSS antennas
