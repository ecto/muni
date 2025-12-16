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
