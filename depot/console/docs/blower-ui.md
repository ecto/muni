# Blower Control UI Implementation

This document specifies the implementation for the blower control interface in the depot console web application.

## Overview

The blower control interface allows operators to monitor and adjust blower power during teleop mode. It displays the current blower power level (0-100%), operational mode (autonomous/teleop), and active state, while providing manual control via a slider and toggle button.

## 1. UI Design

### Component Hierarchy

```
TeleopView
├── Scene (3D viewport)
├── Left Panel Stack
│   ├── Exit Button
│   ├── TelemetryPanel
│   ├── InputPanel
│   ├── PositionPanel
│   └── BlowerPanel ⭐ NEW
└── ConnectionBar
```

### Visual Mockup

```
┌─────────────────────────────────────┐
│ Blower Control        ⚫ Inactive  │
├─────────────────────────────────────┤
│ Mode                                │
│ Teleop                              │
│                                      │
│ ─────────────────                   │
│                                      │
│ Power                         75%   │
│ ━━━━━━━━━━━━━━━━━━░░░░ 75          │
│                                      │
│ ─────────────────                   │
│                                      │
│ [ Enable Blower ]                   │
└─────────────────────────────────────┘
```

Active state:
```
┌─────────────────────────────────────┐
│ Blower Control        🟢 Active    │
├─────────────────────────────────────┤
│ Mode                                │
│ Autonomous                          │
│                                      │
│ ─────────────────                   │
│                                      │
│ Power                         45%   │
│ ━━━━━━━━━░░░░░░░░░░░░░ 45          │
│                                      │
│ ─────────────────                   │
│                                      │
│ [ Disable Blower ]                  │
└─────────────────────────────────────┘
```

### Placement in Existing Interface

The `BlowerPanel` component should be added to the left panel stack in `TeleopView.tsx`, positioned after the `PositionPanel`:

```tsx
{/* Top left panels */}
<div className="absolute top-4 left-4 flex flex-col gap-4 pointer-events-auto">
  <Button variant="secondary" size="sm" onClick={handleExit}>...</Button>
  <TelemetryPanel />
  <InputPanel />
  <PositionPanel />
  <BlowerPanel /> {/* NEW */}
</div>
```

## 2. State Management

### Zustand Store Extensions

Add blower state to `ConsoleState` in `/Users/cam/Developer/muni/depot/console/src/store.ts`:

```typescript
interface ConsoleState {
  // ... existing fields ...

  // Blower control
  blowerPower: number;          // 0-100 (percentage)
  blowerMode: 'autonomous' | 'teleop';
  blowerActive: boolean;
  setBlowerPower: (power: number) => void;
  setBlowerMode: (mode: 'autonomous' | 'teleop') => void;
  setBlowerActive: (active: boolean) => void;
  updateBlowerState: (partial: Partial<{
    power: number;
    mode: 'autonomous' | 'teleop';
    active: boolean;
  }>) => void;
}
```

### Store Implementation

Add to the store creation in `store.ts`:

```typescript
export const useConsoleStore = create<ConsoleState>((set) => ({
  // ... existing state ...

  // Blower control (defaults)
  blowerPower: 0,
  blowerMode: 'teleop',
  blowerActive: false,
  setBlowerPower: (power) => set({ blowerPower: Math.max(0, Math.min(100, power)) }),
  setBlowerMode: (mode) => set({ blowerMode: mode }),
  setBlowerActive: (active) => set({ blowerActive: active }),
  updateBlowerState: (partial) => set((state) => ({
    blowerPower: partial.power !== undefined ? Math.max(0, Math.min(100, partial.power)) : state.blowerPower,
    blowerMode: partial.mode ?? state.blowerMode,
    blowerActive: partial.active ?? state.blowerActive,
  })),
}));
```

## 3. WebSocket Protocol

### Message Types

Add to `/Users/cam/Developer/muni/depot/console/src/lib/protocol.ts`:

```typescript
// Add to existing message type constants
export const MSG_BLOWER = 0x07;
export const MSG_BLOWER_STATUS = 0x11;
```

### Command Encoding (Operator → Rover)

```typescript
/**
 * Encode blower control command.
 * Format: [type:u8] [power:u8] [active:u8]
 *
 * @param power - Blower power 0-100 (percentage)
 * @param active - Whether blower should be active (true/false)
 */
export function encodeBlower(power: number, active: boolean): ArrayBuffer {
  const buf = new ArrayBuffer(3);
  const view = new DataView(buf);
  view.setUint8(0, MSG_BLOWER);
  view.setUint8(1, Math.max(0, Math.min(100, Math.round(power))));
  view.setUint8(2, active ? 1 : 0);
  return buf;
}
```

