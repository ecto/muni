/**
 * Mutable video frame store - bypasses React for performance.
 *
 * Video frames arrive at ~15fps per camera. Storing them in React state
 * (via setCameraFrame) triggers re-renders on every frame, which is
 * wasteful since:
 * - The 3D scene reads frames in useFrame at 60fps
 * - HTML img elements only need updates when src changes
 *
 * This module provides a mutable store pattern (like interpolation.ts)
 * where frames are stored outside React and read directly by consumers.
 *
 * Components can use the getter function in useFrame or poll via
 * setInterval for HTML elements.
 */

/** Frame data for a single camera. */
export interface CameraFrame {
  url: string;
  timestamp: number;
}

/** Internal store state. */
interface VideoFrameState {
  /** Frames from each camera, keyed by camera ID */
  frames: Map<number, CameraFrame>;
  /** Version counter - incremented on every update for change detection */
  version: number;
  /** Timestamp of last update (performance.now()) */
  lastUpdateTime: number;
}

/**
 * Global video frame store - bypasses React for performance.
 * Updated at ~15fps per camera from WebRTC handler.
 * Read at 60fps from useFrame or via polling for HTML elements.
 */
const globalStore: VideoFrameState = {
  frames: new Map(),
  version: 0,
  lastUpdateTime: 0,
};

/** Pending blob URLs to revoke after delay. */
const pendingRevocations: Array<{ url: string; revokeAt: number }> = [];

/** Delay before revoking blob URLs (ms). Allows image loading to complete. */
const REVOCATION_DELAY_MS = 200;

/**
 * Set a camera frame in the store.
 * Handles blob URL lifecycle (revokes old URLs after delay).
 *
 * @param cameraId Camera identifier
 * @param url Blob URL for the frame
 * @param timestamp Frame timestamp in ms
 */
/** Track if we've logged the first frame */
let firstFrameLogged = false;

export function setCameraFrame(
  cameraId: number,
  url: string,
  timestamp: number
): void {
  const now = performance.now();

  // Log first frame for debugging
  if (!firstFrameLogged) {
    console.log(`[videoFrameStore] First frame stored: camera=${cameraId}, version=${globalStore.version + 1}`);
    firstFrameLogged = true;
  }

  // Schedule old URL for revocation
  const oldFrame = globalStore.frames.get(cameraId);
  if (oldFrame) {
    pendingRevocations.push({
      url: oldFrame.url,
      revokeAt: now + REVOCATION_DELAY_MS,
    });
  }

  // Store new frame
  globalStore.frames.set(cameraId, { url, timestamp });
  globalStore.version++;
  globalStore.lastUpdateTime = now;

  // Process pending revocations
  processPendingRevocations(now);
}

/**
 * Process any pending blob URL revocations that are ready.
 */
function processPendingRevocations(now: number): void {
  while (pendingRevocations.length > 0 && pendingRevocations[0].revokeAt <= now) {
    const { url } = pendingRevocations.shift()!;
    URL.revokeObjectURL(url);
  }
}

/**
 * Get a camera frame from the store.
 *
 * @param cameraId Camera identifier
 * @returns Frame data or undefined if not present
 */
export function getCameraFrame(cameraId: number): CameraFrame | undefined {
  return globalStore.frames.get(cameraId);
}

/**
 * Get all camera frames from the store.
 * Returns a new Map to avoid external mutation.
 *
 * @returns Map of camera ID to frame data
 */
export function getAllCameraFrames(): Map<number, CameraFrame> {
  return new Map(globalStore.frames);
}

/**
 * Get the number of cameras with frames.
 */
export function getCameraCount(): number {
  return globalStore.frames.size;
}

/**
 * Get the current version number for change detection.
 * Increments on every frame update.
 */
export function getFrameVersion(): number {
  return globalStore.version;
}

/**
 * Get the latest frame URL and timestamp (for backwards compatibility).
 * Returns the most recently updated frame across all cameras.
 */
export function getLatestFrame(): { url: string | null; timestamp: number } {
  let latest: CameraFrame | null = null;
  for (const frame of globalStore.frames.values()) {
    if (!latest || frame.timestamp > latest.timestamp) {
      latest = frame;
    }
  }
  return {
    url: latest?.url ?? null,
    timestamp: latest?.timestamp ?? 0,
  };
}

// ============================================================================
// Camera Calibration Store
// ============================================================================

/** Camera calibration data received from MSG_CAMERA_INFO. */
export interface CameraCalibration {
  width: number;
  height: number;
  /** Mount position [forward, left, up] in meters */
  position: [number, number, number];
  /** Mount rotation [roll, pitch, yaw] in radians */
  rotation: [number, number, number];
  /** Horizontal field of view (radians) */
  fovH: number;
  /** Vertical field of view (radians) */
  fovV: number;
}

/** Map of camera ID to calibration data. */
const calibrations = new Map<number, CameraCalibration>();

/** Store calibration for a camera. */
export function setCameraCalibration(
  cameraId: number,
  calibration: CameraCalibration,
): void {
  calibrations.set(cameraId, calibration);
}

/** Get calibration for a camera. */
export function getCameraCalibration(
  cameraId: number,
): CameraCalibration | undefined {
  return calibrations.get(cameraId);
}

/** Clear all calibrations (on disconnect). */
export function clearCalibrations(): void {
  calibrations.clear();
}

/**
 * Clear all frames from the store.
 * Used on disconnect to clean up resources.
 */
export function clearAllFrames(): void {
  // Revoke all blob URLs
  for (const frame of globalStore.frames.values()) {
    URL.revokeObjectURL(frame.url);
  }
  globalStore.frames.clear();
  videoStreams.clear();
  globalStore.version++;
  globalStore.lastUpdateTime = performance.now();

  // Clear pending revocations (URLs are already revoked)
  pendingRevocations.length = 0;
}

// ============================================================================
// MediaStream Storage (for H.264 WebRTC video tracks)
// ============================================================================

/** Map of camera ID to MediaStream from WebRTC video tracks. */
const videoStreams = new Map<number, MediaStream>();

/** Store a video stream for a camera. */
export function setVideoStream(cameraId: number, stream: MediaStream): void {
  videoStreams.set(cameraId, stream);
  globalStore.version++;
  globalStore.lastUpdateTime = performance.now();
}

/** Get a video stream for a camera. */
export function getVideoStream(cameraId: number): MediaStream | undefined {
  return videoStreams.get(cameraId);
}

/** Remove a video stream. */
export function removeVideoStream(cameraId: number): void {
  videoStreams.delete(cameraId);
  globalStore.version++;
  globalStore.lastUpdateTime = performance.now();
}

/** Get all video streams. */
export function getAllVideoStreams(): Map<number, MediaStream> {
  return new Map(videoStreams);
}

/** Get the number of active video streams. */
export function getVideoStreamCount(): number {
  return videoStreams.size;
}

/** Map of camera ID to the actively-decoding HTMLVideoElement (owned by sidebar). */
const videoElements = new Map<number, HTMLVideoElement>();

/** Register a visible video element (called by CameraFeed when mounted). */
export function setVideoElement(cameraId: number, el: HTMLVideoElement): void {
  videoElements.set(cameraId, el);
}

/** Unregister a video element (called by CameraFeed on unmount). */
export function removeVideoElement(cameraId: number): void {
  videoElements.delete(cameraId);
}

/** Get the actively-decoding video element for a camera. */
export function getVideoElement(cameraId: number): HTMLVideoElement | undefined {
  return videoElements.get(cameraId);
}
