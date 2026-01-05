# Console Performance Optimization Guide

This document provides performance optimization strategies for the depot console React application, covering rendering, memory management, and real-time updates.

## React Rendering Optimization

### Zustand Store Selectors

**Problem**: Selecting entire store causes re-renders on any state change.

**Solution**: Use selective subscriptions with selector functions.

```typescript
// ❌ Bad: Re-renders on ANY state change
function TelemetryPanel() {
  const state = useConsoleStore();  // Subscribes to entire store
  return <div>Mode: {state.mode}</div>;
}

// ✅ Good: Only re-renders when mode changes
function TelemetryPanel() {
  const mode = useConsoleStore((state) => state.mode);
  return <div>Mode: {mode}</div>;
}

// ✅ Good: Multiple selective subscriptions
function TelemetryPanel() {
  const { mode, pose, velocity } = useConsoleStore((state) => ({
    mode: state.mode,
    pose: state.pose,
    velocity: state.velocity,
  }));

  return (
    <div>
      <div>Mode: {ModeLabels[mode]}</div>
      <div>Position: ({pose.x.toFixed(2)}, {pose.y.toFixed(2)})</div>
      <div>Velocity: {velocity.linear.toFixed(2)} m/s</div>
    </div>
  );
}
```

**Impact**: Reduces unnecessary re-renders by 90%+.

### useMemo for Expensive Calculations

**Problem**: Calculations repeated on every render.

**Solution**: Memoize with `useMemo`.

```typescript
// ❌ Bad: Battery percentage calculated on every render
function BatteryIndicator() {
  const voltage = useConsoleStore((state) => state.power.voltage);

  const percentage = ((voltage - 42.0) / (54.0 - 42.0)) * 100;  // Recalculated every render

  return <div>{percentage.toFixed(0)}%</div>;
}

// ✅ Good: Memoized calculation
function BatteryIndicator() {
  const voltage = useConsoleStore((state) => state.power.voltage);

  const percentage = useMemo(() => {
    return ((voltage - 42.0) / (54.0 - 42.0)) * 100;
  }, [voltage]);

  return <div>{percentage.toFixed(0)}%</div>;
}
```

**When to use**:
- Expensive calculations (loops, filtering, sorting)
- Array/object transformations
- Formatting large datasets

**When NOT to use**:
- Simple arithmetic (toFixed, addition)
- Single property access
- Premature optimization (measure first!)

### useCallback for Event Handlers

**Problem**: Event handlers recreated on every render, causing child re-renders.

**Solution**: Memoize with `useCallback`.

```typescript
// ❌ Bad: New function created on every render
function RoverSelector() {
  const rovers = useConsoleStore((state) => state.rovers);
  const selectRover = useConsoleStore((state) => state.selectRover);

  return (
    <div>
      {rovers.map((rover) => (
        <RoverCard
          key={rover.id}
          rover={rover}
          onClick={() => selectRover(rover.id)}  // New function every render
        />
      ))}
    </div>
  );
}

// ✅ Good: Memoized event handler
function RoverSelector() {
  const rovers = useConsoleStore((state) => state.rovers);
  const selectRover = useConsoleStore((state) => state.selectRover);

  const handleSelect = useCallback(
    (id: string) => {
      selectRover(id);
    },
    [selectRover]
  );

  return (
    <div>
      {rovers.map((rover) => (
        <RoverCard key={rover.id} rover={rover} onClick={() => handleSelect(rover.id)} />
      ))}
    </div>
  );
}
```

**Impact**: Prevents unnecessary child re-renders when paired with `React.memo`.

### React.memo for Expensive Components

**Problem**: Child components re-render even when props unchanged.

**Solution**: Wrap with `React.memo`.

```typescript
// ❌ Without memo: Re-renders even if rover unchanged
function RoverCard({ rover, onClick }: RoverCardProps) {
  console.log("RoverCard rendered");  // Logs on every parent render
  return (
    <div onClick={onClick}>
      <h3>{rover.name}</h3>
      <p>{rover.status}</p>
    </div>
  );
}

// ✅ With memo: Only re-renders when props change
const RoverCard = React.memo(({ rover, onClick }: RoverCardProps) => {
  console.log("RoverCard rendered");  // Only logs when rover or onClick changes
  return (
    <div onClick={onClick}>
      <h3>{rover.name}</h3>
      <p>{rover.status}</p>
    </div>
  );
});
```

**When to use**:
- Expensive components (complex rendering, large lists)
- Components with stable props
- Components rendering frequently

**When NOT to use**:
- Simple components (single div, minimal logic)
- Props change on every render anyway
- Premature optimization

## Input Handling Performance

### Gamepad Polling Rate

