# Bill of Materials

## Summary

| Category    | Est. Cost   |
| ----------- | ----------- |
| Chassis     | $150        |
| Drivetrain  | $800        |
| Electronics | $900        |
| Power       | $400        |
| Wiring/Misc | $100        |
| **Total**   | **~$2,350** |

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

| Part                      | Qty | Unit | Total    | Link   |
| ------------------------- | --- | ---- | -------- | ------ |
| Jetson Orin NX 16GB       | 1   | $600 | $600     | NVIDIA |
| Jetson carrier board      | 1   | $100 | $100     | Seeed  |
| USB CAN adapter           | 1   | $30  | $30      | Amazon |
| LTE modem (Sierra MC7455) | 1   | $80  | $80      | eBay   |
| 7" HDMI display           | 1   | $50  | $50      | Amazon |
| GPS module (optional)     | 1   | $30  | $30      | Amazon |
| **Subtotal**              |     |      | **$890** |        |

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

## Vendor Notes

- **SendCutSend**: Custom cut aluminum/steel, fast turnaround
- **Flipsky**: VESC clones, good value
- **AliExpress**: Hoverboard motors (2-3 week shipping)
- **Seeed Studio**: Jetson carrier boards


