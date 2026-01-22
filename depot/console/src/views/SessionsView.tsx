import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import {
  VideoCamera,
  ArrowClockwise,
  Download,
  ArrowRight,
  Cube,
  Camera,
  MapPin,
  Robot,
  CheckCircle,
  CircleNotch,
  Warning,
  CheckCircle as CheckCircleFill,
} from "@phosphor-icons/react";

interface Session {
  name: string;
  size: number;
  modified: number;
  rover_id?: string;
  pose_count?: number;
  lidar_frame_count?: number;
  camera_frame_count?: number;
  has_gps?: boolean;
  extracted?: boolean;
  duration_secs?: number;
}

interface MapperStatus {
  status: string;
  sessions: number;
  maps: number;
  pending_extractions: number;
  splat_queue: {
    queued: number;
    processing: number;
    completed: number;
    failed: number;
  };
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

function formatTimeAgo(timestamp: number): string {
  const now = Date.now();
  const then = timestamp * 1000;
  const diff = now - then;

  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);
  const weeks = Math.floor(days / 7);
  const months = Math.floor(days / 30);

  if (seconds < 60) return "just now";
  if (minutes < 60) return `${minutes}m ago`;
  if (hours < 24) return `${hours}h ago`;
  if (days < 7) return `${days}d ago`;
  if (weeks < 4) return `${weeks}w ago`;
  return `${months}mo ago`;
}

function formatDuration(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  return `${hours}h ${mins}m`;
}