**Problem**: `requestAnimationFrame` polls at display refresh rate (60Hz+), which is excessive.

**Solution**: Throttle polling to necessary rate.

```typescript
// ❌ Bad: Polls at 60Hz (every 16ms)
useEffect(() => {
  let frameId: number;

  const poll = () => {
    const gamepad = navigator.getGamepads()[0];
    if (gamepad) {
      updateInput(gamepad);  // 60 updates/sec
    }
    frameId = requestAnimationFrame(poll);
  };

  frameId = requestAnimationFrame(poll);
  return () => cancelAnimationFrame(frameId);
}, []);

// ✅ Good: Throttle to 60Hz with timing check
useEffect(() => {
  let frameId: number;
  let lastPoll = 0;
  const POLL_INTERVAL = 16.67; // ~60Hz

  const poll = (timestamp: number) => {
    if (timestamp - lastPoll >= POLL_INTERVAL) {
      const gamepad = navigator.getGamepads()[0];
      if (gamepad) {
        updateInput(gamepad);
      }
      lastPoll = timestamp;
    }
    frameId = requestAnimationFrame(poll);
  };

  frameId = requestAnimationFrame(poll);
  return () => cancelAnimationFrame(frameId);
}, []);
```

**Note**: 60Hz gamepad polling is already appropriate for teleop (commands sent at 100Hz). Don't throttle further.

### Dead Zone Application

**Problem**: Jittery input from stick drift causes unnecessary updates.

**Solution**: Apply dead zone to filter noise.

```typescript
// ✅ Good: Dead zone filtering
function applyDeadzone(value: number, threshold: number = 0.1): number {
  return Math.abs(value) < threshold ? 0 : value;
}

const poll = () => {
  const gamepad = navigator.getGamepads()[0];
  if (gamepad) {
    const linear = applyDeadzone(-gamepad.axes[1]);   // Inverted Y
    const angular = applyDeadzone(gamepad.axes[2]);

    // Only update if changed
    if (linear !== prevLinear || angular !== prevAngular) {
      updateInput({ linear, angular });
      prevLinear = linear;
      prevAngular = angular;
    }
  }
};
```

**Impact**: Reduces state updates by ~80% when sticks at rest.

## 3D Visualization Performance

### Frame Rate Independence

**Problem**: Animation speed varies with frame rate.

**Solution**: Use delta time for consistent motion.

```typescript
// ❌ Bad: Fixed increment (speed depends on frame rate)
useFrame(() => {
  ref.current.position.x += 0.1;  // Faster at 60fps than 30fps
});

// ✅ Good: Delta-based increment (consistent speed)
useFrame((state, delta) => {
  ref.current.position.x += velocity * delta;  // Same speed at any fps
});
```

### Smooth Interpolation (Lerp)

**Problem**: Direct position updates cause jittery motion.

**Solution**: Linear interpolation (lerp) for smooth transitions.

```typescript
// ❌ Bad: Direct assignment (jumpy)
useFrame(() => {
  const pose = useConsoleStore.getState().pose;
  ref.current.position.x = pose.x;
  ref.current.position.z = -pose.y;
});

// ✅ Good: Lerped position (smooth)
useFrame((state, delta) => {
  const pose = useConsoleStore.getState().pose;

  ref.current.position.x = THREE.MathUtils.lerp(
    ref.current.position.x,
    pose.x,
    delta * 10  // Lerp factor (higher = faster convergence)
  );

  ref.current.position.z = THREE.MathUtils.lerp(
    ref.current.position.z,
    -pose.y,
    delta * 10
  );
});
```

**Lerp factor guidelines**:
- `delta * 5`: Slow, smooth (1s to 90% convergence)
- `delta * 10`: Medium (0.5s to 90%)
- `delta * 20`: Fast (0.25s to 90%)

### Angle Wraparound Handling

**Problem**: Rotating from 350° to 10° goes backwards through 340°...20°.

**Solution**: Shortest-path angle interpolation.

```typescript
// ❌ Bad: Direct lerp (long way around)
ref.current.rotation.y = THREE.MathUtils.lerp(
  ref.current.rotation.y,
  targetAngle,
  delta * 10
);

// ✅ Good: Shortest-path interpolation
const current = ref.current.rotation.y;
const target = targetAngle;

// Calculate shortest angular difference
const diff = ((target - current + Math.PI) % (2 * Math.PI)) - Math.PI;

ref.current.rotation.y += diff * delta * 10;
```

### Geometry and Material Reuse

**Problem**: Creating new geometries/materials every render causes memory leaks.

**Solution**: Reuse geometries and materials.

