import { Component, type ReactNode, type ErrorInfo } from "react";
import { errorTracker } from "@/lib/errorTracking";

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false, error: null };

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    errorTracker.capture(error, {
      componentStack: info.componentStack ?? undefined,
    });
  }

  render() {
    if (this.state.hasError) {
      return (
        this.props.fallback ?? (
          <div style={{ padding: "2rem", fontFamily: "monospace" }}>
            <h1>Something went wrong</h1>
            <pre style={{ whiteSpace: "pre-wrap", color: "#ef4444" }}>
              {this.state.error?.message}
            </pre>
            <button
              onClick={() => this.setState({ hasError: false, error: null })}
            >
              Try again
            </button>
          </div>
        )
      );
    }
    return this.props.children;
  }
}
