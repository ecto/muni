# Documentation Patterns by Language

Language-specific documentation patterns and best practices for Rust and TypeScript in the Muni project.

## Rust Documentation

### Module-Level Documentation (`//!`)

**Location**: Top of `lib.rs` files

**Structure**:
```rust
//! Brief one-line description of the module.
//!
//! Longer description explaining the purpose, architecture, and key concepts.
//! This section can span multiple paragraphs.
//!
//! # Examples
//!
//! ```
//! use my_crate::Thing;
//!
//! let thing = Thing::new();
//! thing.do_work();
//! ```
//!
//! # Architecture
//!
//! Explanation of how the module fits into the larger system.
//!
//! # Safety
//!
//! Important safety considerations (if any unsafe code exists).
```

**Example**:
```rust
//! State machine and mode management for bvr.
//!
//! This module implements the rover's operational modes with safety-critical
//! transition logic. Modes include Disabled, Idle, Teleop, Autonomous, EStop,
//! and Fault. The state machine ensures that:
//!
//! - E-stop requires explicit release
//! - Invalid transitions are rejected
//! - Mode changes are logged
//! - LED feedback is synchronized
//!
//! # Examples
//!
//! ```
//! use bvr_state::{StateMachine, Event, Mode};
//!
//! let mut sm = StateMachine::new();
//! assert_eq!(sm.mode(), Mode::Disabled);
//!
//! sm.handle(Event::Enable);
//! assert_eq!(sm.mode(), Mode::Idle);
//! ```
//!
//! # Safety
//!
//! E-stop transitions are one-way and require explicit `Event::EStopRelease`.
//! Always check `is_driving()` before sending motor commands.
```

### Function Documentation (`///`)

**Structure**:
```rust
/// Brief one-line summary (ends with period).
///
/// Detailed explanation of what the function does, when to use it,
/// and any important considerations.
///
/// # Arguments
///
/// * `param1` - Description of first parameter
/// * `param2` - Description of second parameter
///
/// # Returns
///
/// Description of return value and what it represents.
///
/// # Errors
///
/// When this function returns an error and what the error means.
///
/// # Panics
///
/// Conditions under which this function will panic (if any).
///
/// # Examples
///
/// ```
/// use my_crate::my_function;
///
/// let result = my_function(42, "hello");
/// assert_eq!(result, expected);
/// ```
///
/// # Safety
///
/// Safety invariants (for unsafe functions only).
pub fn my_function(param1: i32, param2: &str) -> Result<Output, Error> {
    // ...
}
```

**Real Example**:
```rust
/// Handles a state machine event and transitions to a new mode if valid.
///
/// Events are processed according to the current mode and transition rules.
/// Invalid transitions are logged and ignored. All successful transitions
/// update LED feedback and log both old and new modes.
///
/// # Arguments
///
/// * `event` - The event to process (Enable, Disable, EStop, etc.)
///
/// # Examples
///
/// ```
/// use bvr_state::{StateMachine, Event, Mode};
///
/// let mut sm = StateMachine::new();
/// sm.handle(Event::Enable);
/// assert_eq!(sm.mode(), Mode::Idle);
///
/// sm.handle(Event::Enable);  // Idle → Teleop
/// assert_eq!(sm.mode(), Mode::Teleop);
/// ```
///
/// # Safety
///
/// E-stop events are accepted from any mode and always succeed. They
/// require explicit `Event::EStopRelease` to exit.
pub fn handle(&mut self, event: Event) {
    match (self.mode, event) {
        // ...
    }
}
```

### Struct/Enum Documentation

**Structure**:
```rust
/// Brief description of the type.
///
/// Detailed explanation of what this type represents, when to use it,
/// and important considerations.
///
/// # Examples
///
/// ```
/// let instance = MyStruct::new();
/// ```
///
/// # Fields (for structs with pub fields)
///
/// * `field1` - Description
/// * `field2` - Description
#[derive(Debug, Clone)]
pub struct MyStruct {
    /// Description of this field
    pub field1: i32,
    /// Description of this field
    pub field2: String,
}