```typescript
// ❌ Bad: New geometry every render
function RoverModel() {
  return (
    <mesh>
      <boxGeometry args={[1, 0.5, 1.5]} />  {/* Created every render */}
      <meshStandardMaterial color="green" />
    </mesh>
  );
}

// ✅ Good: Reused geometry (React Three Fiber handles this)
function RoverModel() {
  // Geometry is cached by args
  return (
    <mesh>
      <boxGeometry args={[1, 0.5, 1.5]} />
      <meshStandardMaterial color="green" />
    </mesh>
  );
}

// ✅ Better: useMemo for complex geometries
function RoverModel() {
  const geometry = useMemo(() => new THREE.BoxGeometry(1, 0.5, 1.5), []);
  const material = useMemo(() => new THREE.MeshStandardMaterial({ color: "green" }), []);

  return <mesh geometry={geometry} material={material} />;
}
```

### Dispose Resources

**Problem**: Three.js objects not disposed cause GPU memory leaks.

**Solution**: Dispose geometries, materials, and textures on unmount.

```typescript
// ✅ Good: Dispose on unmount
useEffect(() => {
  const geometry = new THREE.BoxGeometry(1, 1, 1);
  const material = new THREE.MeshStandardMaterial({ color: "red" });

  return () => {
    geometry.dispose();
    material.dispose();
  };
}, []);
```

React Three Fiber disposes objects automatically when removed from scene, but explicit disposal is safer for dynamically created objects.

## Video Streaming Performance

### Blob URL Management

**Problem**: Blob URLs not revoked cause memory leaks.

**Solution**: Revoke blob URLs when no longer needed.

```typescript
// ❌ Bad: Blob URLs never revoked (memory leak)
useEffect(() => {
  const ws = new WebSocket(videoAddress);
  ws.binaryType = "arraybuffer";

  ws.onmessage = (event) => {
    const frame = decodeVideoFrame(event.data);
    const blob = new Blob([frame.jpeg], { type: "image/jpeg" });
    const url = URL.createObjectURL(blob);  // Created but never revoked
    setVideoFrame(url);
  };

  return () => ws.close();
}, [videoAddress]);

// ✅ Good: Revoke old URL before setting new one
useEffect(() => {
  const ws = new WebSocket(videoAddress);
  ws.binaryType = "arraybuffer";

  ws.onmessage = (event) => {
    const frame = decodeVideoFrame(event.data);
    const blob = new Blob([frame.jpeg], { type: "image/jpeg" });
    const url = URL.createObjectURL(blob);

    // Revoke previous URL
    setVideoFrame((prevUrl) => {
      if (prevUrl) {
        URL.revokeObjectURL(prevUrl);
      }
      return url;
    });
  };

  return () => {
    ws.close();
    // Revoke on unmount
    setVideoFrame((prevUrl) => {
      if (prevUrl) URL.revokeObjectURL(prevUrl);
      return null;
    });
  };
}, [videoAddress]);
```

### Texture Updates

**Problem**: Creating new texture for every frame is expensive.

**Solution**: Reuse texture and update `needsUpdate` flag.

```typescript
// ❌ Bad: New texture every frame
useFrame(() => {
  if (videoFrame) {
    const texture = new THREE.TextureLoader().load(videoFrame);  // Expensive
    materialRef.current.map = texture;
  }
});

// ✅ Good: Update existing texture
useEffect(() => {
  if (!videoFrame || !materialRef.current) return;

  const img = new Image();
  img.onload = () => {
    if (materialRef.current.map) {
      // Update existing texture
      materialRef.current.map.image = img;
      materialRef.current.map.needsUpdate = true;
    } else {
      // Create texture once
      materialRef.current.map = new THREE.Texture(img);
      materialRef.current.map.needsUpdate = true;
    }
  };
  img.src = videoFrame;

  return () => {
    URL.revokeObjectURL(videoFrame);
  };
}, [videoFrame]);
```

### FPS Monitoring

**Solution**: Track frame rate for debugging.

```typescript
function useVideoFps() {
  const [fps, setFps] = useState(0);
  const frameTimesRef = useRef<number[]>([]);

  const recordFrame = useCallback(() => {
    const now = performance.now();
    frameTimesRef.current.push(now);

    // Keep last 1 second of frames
    frameTimesRef.current = frameTimesRef.current.filter((t) => now - t < 1000);

    setFps(frameTimesRef.current.length);
  }, []);

  return { fps, recordFrame };
}

// Usage
ws.onmessage = (event) => {
  recordFrame();  // Call on every frame
  // ... decode and display frame
};
```

## Memory Management

### Component Cleanup

**Pattern**: Always clean up resources in `useEffect` return.