### Status Decoding (Rover → Operator)

```typescript
/**
 * Decoded blower status from rover.
 */
export interface DecodedBlowerStatus {
  power: number;           // 0-100
  mode: 'autonomous' | 'teleop';
  active: boolean;
}

/**
 * Decode blower status message.
 * Format: [type:u8] [power:u8] [mode:u8] [active:u8]
 */
export function decodeBlowerStatus(data: ArrayBuffer): DecodedBlowerStatus | null {
  const view = new DataView(data);

  if (data.byteLength < 4) {
    return null;
  }

  const msgType = view.getUint8(0);
  if (msgType !== MSG_BLOWER_STATUS) {
    return null;
  }

  const power = view.getUint8(1);
  const modeValue = view.getUint8(2);
  const active = view.getUint8(3) === 1;

  const mode = modeValue === 0 ? 'autonomous' : 'teleop';

  return {
    power,
    mode,
    active,
  };
}
```

### Example JSON Payloads (for reference/debugging)

While the actual protocol uses binary messages, here are equivalent JSON representations for clarity:

**Set Blower Power (Operator → Rover)**
```json
{
  "type": "blower_command",
  "power": 75,
  "active": true
}
```

**Blower Status Update (Rover → Operator)**
```json
{
  "type": "blower_status",
  "power": 45,
  "mode": "autonomous",
  "active": true
}
```

## 4. React Component

### Component File Structure

Create `/Users/cam/Developer/muni/depot/console/src/components/teleop/BlowerPanel.tsx`:

```typescript
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Button } from "@/components/ui/button";
import { Fan, Circle } from "@phosphor-icons/react";
import { useConsoleStore } from "@/store";
import { Mode } from "@/lib/types";
import { useBlowerControl } from "@/hooks/useBlowerControl";

interface BlowerPanelProps {
  /**
   * Optional className for custom styling.
   */
  className?: string;
}

/**
 * Blower control panel for teleop mode.
 *
 * Displays current blower power, mode, and active state.
 * Allows operator to adjust power and toggle blower on/off when in teleop mode.
 */
export function BlowerPanel({ className }: BlowerPanelProps) {
  const { telemetry, blowerPower, blowerMode, blowerActive } = useConsoleStore();
  const { setBlowerPower, toggleBlower } = useBlowerControl();

  // Blower controls are only available in Teleop mode
  const isTeleopMode = telemetry.mode === Mode.Teleop;
  const isDisabled = !isTeleopMode;

  return (
    <Card className={`w-64 bg-card/90 backdrop-blur ${className ?? ''}`}>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-medium flex items-center justify-between">
          <span className="flex items-center gap-2">
            <Fan className="h-4 w-4" weight={blowerActive ? "fill" : "regular"} />
            Blower Control
          </span>
          <Badge variant={blowerActive ? "default" : "outline"}>
            {blowerActive ? (
              <>
                <Circle className="h-2 w-2 mr-1 fill-green-500" weight="fill" />
                Active
              </>
            ) : (
              <>
                <Circle className="h-2 w-2 mr-1" weight="regular" />
                Inactive
              </>
            )}
          </Badge>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Mode indicator */}
        <div className="flex items-center justify-between">
          <span className="text-sm text-muted-foreground">Mode</span>
          <Badge variant={blowerMode === 'teleop' ? 'default' : 'secondary'}>
            {blowerMode === 'teleop' ? 'Teleop' : 'Autonomous'}
          </Badge>
        </div>

        <Separator />

        {/* Power slider */}
        <div className="space-y-2">
          <div className="flex items-center justify-between text-sm">
            <span className="text-muted-foreground">Power</span>
            <span className="font-mono">{blowerPower}%</span>
          </div>
          <input
            type="range"
            min="0"
            max="100"
            step="5"
            value={blowerPower}
            onChange={(e) => setBlowerPower(Number(e.target.value))}
            disabled={isDisabled}
            className="w-full h-2 bg-muted rounded-lg appearance-none cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed
              [&::-webkit-slider-thumb]:appearance-none
              [&::-webkit-slider-thumb]:w-4
              [&::-webkit-slider-thumb]:h-4
              [&::-webkit-slider-thumb]:rounded-full
              [&::-webkit-slider-thumb]:bg-primary
              [&::-webkit-slider-thumb]:cursor-pointer
              [&::-webkit-slider-thumb]:disabled:bg-muted-foreground
              [&::-moz-range-thumb]:w-4
              [&::-moz-range-thumb]:h-4
              [&::-moz-range-thumb]:rounded-full
              [&::-moz-range-thumb]:bg-primary
              [&::-moz-range-thumb]:cursor-pointer
              [&::-moz-range-thumb]:border-0
              [&::-moz-range-thumb]:disabled:bg-muted-foreground"
            style={{
              background: `linear-gradient(to right, hsl(var(--primary)) 0%, hsl(var(--primary)) ${blowerPower}%, hsl(var(--muted)) ${blowerPower}%, hsl(var(--muted)) 100%)`
            }}
          />
        </div>

        <Separator />

        {/* Toggle button */}
        <Button
          variant={blowerActive ? "destructive" : "default"}
          size="sm"
          onClick={toggleBlower}
          disabled={isDisabled}
          className="w-full"
        >
          {blowerActive ? "Disable Blower" : "Enable Blower"}
        </Button>

        {/* Warning when not in teleop */}
        {!isTeleopMode && (
          <p className="text-xs text-muted-foreground text-center">
            Blower controls disabled outside Teleop mode
          </p>
        )}
      </CardContent>
    </Card>
  );
}
```