/// Brief description of the enum.
///
/// Detailed explanation of the different variants and when each is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MyEnum {
    /// First variant description
    VariantOne,
    /// Second variant description with associated data
    VariantTwo(i32),
    /// Third variant
    VariantThree { field: String },
}
```

**Real Example**:
```rust
/// Represents the rover's current operational mode.
///
/// The mode determines which operations are permitted. Use `is_driving()`
/// to check if motor commands should be sent, and `is_safe()` to check
/// if configuration changes are allowed.
///
/// # Mode Transitions
///
/// Valid transitions:
/// - Disabled → Idle (via Enable)
/// - Idle → Teleop (via Enable)
/// - Teleop → Autonomous (via Autonomous)
/// - Any → EStop (via EStop)
/// - EStop → Idle (via EStopRelease)
///
/// # Examples
///
/// ```
/// use bvr_state::Mode;
///
/// let mode = Mode::Teleop;
/// assert!(mode.is_driving());
/// assert!(!mode.is_safe());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Motors disabled, awaiting initialization
    Disabled,
    /// Ready state, motors enabled but stationary
    Idle,
    /// Under human control via teleop
    Teleop,
    /// Autonomous navigation active
    Autonomous,
    /// Emergency stop, requires explicit release
    EStop,
    /// Error state, requires fault clear
    Fault,
}
```

### Error Type Documentation

**Pattern**:
```rust
/// Error type for [module name] operations.
///
/// These errors can occur during [common operations].
#[derive(Error, Debug)]
pub enum MyError {
    /// Brief description of when this error occurs.
    ///
    /// Additional context about the error condition.
    #[error("Display message: {0}")]
    VariantOne(String),

    /// Another error variant.
    #[error("Network error")]
    Network(#[from] std::io::Error),
}
```

**Real Example**:
```rust
/// Error type for CAN bus operations.
///
/// These errors can occur during frame transmission, reception, or parsing.
#[derive(Error, Debug)]
pub enum CanError {
    /// Socket-level communication error with the CAN interface.
    ///
    /// This typically indicates a hardware or driver issue.
    #[error("Socket error: {0}")]
    Socket(String),

    /// CAN ID is outside valid range (0x000-0x7FF for standard, 0x00000000-0x1FFFFFFF for extended).
    #[error("Invalid CAN ID: {0}")]
    InvalidId(u32),

    /// Timeout waiting for CAN frame reception.
    ///
    /// The configured timeout (typically 10ms) elapsed without receiving data.
    #[error("Timeout waiting for response")]
    Timeout,

