import { useParams, Link } from "react-router-dom";
import { ArrowLeft, Download } from "@phosphor-icons/react";
import { useEffect, useState, useMemo } from "react";

interface Session {
  name: string;
  size: number;
  modified: number;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function formatDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString();
}

export function SessionDetailView() {
  const { sessionName } = useParams<{ sessionName: string }>();
  const [session, setSession] = useState<Session | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Memoize the viewer URL to prevent iframe reloads
  const viewerUrl = useMemo(() => {
    if (!sessionName) return null;
    const rrdUrl = `${window.location.origin}/api/mapper/sessions/${encodeURIComponent(sessionName)}.rrd`;
    return `/rerun/viewer.html?url=${encodeURIComponent(rrdUrl)}`;
  }, [sessionName]);

  // Fetch session metadata
  useEffect(() => {
    async function fetchSession() {
      if (!sessionName) return;

      try {
        const response = await fetch("/api/mapper/sessions");
        if (!response.ok) {
          throw new Error(`Failed to fetch sessions: ${response.status}`);
        }
        const data = await response.json();
        const found = data.sessions?.find((s: Session) => s.name === sessionName);
        if (found) {
          setSession(found);
        } else {
          setError("Session not found");
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to fetch session");
      }
    }

    fetchSession();
  }, [sessionName]);

  return (
    <div className="absolute inset-0 flex flex-col overflow-hidden">
      {/* Header */}
      <div className="flex-none flex items-center justify-between p-4 border-b border-border bg-background z-10">
        <div className="flex items-center gap-4">
          <Link
            to="/sessions"
            className="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors"
          >
            <ArrowLeft className="h-4 w-4" />
            Back to Sessions
          </Link>
          <div className="h-4 w-px bg-border" />
          <div>
            <h1 className="font-mono text-sm font-medium">{sessionName}</h1>
            {session && (
              <p className="text-xs text-muted-foreground">
                {formatBytes(session.size)} • {formatDate(session.modified)}
              </p>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2">
          {sessionName && (
            <a
              href={`/api/mapper/sessions/${sessionName}`}
              download
              className="flex items-center gap-2 px-3 py-1.5 text-sm bg-muted hover:bg-muted/80 rounded-md transition-colors"
            >
              <Download className="h-4 w-4" />
              Download
            </a>
          )}
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 min-h-0 relative overflow-hidden">
        {error && (
          <div className="absolute inset-0 flex items-center justify-center bg-background">
            <div className="text-center">
              <p className="text-destructive mb-2">{error}</p>
              <Link
                to="/sessions"
                className="text-sm text-muted-foreground hover:text-foreground"
              >
                Return to sessions list
              </Link>
            </div>
          </div>
        )}

        {viewerUrl && !error && (
          <iframe
            key={sessionName}
            src={viewerUrl}
            className="absolute inset-0 w-full h-full border-0"
            title={`Rerun Viewer - ${sessionName}`}
            allow="fullscreen"
          />
        )}
      </div>
    </div>
  );
}