export function SessionsView() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [mapperStatus, setMapperStatus] = useState<MapperStatus | null>(null);
  const [statusError, setStatusError] = useState(false);

  const fetchSessions = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch("/api/mapper/sessions");
      if (!response.ok) {
        throw new Error(`Failed to fetch sessions: ${response.status}`);
      }
      const data = await response.json();
      setSessions(data.sessions || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch sessions");
    } finally {
      setLoading(false);
    }
  };

  const fetchStatus = async () => {
    try {
      const response = await fetch("/api/mapper/status");
      if (!response.ok) {
        setStatusError(true);
        return;
      }
      const data = await response.json();
      setMapperStatus(data);
      setStatusError(false);
    } catch {
      setStatusError(true);
    }
  };

  useEffect(() => {
    fetchSessions();
    fetchStatus();
    // Poll status every 10 seconds
    const interval = setInterval(fetchStatus, 10000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="min-h-full p-6">
      <div className="max-w-4xl mx-auto space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">Sessions</h1>
            <p className="text-muted-foreground">
              Recorded telemetry from rover operations
            </p>
          </div>
          <button
            onClick={() => { fetchSessions(); fetchStatus(); }}
            disabled={loading}
            className="flex items-center gap-2 px-3 py-2 text-sm bg-muted hover:bg-muted/80 rounded-md transition-colors disabled:opacity-50"
          >
            <ArrowClockwise className={`h-4 w-4 ${loading ? "animate-spin" : ""}`} />
            Refresh
          </button>
        </div>

        {/* Mapper Status Card */}
        <div className="bg-muted/30 border border-border rounded-lg p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-6">
              {/* Service Status */}
              <div className="flex items-center gap-2">
                {statusError ? (
                  <>
                    <div className="h-2 w-2 rounded-full bg-red-500" />
                    <span className="text-sm text-muted-foreground">Mapper Offline</span>
                  </>
                ) : (
                  <>
                    <div className="h-2 w-2 rounded-full bg-green-500" />
                    <span className="text-sm text-muted-foreground">Mapper Online</span>
                  </>
                )}
              </div>

              {mapperStatus && !statusError && (
                <>
                  {/* Extraction Queue */}
                  {(mapperStatus.pending_extractions ?? 0) > 0 && (
                    <div className="flex items-center gap-2 text-sm">
                      <CircleNotch className="h-4 w-4 text-blue-500 animate-spin" />
                      <span className="text-muted-foreground">
                        {mapperStatus.pending_extractions} pending extraction{mapperStatus.pending_extractions !== 1 ? 's' : ''}
                      </span>
                    </div>
                  )}

                  {/* Splat Queue */}
                  {(mapperStatus.splat_queue?.processing ?? 0) > 0 && (
                    <div className="flex items-center gap-2 text-sm">
                      <CircleNotch className="h-4 w-4 text-orange-500 animate-spin" />
                      <span className="text-muted-foreground">
                        {mapperStatus.splat_queue.processing} splatting
                      </span>
                    </div>
                  )}

                  {(mapperStatus.splat_queue?.queued ?? 0) > 0 && (
                    <div className="flex items-center gap-2 text-sm">
                      <Cube className="h-4 w-4 text-muted-foreground" />
                      <span className="text-muted-foreground">
                        {mapperStatus.splat_queue.queued} queued
                      </span>
                    </div>
                  )}

                  {(mapperStatus.splat_queue?.failed ?? 0) > 0 && (
                    <div className="flex items-center gap-2 text-sm">
                      <Warning className="h-4 w-4 text-red-500" />
                      <span className="text-red-500">
                        {mapperStatus.splat_queue.failed} failed
                      </span>
                    </div>
                  )}
                </>
              )}
            </div>

            {/* Maps count */}
            {mapperStatus && !statusError && (
              <Link
                to="/maps"
                className="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors"
              >
                <CheckCircleFill className="h-4 w-4 text-green-500" />
                {mapperStatus.maps} map{mapperStatus.maps !== 1 ? 's' : ''}
                <ArrowRight className="h-3 w-3" />
              </Link>
            )}
          </div>
        </div>

        {/* Error state */}
        {error && (
          <div className="bg-destructive/10 border border-destructive/20 text-destructive p-4 rounded-md">
            {error}
          </div>
        )}

        {/* Loading state */}
        {loading && sessions.length === 0 && (
          <div className="bg-muted/50 border border-border p-8 text-center">
            <ArrowClockwise className="h-8 w-8 mx-auto mb-4 text-muted-foreground animate-spin" />
            <p className="text-muted-foreground">Loading sessions...</p>
          </div>
        )}

        {/* Empty state */}
        {!loading && sessions.length === 0 && !error && (
          <div className="bg-muted/50 border border-border p-8 text-center">
            <VideoCamera className="h-12 w-12 mx-auto mb-4 text-muted-foreground opacity-50" />
            <h3 className="font-medium mb-2">No Sessions</h3>
            <p className="text-sm text-muted-foreground max-w-md mx-auto">
              Sessions will appear here when rovers sync their recordings to the depot.
            </p>
          </div>
        )}

        {/* Sessions list */}
        {sessions.length > 0 && (
          <div className="border border-border rounded-md overflow-hidden">
            <table className="w-full text-sm">
              <thead className="bg-muted/50">
                <tr>
                  <th className="text-left px-4 py-3 font-medium">Session</th>
                  <th className="text-center px-4 py-3 font-medium">Data</th>
                  <th className="text-right px-4 py-3 font-medium">Size</th>
                  <th className="text-right px-4 py-3 font-medium">Modified</th>
                  <th className="text-right px-4 py-3 font-medium w-20">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                {sessions.map((session) => (
                  <tr key={session.name} className="hover:bg-muted/30 transition-colors group">
                    <td className="px-4 py-3">
                      <Link
                        to={`/sessions/${encodeURIComponent(session.name)}`}
                        className="flex items-center gap-2 hover:text-foreground"
                      >
                        <VideoCamera className="h-4 w-4 text-muted-foreground" />
                        <div className="flex flex-col">
                          <span className="font-mono text-xs">{session.name}</span>
                          <div className="flex items-center gap-2 text-xs text-muted-foreground">
                            {session.rover_id && (
                              <span className="flex items-center gap-1">
                                <Robot className="h-3 w-3" />
                                {session.rover_id}
                              </span>
                            )}
                            {session.duration_secs && session.duration_secs > 0 && (
                              <span>{formatDuration(session.duration_secs)}</span>
                            )}
                          </div>
                        </div>
                      </Link>
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex items-center justify-center gap-2">
                        {session.extracted ? (
                          <>
                            {session.lidar_frame_count !== undefined && session.lidar_frame_count > 0 ? (
                              <span
                                className="flex items-center gap-1 text-xs text-green-600 dark:text-green-400"
                                title={`${session.lidar_frame_count.toLocaleString()} LiDAR frames`}
                              >
                                <Cube className="h-4 w-4" />
                                {session.lidar_frame_count.toLocaleString()}
                              </span>
                            ) : (
                              <span
                                className="flex items-center gap-1 text-xs text-muted-foreground"
                                title="No LiDAR data"
                              >
                                <Cube className="h-4 w-4 opacity-30" />
                              </span>
                            )}
                            {session.camera_frame_count !== undefined && session.camera_frame_count > 0 ? (
                              <span
                                className="flex items-center gap-1 text-xs text-blue-600 dark:text-blue-400"
                                title={`${session.camera_frame_count.toLocaleString()} camera frames`}
                              >
                                <Camera className="h-4 w-4" />
                                {session.camera_frame_count.toLocaleString()}
                              </span>
                            ) : (
                              <span
                                className="flex items-center gap-1 text-xs text-muted-foreground"
                                title="No camera data"
                              >
                                <Camera className="h-4 w-4 opacity-30" />
                              </span>
                            )}
                            {session.has_gps && (
                              <span
                                className="text-orange-600 dark:text-orange-400"
                                title="Has GPS coordinates"
                              >
                                <MapPin className="h-4 w-4" />
                              </span>
                            )}
                            <span
                              className="text-green-600 dark:text-green-400"
                              title="Extracted"
                            >
                              <CheckCircle className="h-4 w-4" />
                            </span>
                          </>
                        ) : (
                          <span className="text-xs text-muted-foreground">Not extracted</span>
                        )}
                      </div>
                    </td>
                    <td className="px-4 py-3 text-right text-muted-foreground">
                      {formatBytes(session.size)}
                    </td>
                    <td
                      className="px-4 py-3 text-right text-muted-foreground"
                      title={formatDate(session.modified)}
                    >
                      {formatTimeAgo(session.modified)}
                    </td>
                    <td className="px-4 py-3 text-right">
                      <div className="flex items-center justify-end gap-1">
                        <a
                          href={`/api/mapper/sessions/${session.name}`}
                          download
                          className="inline-flex items-center justify-center h-8 w-8 rounded hover:bg-muted transition-colors"
                          title="Download"
                          onClick={(e) => e.stopPropagation()}
                        >
                          <Download className="h-4 w-4" />
                        </a>
                        <Link
                          to={`/sessions/${encodeURIComponent(session.name)}`}
                          className="inline-flex items-center justify-center h-8 w-8 rounded hover:bg-muted transition-colors"
                          title="View in Rerun"
                        >
                          <ArrowRight className="h-4 w-4" />
                        </Link>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {/* Summary */}
        {sessions.length > 0 && (
          <p className="text-sm text-muted-foreground">
            {sessions.length} session{sessions.length !== 1 ? "s" : ""} •{" "}
            {formatBytes(sessions.reduce((acc, s) => acc + s.size, 0))} total
          </p>
        )}
      </div>
    </div>
  );
}