### Custom Hook for Blower Control

Create `/Users/cam/Developer/muni/depot/console/src/hooks/useBlowerControl.ts`:

```typescript
import { useCallback, useRef } from "react";
import { useConsoleStore } from "@/store";
import { encodeBlower } from "@/lib/protocol";

/**
 * Hook for managing blower control commands.
 *
 * Provides methods to set blower power and toggle active state,
 * automatically sending commands via WebSocket.
 */
export function useBlowerControl() {
  const { blowerPower, blowerActive, setBlowerPower: updatePower, setBlowerActive } = useConsoleStore();

  // Access WebSocket ref from useRoverConnection
  // Note: This requires exposing wsRef from useRoverConnection or accessing via a global
  // For now, we'll use a similar pattern - store WS in a ref that's accessible
  const wsRef = useRef<WebSocket | null>(null);

  // Method to set WebSocket reference (called from useRoverConnection)
  const setWebSocket = useCallback((ws: WebSocket | null) => {
    wsRef.current = ws;
  }, []);

  /**
   * Set blower power and send command to rover.
   */
  const setBlowerPower = useCallback((power: number) => {
    updatePower(power);

    // Send command if connected
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(encodeBlower(power, blowerActive));
    }
  }, [updatePower, blowerActive]);

  /**
   * Toggle blower active state and send command to rover.
   */
  const toggleBlower = useCallback(() => {
    const newActive = !blowerActive;
    setBlowerActive(newActive);

    // Send command if connected
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(encodeBlower(blowerPower, newActive));
    }
  }, [blowerActive, blowerPower, setBlowerActive]);

  /**
   * Directly set blower active state (for status updates from rover).
   */
  const setActive = useCallback((active: boolean) => {
    setBlowerActive(active);
  }, [setBlowerActive]);

  return {
    setBlowerPower,
    toggleBlower,
    setActive,
    setWebSocket,
  };
}
```

**Note:** The WebSocket reference sharing between `useRoverConnection` and `useBlowerControl` requires modification to `useRoverConnection.ts` to expose the WebSocket ref. See Integration section below.

## 5. Integration

### 5.1 Add Component to TeleopView

Modify `/Users/cam/Developer/muni/depot/console/src/views/TeleopView.tsx`:

```tsx
import { BlowerPanel } from "@/components/teleop/BlowerPanel";

export function TeleopView() {
  // ... existing code ...

  return (
    <div className="h-screen w-screen overflow-hidden bg-background">
      <Scene />

      {/* ... existing overlays ... */}

      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-4 left-4 flex flex-col gap-4 pointer-events-auto">
          <Button variant="secondary" size="sm" onClick={handleExit}>...</Button>
          <TelemetryPanel />
          <InputPanel />
          <PositionPanel />
          <BlowerPanel />  {/* NEW */}
        </div>

        {/* ... rest of UI ... */}
      </div>
    </div>
  );
}
```

### 5.2 Extend useRoverConnection

Modify `/Users/cam/Developer/muni/depot/console/src/hooks/useRoverConnection.ts` to:

1. Decode blower status messages
2. Expose WebSocket reference for blower commands

**Add imports:**
```typescript
import { decodeBlowerStatus } from "@/lib/protocol";
```

**Add to onmessage handler:**
```typescript
ws.onmessage = (event) => {
  if (!(event.data instanceof ArrayBuffer)) {
    return;
  }

  // Try telemetry first
  const decoded = decodeTelemetry(event.data);
  if (decoded) {
    // ... existing telemetry handling ...
    return;
  }

  // Try blower status
  const blowerStatus = decodeBlowerStatus(event.data);
  if (blowerStatus) {
    const { updateBlowerState } = useConsoleStore.getState();
    updateBlowerState({
      power: blowerStatus.power,
      mode: blowerStatus.mode,
      active: blowerStatus.active,
    });
    return;
  }
};
```