```typescript
// ✅ Good: Comprehensive cleanup
useEffect(() => {
  // Setup
  const ws = new WebSocket(address);
  const interval = setInterval(sendHeartbeat, 100);
  const listener = (e: KeyboardEvent) => handleKey(e);
  document.addEventListener("keydown", listener);

  // Cleanup
  return () => {
    ws.close();
    clearInterval(interval);
    document.removeEventListener("keydown", listener);
  };
}, [address]);
```

### Store Cleanup

**Pattern**: Reset state when navigating away.

```typescript
// In view component
useEffect(() => {
  // Reset state on unmount
  return () => {
    useConsoleStore.getState().reset();
  };
}, []);
```

## Bundle Size Optimization

### Code Splitting

**Problem**: Large initial bundle (slow first load).

**Solution**: Lazy load routes with `React.lazy`.

```typescript
// ❌ Bad: All routes loaded upfront
import FleetView from "./views/FleetView";
import TeleopView from "./views/TeleopView";
import SessionsView from "./views/SessionsView";

// ✅ Good: Lazy-loaded routes
const FleetView = lazy(() => import("./views/FleetView"));
const TeleopView = lazy(() => import("./views/TeleopView"));
const SessionsView = lazy(() => import("./views/SessionsView"));

function App() {
  return (
    <Suspense fallback={<Loading />}>
      <Routes>
        <Route path="/fleet" element={<FleetView />} />
        <Route path="/teleop" element={<TeleopView />} />
        <Route path="/sessions" element={<SessionsView />} />
      </Routes>
    </Suspense>
  );
}
```

**Impact**: Reduces initial bundle by 50%+.

### Tree Shaking

**Tip**: Import only what you need from libraries.

```typescript
// ❌ Bad: Imports entire library
import * as THREE from "three";

// ✅ Good: Import specific items
import { BoxGeometry, MeshStandardMaterial } from "three";
```

## Performance Monitoring

### React DevTools Profiler

**Usage**:
1. Install React DevTools browser extension
2. Open DevTools → Profiler tab
3. Click record, interact with app, stop recording
4. Analyze flame graph for slow renders

**Look for**:
- Components rendering too frequently
- Long render times (>16ms at 60fps)
- Unnecessary re-renders

### Chrome Performance Tab

**Usage**:
1. Open DevTools → Performance tab
2. Record interaction
3. Analyze flame chart

**Look for**:
- Long tasks (>50ms blocks main thread)
- Excessive garbage collection (GC)
- Layout thrashing (forced reflows)

### Custom Performance Marks

```typescript
// Mark key operations
performance.mark("telemetry-decode-start");
const telemetry = decodeTelemetry(data);
performance.mark("telemetry-decode-end");

performance.measure("telemetry-decode", "telemetry-decode-start", "telemetry-decode-end");

// View in DevTools Performance tab or console
console.log(performance.getEntriesByName("telemetry-decode"));
```

## Performance Checklist

### Rendering
- [ ] Store selectors used (not full store)
- [ ] useMemo for expensive calculations
- [ ] useCallback for event handlers passed to children
- [ ] React.memo for expensive child components
- [ ] No inline object/array creation in JSX

### 3D Graphics
- [ ] Delta time used for animations
- [ ] Lerp for smooth interpolation
- [ ] Angle wraparound handled
- [ ] Geometries and materials reused
- [ ] Resources disposed on unmount

### Input
- [ ] Gamepad polling at appropriate rate (60Hz)
- [ ] Dead zone applied
- [ ] Input changes trigger updates (not every poll)

### Video
- [ ] Blob URLs revoked when replaced
- [ ] Textures updated (not recreated)
- [ ] FPS monitoring in place

### Memory
- [ ] useEffect cleanup functions present
- [ ] Event listeners removed
- [ ] WebSockets closed
- [ ] Intervals cleared

### Bundle
- [ ] Routes lazy-loaded
- [ ] Tree-shaking enabled (Vite default)
- [ ] Source maps in production disabled

## Benchmarks

Typical performance targets:

| Metric                  | Target   | Notes                          |
|-------------------------|----------|--------------------------------|
| Initial load (gzipped)  | <300 KB  | Main bundle                    |
| Time to Interactive     | <2s      | On 3G network                  |
| Command send rate       | 100 Hz   | 10ms interval                  |
| Telemetry receive rate  | 20-30 Hz | 50-33ms interval               |
| Video frame rate        | 30 fps   | 33ms interval                  |
| 3D scene FPS            | 60 fps   | 16.67ms per frame              |
| Gamepad poll rate       | 60 Hz    | 16.67ms interval               |
| Round-trip latency (local) | <20ms | Includes encoding/decoding     |
| Memory usage (idle)     | <100 MB  | After initial load             |
| Memory growth (1hr)     | <50 MB   | With constant video streaming  |

Measure actual performance: **Don't guess, measure!**