    /// Received CAN frame has invalid format or data length.
    #[error("Invalid frame data")]
    InvalidFrame,
}
```

### Safety Documentation

**For `unsafe` functions**:
```rust
/// Reads memory-mapped hardware register.
///
/// # Safety
///
/// Caller must ensure:
/// - The address is valid and mapped
/// - No other code is concurrently accessing the same register
/// - The register is safe to read (no side effects)
///
/// # Examples
///
/// ```
/// unsafe {
///     let value = read_register(0x4000_0000);
/// }
/// ```
pub unsafe fn read_register(addr: usize) -> u32 {
    // ...
}
```

### Testing Documentation

Document test functions briefly:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Test that watchdog times out after configured duration.
    #[test]
    fn test_watchdog_timeout() {
        let mut wd = Watchdog::new(Duration::from_millis(100));
        assert!(wd.is_timed_out());

        wd.feed();
        assert!(!wd.is_timed_out());

        std::thread::sleep(Duration::from_millis(150));
        assert!(wd.is_timed_out());
    }

    /// Test that e-stop requires explicit release.
    #[test]
    fn test_estop_requires_release() {
        let mut sm = StateMachine::new();
        sm.handle(Event::Enable);  // Idle → Teleop

        sm.handle(Event::EStop);
        assert_eq!(sm.mode(), Mode::EStop);

        // Try to enable (should stay in EStop)
        sm.handle(Event::Enable);
        assert_eq!(sm.mode(), Mode::EStop);

        // Release
        sm.handle(Event::EStopRelease);
        assert_eq!(sm.mode(), Mode::Idle);
    }
}
```

## TypeScript/React Documentation

### Function/Hook Documentation (JSDoc)

**Structure**:
```typescript
/**
 * Brief one-line summary.
 *
 * Detailed explanation of what the function does and when to use it.
 *
 * @param paramName - Description of parameter
 * @param anotherParam - Description with type info
 * @returns Description of return value
 * @throws {ErrorType} When this function throws
 *
 * @example
 * ```typescript
 * const result = myFunction("hello", 42);
 * console.log(result);
 * ```
 */
export function myFunction(paramName: string, anotherParam: number): Result {
  // ...
}
```

**Real Example**:
```typescript
/**
 * Manages WebSocket connection to rover for real-time teleoperation.
 *
 * Handles binary protocol encoding/decoding, automatic reconnection with
 * exponential backoff (up to 30s max), and connection state management.
 * Commands are sent at 100Hz, telemetry received at 20Hz.
 *
 * @param address - WebSocket address (e.g., "ws://rover.local:4850")
 * @returns Connection state and send function
 *
 * @example
 * ```typescript
 * const { connected, send, latency } = useRoverConnection("ws://192.168.1.100:4850");
 *
 * if (connected) {
 *   const twist = { linear: 1.0, angular: 0.0, boost: false };
 *   send(encodeTwist(twist));
 * }
 * ```
 */
export function useRoverConnection(address: string) {
  const [ws, setWs] = useState<WebSocket | null>(null);
  const [connected, setConnected] = useState(false);
  const [latency, setLatency] = useState(0);

  // ... implementation
}
```

### Component Documentation

**Structure**:
```typescript
/**
 * Brief description of component.
 *
 * Detailed explanation of what the component renders and its purpose.
 *
 * @example
 * ```tsx
 * <MyComponent
 *   prop1="value"
 *   prop2={42}
 * />
 * ```
 */
export function MyComponent({ prop1, prop2 }: MyComponentProps) {
  // ...
}
```

**Real Example**:
```typescript
/**
 * Displays real-time rover telemetry including mode, pose, velocity, and power status.
 *
 * Updates at 20Hz via Zustand store subscription. Shows current operational mode
 * with color-coded badges, position in world frame, velocity with boost indicator,
 * and battery voltage with percentage calculation.
 *
 * @param className - Optional Tailwind classes for styling
 * @param showAdvanced - Whether to show detailed motor/controller data
 *
 * @example
 * ```tsx
 * <TelemetryPanel className="w-64" showAdvanced={true} />
 * ```
 */
export function TelemetryPanel({ className, showAdvanced = false }: TelemetryPanelProps) {
  const { mode, pose, velocity, power } = useConsoleStore((state) => ({
    mode: state.mode,
    pose: state.pose,
    velocity: state.velocity,
    power: state.power,
  }));

  return (
    <Card className={cn("p-4", className)}>
      {/* ... */}
    </Card>
  );
}
```

### Interface/Type Documentation

**Structure**:
```typescript
/**
 * Brief description of the interface.
 *
 * Detailed explanation of what this type represents and when it's used.
 *
 * @example
 * ```typescript
 * const data: MyInterface = {
 *   field1: "value",
 *   field2: 42,
 * };
 * ```
 */
export interface MyInterface {
  /** Description of field1 */
  field1: string;
  /** Description of field2 with additional context */
  field2: number;
  /** Optional field with default behavior */
  field3?: boolean;
}
```

**Real Example**:
```typescript
/**
 * Telemetry data received from rover at 20 Hz.
 *
 * Contains current operational mode, pose (position and heading), velocity
 * (linear and angular), power status (battery voltage/current), and temperature
 * readings for all motors and controllers. Data is transmitted via WebSocket
 * using binary protocol (MSG_TELEMETRY 0x10) with 92 bytes minimum payload.
 *
 * @example
 * ```typescript
 * const telemetry: Telemetry = {
 *   mode: Mode.Teleop,
 *   pose: { x: 10.5, y: 3.2, theta: 1.57 },
 *   velocity: { linear: 1.0, angular: 0.0, boost: false },
 *   power: { voltage: 48.6, current: 12.3 },
 *   temperatures: {
 *     motors: [45.2, 46.1, 44.8, 45.5],
 *     controllers: [38.1, 39.2, 38.5, 38.9],
 *   },
 *   erpm: [5000, 5100, 4950, 5050],
 *   gps_fix: 3,
 * };
 * ```
 */
export interface Telemetry {
  /** Current operational mode (Idle, Teleop, Autonomous, EStop, Fault) */
  mode: Mode;
  /** Position (x, y in meters) and heading (theta in radians) in world frame */
  pose: Pose;
  /** Current velocity command (linear in m/s, angular in rad/s) */
  velocity: Twist;
  /** Battery voltage (V) and current (A) */
  power: PowerStatus;
  /** Motor and controller temperatures in °C */
  temperatures: TempStatus;
  /** Electrical RPM for each motor (FL, FR, RL, RR) */
  erpm: [number, number, number, number];
  /** GPS fix quality: 0=none, 1=2D, 2=3D, 3=RTK */
  gps_fix: number;
}
```

### Enum/Const Documentation

**Structure**:
```typescript
/**
 * Brief description of the constant set.
 *
 * Explanation of when to use each value.
 */
export const MyEnum = {
  /** Description of first value */
  Value1: 0,
  /** Description of second value */
  Value2: 1,
  /** Description of third value */
  Value3: 2,
} as const;

export type MyEnum = (typeof MyEnum)[keyof typeof MyEnum];
```

**Real Example**:
```typescript
/**
 * Operational modes for the rover.
 *
 * These modes determine which operations are permitted and control LED feedback.
 * Transitions between modes are managed by the state machine on the rover.
 *
 * - **Disabled**: Motors off, not initialized
 * - **Idle**: Ready, motors enabled but stationary
 * - **Teleop**: Human control via gamepad/keyboard
 * - **Autonomous**: Self-driving navigation
 * - **EStop**: Emergency stop, requires explicit release
 * - **Fault**: Error state, requires fault clear
 */
export const Mode = {
  /** Motors disabled, awaiting initialization (LED: red solid) */
  Disabled: 0,
  /** Ready state, motors enabled but stationary (LED: blue solid) */
  Idle: 1,
  /** Under human control via teleop (LED: green pulse, 2s period) */
  Teleop: 2,
  /** Autonomous navigation active (LED: cyan pulse, 1.5s period) */
  Autonomous: 3,
  /** Emergency stop, requires explicit release (LED: red flash, 200ms) */
  EStop: 4,
  /** Error state, requires fault clear (LED: orange flash, 500ms) */
  Fault: 5,
} as const;

export type Mode = (typeof Mode)[keyof typeof Mode];

/**
 * Human-readable labels for rover modes.
 *
 * Use for UI display where numeric mode values need text representation.
 */
export const ModeLabels: Record<Mode, string> = {
  [Mode.Disabled]: "Disabled",
  [Mode.Idle]: "Idle",
  [Mode.Teleop]: "Teleop",
  [Mode.Autonomous]: "Autonomous",
  [Mode.EStop]: "E-Stop",
  [Mode.Fault]: "Fault",
};
```

### Custom Hook Documentation

**Pattern**:
```typescript
/**
 * Custom hook brief description.
 *
 * Detailed explanation of hook behavior, when to use it, and dependencies.
 *
 * @param dependency - Parameter that hook depends on
 * @returns Object with hook state and functions
 *
 * @example
 * ```typescript
 * function MyComponent() {
 *   const { state, action } = useMyHook(dependency);
 *   return <div onClick={action}>{state}</div>;
 * }
 * ```
 */
export function useMyHook(dependency: string) {
  // ...
}
```

## Documentation Anti-Patterns

### ❌ Don't Repeat Function Name
```rust
/// Sets the mode.  // ❌ Obvious from function name
pub fn set_mode(mode: Mode) { }
```

### ✅ Describe Purpose and Context
```rust
/// Transitions to the specified mode if the transition is valid.
///
/// Invalid transitions are logged and ignored. E-stop requires explicit release.
pub fn set_mode(mode: Mode) { }
```

---

### ❌ Don't State the Obvious
```typescript
/**
 * Gets the telemetry.  // ❌ Doesn't add value
 */
export function getTelemetry() { }
```

### ✅ Explain What It Does
```typescript
/**
 * Retrieves the most recent telemetry snapshot from the Zustand store.
 *
 * Returns cached data immediately (no network request). Use `useConsoleStore`
 * for reactive updates on telemetry changes.
 */
export function getTelemetry(): Telemetry { }
```

---

### ❌ Don't Document Implementation
```rust
/// Uses a match statement to handle events.  // ❌ Implementation detail
pub fn handle(&mut self, event: Event) { }
```

### ✅ Document Behavior
```rust
/// Processes a state machine event and transitions to a new mode if valid.
///
/// Invalid transitions are rejected and logged. E-stop events always succeed.
pub fn handle(&mut self, event: Event) { }
```

---

### ❌ Don't Copy-Paste Docs
```rust
/// Handles an event.  // ❌ Generic, could apply to anything
pub fn handle_estop(&mut self) { }

/// Handles an event.  // ❌ Identical to above
pub fn handle_enable(&mut self) { }
```

### ✅ Be Specific
```rust
/// Immediately transitions to EStop mode from any current mode.
///
/// This is a safety-critical operation that always succeeds.
pub fn handle_estop(&mut self) { }

/// Enables the rover, transitioning from Disabled to Idle.
///
/// This function validates that the rover is in Disabled mode before enabling.
pub fn handle_enable(&mut self) -> Result<(), StateError> { }
```

## References

- Rust Doc Comments: https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html
- JSDoc Reference: https://jsdoc.app/
- TSDoc: https://tsdoc.org/