**Expose WebSocket in return value:**
```typescript
export function useRoverConnection() {
  // ... existing code ...

  return {
    connect,
    disconnect,
    sendEStop,
    sendEStopRelease,
    ws: wsRef.current,  // NEW: expose for other hooks
  };
}
```

### 5.3 Connect useBlowerControl to WebSocket

Modify the `useBlowerControl` hook to use the exposed WebSocket:

```typescript
export function useBlowerControl() {
  const { ws } = useRoverConnection();  // Get WS from connection hook
  const { blowerPower, blowerActive, setBlowerPower: updatePower, setBlowerActive } = useConsoleStore();

  const setBlowerPower = useCallback((power: number) => {
    updatePower(power);

    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(encodeBlower(power, blowerActive));
    }
  }, [ws, updatePower, blowerActive]);

  // ... rest of hook ...
}
```

### 5.4 Interaction with Teleop Mode

The blower controls are automatically disabled when not in Teleop mode by checking:

```typescript
const isTeleopMode = telemetry.mode === Mode.Teleop;
```

When the rover transitions out of Teleop mode:
- The slider becomes disabled (grayed out)
- The toggle button becomes disabled
- A help text appears explaining controls are disabled
- The mode badge updates to show current mode

The blower can still be active in Autonomous mode, but operator cannot manually adjust it.

## 6. Testing

### Manual Testing Checklist

#### Visual Tests
- [ ] Component renders with correct default state (inactive, 0% power)
- [ ] Badge shows "Inactive" with hollow circle when blower is off
- [ ] Badge shows "Active" with green filled circle when blower is on
- [ ] Mode badge correctly displays "Teleop" or "Autonomous"
- [ ] Power percentage displays correctly next to slider
- [ ] Slider visual fill matches power percentage
- [ ] Button text changes between "Enable Blower" and "Disable Blower"
- [ ] Component matches styling of other panels (card background, font, spacing)

#### Interaction Tests
- [ ] Slider adjusts power value (0-100 in steps of 5)
- [ ] Moving slider sends WebSocket command
- [ ] Clicking toggle button switches active state
- [ ] Toggle button sends WebSocket command
- [ ] Controls are enabled in Teleop mode
- [ ] Controls are disabled in Idle mode
- [ ] Controls are disabled in Autonomous mode
- [ ] Controls are disabled in E-Stop mode
- [ ] Warning text appears when not in Teleop mode

#### WebSocket Tests
- [ ] encodeBlower produces correct binary format
- [ ] decodeBlowerStatus correctly parses rover messages
- [ ] Blower status updates from rover reflect in UI
- [ ] Commands sent match expected binary protocol
- [ ] Power value is clamped to 0-100 range
- [ ] Changing power while inactive still sends updated power value

#### Integration Tests
- [ ] Component appears in left panel stack
- [ ] Component doesn't interfere with other panels
- [ ] State persists when navigating between rovers (if applicable)
- [ ] WebSocket disconnection doesn't crash component
- [ ] Rapid slider adjustments don't overwhelm connection

### Edge Cases to Verify

#### Power Limits
- [ ] Setting power to -10 clamps to 0
- [ ] Setting power to 150 clamps to 100
- [ ] Slider cannot be dragged below 0
- [ ] Slider cannot be dragged above 100

#### Connection States
- [ ] Component handles WebSocket being null
- [ ] Component handles WebSocket being closed
- [ ] Component handles WebSocket being connecting
- [ ] No errors thrown when sending commands while disconnected

#### Mode Transitions
- [ ] Switching from Teleop → Autonomous disables controls
- [ ] Switching from Autonomous → Teleop enables controls
- [ ] Blower state is preserved across mode transitions
- [ ] Mode badge updates immediately on mode change

#### Rapid Input
- [ ] Dragging slider rapidly doesn't cause lag
- [ ] Multiple quick toggle clicks don't desync state
- [ ] Power updates from rover don't conflict with user input

## 7. Styling

### Tailwind CSS Classes Used

The component uses the existing design system from the depot console:

#### Card Container
```tsx
className="w-64 bg-card/90 backdrop-blur"
```
- `w-64`: Fixed width matching other panels (256px)
- `bg-card/90`: Semi-transparent card background
- `backdrop-blur`: Glass morphism effect

