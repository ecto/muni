/**
 * Client-side error tracking with session correlation IDs.
 *
 * Captures unhandled errors and promise rejections, buffers them, and
 * periodically flushes to `/api/errors`. Each browser session gets a
 * stable UUID so errors can be correlated with rover telemetry sessions.
 *
 * The `/api/errors` endpoint does not exist yet on the backend -- this
 * module is the first step toward structured error collection.
 */

interface ErrorEvent {
  message: string;
  stack?: string;
  context?: Record<string, unknown>;
  timestamp: string;
  sessionId: string;
  url: string;
  userAgent: string;
}

class ErrorTracker {
  private sessionId: string;
  private buffer: ErrorEvent[] = [];
  private flushInterval: number | null = null;

  constructor() {
    this.sessionId = crypto.randomUUID();
    this.setupGlobalHandlers();
  }

  private setupGlobalHandlers() {
    // Catch unhandled errors
    window.addEventListener("error", (event) => {
      this.capture(event.error ?? new Error(event.message), {
        filename: event.filename,
        lineno: event.lineno,
        colno: event.colno,
      });
    });

    // Catch unhandled promise rejections
    window.addEventListener("unhandledrejection", (event) => {
      const error =
        event.reason instanceof Error
          ? event.reason
          : new Error(String(event.reason));
      this.capture(error, { type: "unhandledrejection" });
    });
  }

  capture(error: Error, context?: Record<string, unknown>) {
    const event: ErrorEvent = {
      message: error.message,
      stack: error.stack,
      context,
      timestamp: new Date().toISOString(),
      sessionId: this.sessionId,
      url: window.location.href,
      userAgent: navigator.userAgent,
    };

    this.buffer.push(event);

    // Also log to console for dev visibility
    console.error("[ErrorTracker]", error.message, context);

    // Flush if buffer exceeds threshold
    if (this.buffer.length >= 10) {
      this.flush();
    }
  }

  getSessionId(): string {
    return this.sessionId;
  }

  getBufferedEvents(): readonly ErrorEvent[] {
    return this.buffer;
  }

  private async flush() {
    if (this.buffer.length === 0) return;
    const events = [...this.buffer];
    this.buffer = [];

    try {
      // POST to console backend error endpoint
      await fetch("/api/errors", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ events }),
      });
    } catch {
      // If reporting fails, put events back (up to a limit)
      if (this.buffer.length < 100) {
        this.buffer.unshift(...events);
      }
    }
  }

  startPeriodicFlush(intervalMs = 30_000) {
    this.flushInterval = window.setInterval(() => this.flush(), intervalMs);
  }

  stopPeriodicFlush() {
    if (this.flushInterval !== null) {
      clearInterval(this.flushInterval);
      this.flushInterval = null;
    }
  }
}

export const errorTracker = new ErrorTracker();
