/**
 * Web Worker for sending teleop commands at a consistent rate.
 *
 * Web Workers run on a separate thread, immune to main thread blocking
 * from React re-renders, garbage collection, or other JavaScript work.
 */

// Message types from main thread
interface WorkerMessage {
  type: 'start' | 'stop' | 'updateInput';
  wsUrl?: string;
  input?: InputState;
}

interface InputState {
  linear: number;
  angular: number;
  boost: boolean;
  estop: boolean;
  toolAxis: number;
  actionA: boolean;
  actionB: boolean;
}

// Protocol constants
const MSG_TWIST = 0x01;
const MSG_ESTOP = 0x02;
const MSG_HEARTBEAT = 0x03;
const MSG_TOOL = 0x05;

const COMMAND_INTERVAL_MS = 33; // ~30Hz
const HEARTBEAT_INTERVAL_MS = 100;
const SPEED_NORMAL = 3.0;
const SPEED_BOOST = 5.0;
const MAX_ANGULAR_VEL = 1.5;
const TOOL_DEADZONE = 0.01;

let ws: WebSocket | null = null;
let commandInterval: ReturnType<typeof setInterval> | null = null;
let heartbeatInterval: ReturnType<typeof setInterval> | null = null;
let currentInput: InputState = {
  linear: 0,
  angular: 0,
  boost: false,
  estop: false,
  toolAxis: 0,
  actionA: false,
  actionB: false,
};

function encodeTwist(linear: number, angular: number, boost: boolean): ArrayBuffer {
  const buf = new ArrayBuffer(26);
  const view = new DataView(buf);
  view.setUint8(0, MSG_TWIST);
  view.setFloat64(1, linear, true);
  view.setFloat64(9, angular, true);
  view.setUint8(17, boost ? 1 : 0);
  view.setBigUint64(18, BigInt(Date.now()), true);
  return buf;
}

function encodeEStop(): ArrayBuffer {
  const buf = new ArrayBuffer(1);
  new DataView(buf).setUint8(0, MSG_ESTOP);
  return buf;
}

function encodeHeartbeat(): ArrayBuffer {
  const buf = new ArrayBuffer(1);
  new DataView(buf).setUint8(0, MSG_HEARTBEAT);
  return buf;
}

function encodeTool(axis: number, motor: number, actionA: boolean, actionB: boolean): ArrayBuffer {
  const buf = new ArrayBuffer(11);
  const view = new DataView(buf);
  view.setUint8(0, MSG_TOOL);
  view.setFloat32(1, axis, true);
  view.setFloat32(5, motor, true);
  view.setUint8(9, actionA ? 1 : 0);
  view.setUint8(10, actionB ? 1 : 0);
  return buf;
}

function sendCommands() {
  if (!ws || ws.readyState !== WebSocket.OPEN) return;

  const input = currentInput;

  // E-Stop takes priority
  if (input.estop) {
    ws.send(encodeEStop());
    return;
  }

  // Send velocity command
  const speedMultiplier = input.boost ? SPEED_BOOST : SPEED_NORMAL;
  const linear = input.linear * speedMultiplier;
  const angular = input.angular * MAX_ANGULAR_VEL;
  ws.send(encodeTwist(linear, angular, input.boost));

  // Send tool command if active
  if (Math.abs(input.toolAxis) > TOOL_DEADZONE || input.actionA || input.actionB) {
    ws.send(encodeTool(input.toolAxis, 0, input.actionA, input.actionB));
  }
}

function start(wsUrl: string) {
  stop();

  ws = new WebSocket(wsUrl);
  ws.binaryType = 'arraybuffer';

  ws.onopen = () => {
    self.postMessage({ type: 'connected' });

    // Send initial zero command
    ws!.send(encodeTwist(0, 0, false));
    ws!.send(encodeHeartbeat());

    // Start command loop
    commandInterval = setInterval(sendCommands, COMMAND_INTERVAL_MS);

    // Start heartbeat loop
    heartbeatInterval = setInterval(() => {
      if (ws?.readyState === WebSocket.OPEN) {
        ws.send(encodeHeartbeat());
      }
    }, HEARTBEAT_INTERVAL_MS);
  };

  ws.onmessage = (event) => {
    // Forward telemetry to main thread
    if (event.data instanceof ArrayBuffer) {
      // Clone the buffer since we can't transfer and keep it
      const copy = event.data.slice(0);
      self.postMessage({ type: 'telemetry', data: copy });
    }
  };

  ws.onclose = () => {
    self.postMessage({ type: 'disconnected' });
    stop();
  };

  ws.onerror = () => {
    self.postMessage({ type: 'error' });
  };
}

function stop() {
  if (commandInterval) {
    clearInterval(commandInterval);
    commandInterval = null;
  }
  if (heartbeatInterval) {
    clearInterval(heartbeatInterval);
    heartbeatInterval = null;
  }
  if (ws) {
    ws.close();
    ws = null;
  }
}

// Handle messages from main thread
self.onmessage = (event: MessageEvent<WorkerMessage>) => {
  const { type } = event.data;

  switch (type) {
    case 'start':
      if (event.data.wsUrl) {
        start(event.data.wsUrl);
      }
      break;
    case 'stop':
      stop();
      break;
    case 'updateInput':
      if (event.data.input) {
        currentInput = event.data.input;
      }
      break;
  }
};