#### Header
```tsx
className="text-sm font-medium flex items-center justify-between"
```
- Consistent with `TelemetryPanel` and `InputPanel` headers

#### Badges
```tsx
variant={blowerActive ? "default" : "outline"}
variant={blowerMode === 'teleop' ? 'default' : 'secondary'}
```
- Uses existing badge variants from `@/components/ui/badge`
- `default`: Blue primary color for active states
- `outline`: Gray border for inactive states
- `secondary`: Muted background for non-primary info

#### Status Indicators
```tsx
<Circle className="h-2 w-2 mr-1 fill-green-500" weight="fill" />
```
- Green filled circle for active state
- Hollow circle for inactive state
- Phosphor Icons for consistency

#### Range Slider Styling

The native HTML range input is styled to match the console's design:

```css
/* Applied via className string */
w-full h-2 bg-muted rounded-lg appearance-none cursor-pointer
disabled:opacity-50 disabled:cursor-not-allowed

/* WebKit (Chrome, Safari) */
[&::-webkit-slider-thumb]:appearance-none
[&::-webkit-slider-thumb]:w-4
[&::-webkit-slider-thumb]:h-4
[&::-webkit-slider-thumb]:rounded-full
[&::-webkit-slider-thumb]:bg-primary
[&::-webkit-slider-thumb]:cursor-pointer

/* Firefox */
[&::-moz-range-thumb]:w-4
[&::-moz-range-thumb]:h-4
[&::-moz-range-thumb]:rounded-full
[&::-moz-range-thumb]:bg-primary
[&::-moz-range-thumb]:border-0
```

The slider uses a gradient background to show filled/unfilled portions:

```tsx
style={{
  background: `linear-gradient(to right,
    hsl(var(--primary)) 0%,
    hsl(var(--primary)) ${blowerPower}%,
    hsl(var(--muted)) ${blowerPower}%,
    hsl(var(--muted)) 100%)`
}}
```

#### Button Styling
```tsx
variant={blowerActive ? "destructive" : "default"}
size="sm"
className="w-full"
```
- `destructive` variant (red) when active to emphasize "Disable" action
- `default` variant (blue) when inactive for "Enable" action
- `size="sm"` for compact appearance
- `w-full` to span card width

#### Text Styles
```tsx
className="text-sm text-muted-foreground"      // Labels
className="font-mono"                           // Numeric values
className="text-xs text-muted-foreground text-center"  // Help text
```

### Color System

All colors use CSS custom properties from Tailwind v4:

- `--primary`: Interactive elements (slider thumb, active badges)
- `--muted`: Background for inactive elements
- `--muted-foreground`: Secondary text
- `--destructive`: Disable button (red)
- `--card`: Panel background
- `--green-500`: Active status indicator

### Responsive Behavior

The component maintains fixed width (`w-64`) and stacks vertically in the left panel. On smaller screens, the panel may need to be collapsible or moved to a different location (not implemented in this version).

## Additional Notes

### Future Enhancements

1. **Blower Feedback**: Add motor current or RPM display if available from rover
2. **Auto-disable Timer**: Automatically disable blower after N minutes of inactivity
3. **Power Presets**: Quick-select buttons for common power levels (25%, 50%, 75%, 100%)
4. **Visual Indicator**: Add animated fan icon that spins when active
5. **Audio Feedback**: Play sound when blower is enabled/disabled
6. **Telemetry Graph**: Show blower power history over time

### Known Limitations

1. WebSocket ref sharing between hooks requires careful synchronization
2. Rapid slider adjustments may send many WebSocket messages (consider debouncing)
3. No local queue for commands sent while disconnected
4. State doesn't persist across browser refresh (could add localStorage)

### Related Files

- `/Users/cam/Developer/muni/depot/console/src/components/teleop/BlowerPanel.tsx` (component)
- `/Users/cam/Developer/muni/depot/console/src/hooks/useBlowerControl.ts` (control logic)
- `/Users/cam/Developer/muni/depot/console/src/lib/protocol.ts` (binary protocol)
- `/Users/cam/Developer/muni/depot/console/src/store.ts` (state management)
- `/Users/cam/Developer/muni/depot/console/src/views/TeleopView.tsx` (integration)
- `/Users/cam/Developer/muni/depot/console/src/hooks/useRoverConnection.ts` (WebSocket)

### Rover-side Implementation

This documentation covers only the console UI. The rover firmware must implement:

1. Blower command handler (0x07 message type)
2. Blower status publisher (0x11 message type)
3. Autonomous blower control logic
4. Mode-aware blower enable/disable
5. Safety interlocks (e-stop should disable blower)

Refer to BVR firmware documentation for rover-side implementation details.
