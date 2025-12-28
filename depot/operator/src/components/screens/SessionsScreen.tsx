import { useOperatorStore } from "@/store";
import { useSessions, getRerunViewerUrl } from "@/hooks/useSessions";
import { View, type Session } from "@/lib/types";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import {
  ArrowLeft,
  Calendar,
  Clock,
  MapPin,
  Robot,
  VideoCamera,
  Cube,
  Play,
  ArrowsClockwise,
  Warning,
  FunnelSimple,
} from "@phosphor-icons/react";
import { useState, useMemo } from "react";

function formatDuration(secs: number): string {
  if (secs < 60) {
    return `${Math.round(secs)}s`;
  }
  if (secs < 3600) {
    const mins = Math.floor(secs / 60);
    const remainingSecs = Math.round(secs % 60);
    return `${mins}m ${remainingSecs}s`;
  }
  const hours = Math.floor(secs / 3600);
  const mins = Math.floor((secs % 3600) / 60);
  return `${hours}h ${mins}m`;
}

function formatDate(isoString: string): string {
  const date = new Date(isoString);
  return date.toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

function formatTime(isoString: string): string {
  const date = new Date(isoString);
  return date.toLocaleTimeString("en-US", {
    hour: "numeric",
    minute: "2-digit",
  });
}

function SessionListItem({
  session,
  isSelected,
  onSelect,
  onPlayback,
}: {
  session: Session;
  isSelected: boolean;
  onSelect: () => void;
  onPlayback: () => void;
}) {
  const hasVideo = session.camera_frames > 0;
  const hasLidar = session.lidar_frames > 0;
  const hasGps = session.gps_bounds !== null;

  return (
    <div
      className={`p-4 cursor-pointer transition-colors border-l-2 ${
        isSelected
          ? "bg-accent border-l-primary"
          : "border-l-transparent hover:bg-accent/50"
      }`}
      onClick={onSelect}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="flex-1 min-w-0">
          {/* Date and time */}
          <div className="flex items-center gap-2 text-sm font-medium">
            <Calendar className="h-4 w-4 text-muted-foreground" weight="fill" />
            <span>{formatDate(session.started_at)}</span>
            <span className="text-muted-foreground">
              {formatTime(session.started_at)}
            </span>
          </div>

          {/* Rover and duration */}
          <div className="flex items-center gap-3 mt-1 text-xs text-muted-foreground">
            <span className="flex items-center gap-1">
              <Robot className="h-3 w-3" weight="fill" />
              {session.rover_id}
            </span>
            <span className="flex items-center gap-1">
              <Clock className="h-3 w-3" weight="fill" />
              {formatDuration(session.duration_secs)}
            </span>
          </div>
        </div>

        {/* Data badges */}
        <div className="flex gap-1">
          {hasVideo && (
            <Badge variant="outline" className="text-xs px-1.5">
              <VideoCamera className="h-3 w-3 mr-1" weight="fill" />
              {session.camera_frames}
            </Badge>
          )}
          {hasLidar && (
            <Badge variant="outline" className="text-xs px-1.5">
              <Cube className="h-3 w-3 mr-1" weight="fill" />
              {session.lidar_frames}
            </Badge>
          )}
        </div>
      </div>

      {/* Stats row */}
      <div className="flex items-center gap-3 mt-2 text-xs text-muted-foreground">
        <span>{session.pose_samples.toLocaleString()} poses</span>
        {hasGps && (
          <span className="flex items-center gap-1">
            <MapPin className="h-3 w-3" weight="fill" />
            GPS
          </span>
        )}
      </div>

      {/* Playback button when selected */}
      {isSelected && (
        <Button
          className="w-full mt-3 gap-2"
          size="sm"
          onClick={(e) => {
            e.stopPropagation();
            onPlayback();
          }}
        >
          <Play className="h-4 w-4" weight="fill" />
          Open in Rerun Viewer
        </Button>
      )}
    </div>
  );
}

function EmptyState({ message }: { message: string }) {
  return (
    <div className="flex-1 flex items-center justify-center p-8">
      <div className="text-center text-muted-foreground">
        <VideoCamera className="h-12 w-12 mx-auto mb-3 opacity-50" weight="thin" />
        <p className="text-sm">{message}</p>
      </div>
    </div>
  );
}

function ErrorState({ message, onRetry }: { message: string; onRetry: () => void }) {
  return (
    <div className="flex-1 flex items-center justify-center p-8">
      <div className="text-center">
        <Warning className="h-12 w-12 mx-auto mb-3 text-destructive" weight="fill" />
        <p className="text-sm text-muted-foreground mb-4">{message}</p>
        <Button variant="outline" size="sm" onClick={onRetry}>
          <ArrowsClockwise className="h-4 w-4 mr-2" />
          Retry
        </Button>
      </div>
    </div>
  );
}

export function SessionsScreen() {
  const { setView, selectedSessionId, selectSession, sessionRoverFilter, setSessionRoverFilter } =
    useOperatorStore();
  const { sessions, loading, error, refresh } = useSessions();
  const [showFilters, setShowFilters] = useState(false);

  // Get unique rover IDs from sessions
  const roverIds = useMemo(() => {
    const ids = new Set(sessions.map((s) => s.rover_id));
    return Array.from(ids).sort();
  }, [sessions]);

  // Filter sessions by selected rover
  const filteredSessions = useMemo(() => {
    if (!sessionRoverFilter) return sessions;
    return sessions.filter((s) => s.rover_id === sessionRoverFilter);
  }, [sessions, sessionRoverFilter]);

  const handlePlayback = (session: Session) => {
    // Open Rerun viewer in new tab
    const url = getRerunViewerUrl(session);
    window.open(url, "_blank", "noopener,noreferrer");
  };

  const handleBack = () => {
    setView(View.Home);
  };

  // Group sessions by date
  const groupedSessions = useMemo(() => {
    const groups: { [date: string]: Session[] } = {};
    for (const session of filteredSessions) {
      const date = formatDate(session.started_at);
      if (!groups[date]) {
        groups[date] = [];
      }
      groups[date].push(session);
    }
    return groups;
  }, [filteredSessions]);

  const totalDuration = useMemo(() => {
    return filteredSessions.reduce((sum, s) => sum + s.duration_secs, 0);
  }, [filteredSessions]);

  return (
    <div className="h-screen w-screen overflow-hidden bg-background flex dark">
      {/* Session list panel */}
      <div className="w-96 h-full flex flex-col border-r border-border bg-card">
        {/* Header */}
        <div className="p-4 border-b border-border">
          <div className="flex items-center gap-2 mb-1">
            <Button
              variant="ghost"
              size="sm"
              className="h-8 w-8 p-0"
              onClick={handleBack}
            >
              <ArrowLeft className="h-4 w-4" />
            </Button>
            <VideoCamera className="h-5 w-5 text-primary" weight="fill" />
            <h1 className="font-semibold text-lg">Session History</h1>
          </div>
          <div className="flex items-center justify-between text-sm text-muted-foreground mt-2">
            <span>
              {filteredSessions.length} session
              {filteredSessions.length !== 1 ? "s" : ""}
              {sessionRoverFilter && ` from ${sessionRoverFilter}`}
            </span>
            <span>{formatDuration(totalDuration)} total</span>
          </div>
        </div>

        {/* Filters */}
        <div className="px-4 py-2 border-b border-border">
          <Button
            variant="ghost"
            size="sm"
            className="w-full justify-start gap-2"
            onClick={() => setShowFilters(!showFilters)}
          >
            <FunnelSimple className="h-4 w-4" />
            Filters
            {sessionRoverFilter && (
              <Badge variant="secondary" className="ml-auto">
                1
              </Badge>
            )}
          </Button>

          {showFilters && (
            <div className="mt-2 space-y-2">
              <div className="text-xs text-muted-foreground mb-1">Rover</div>
              <div className="flex flex-wrap gap-1">
                <Button
                  variant={sessionRoverFilter === null ? "default" : "outline"}
                  size="sm"
                  className="h-7 text-xs"
                  onClick={() => setSessionRoverFilter(null)}
                >
                  All
                </Button>
                {roverIds.map((id) => (
                  <Button
                    key={id}
                    variant={sessionRoverFilter === id ? "default" : "outline"}
                    size="sm"
                    className="h-7 text-xs"
                    onClick={() => setSessionRoverFilter(id)}
                  >
                    {id}
                  </Button>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* Session list */}
        <div className="flex-1 overflow-y-auto">
          {loading ? (
            <EmptyState message="Loading sessions..." />
          ) : error ? (
            <ErrorState message={error} onRetry={refresh} />
          ) : filteredSessions.length === 0 ? (
            <EmptyState
              message={
                sessionRoverFilter
                  ? `No sessions found for ${sessionRoverFilter}`
                  : "No recorded sessions found"
              }
            />
          ) : (
            Object.entries(groupedSessions).map(([date, dateSessions]) => (
              <div key={date}>
                <div className="px-4 py-2 text-xs font-medium text-muted-foreground bg-muted/50 sticky top-0">
                  {date}
                </div>
                {dateSessions.map((session) => (
                  <div key={session.session_id}>
                    <SessionListItem
                      session={session}
                      isSelected={selectedSessionId === session.session_id}
                      onSelect={() => selectSession(session.session_id)}
                      onPlayback={() => handlePlayback(session)}
                    />
                    <Separator />
                  </div>
                ))}
              </div>
            ))
          )}
        </div>

        {/* Refresh button */}
        <div className="p-4 border-t border-border">
          <Button
            variant="outline"
            className="w-full gap-2"
            onClick={refresh}
            disabled={loading}
          >
            <ArrowsClockwise
              className={`h-4 w-4 ${loading ? "animate-spin" : ""}`}
            />
            Refresh
          </Button>
        </div>
      </div>

      {/* Main content area */}
      <div className="flex-1 flex items-center justify-center bg-muted/30">
        {selectedSessionId ? (
          <Card className="max-w-md">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Play className="h-5 w-5" weight="fill" />
                Session Playback
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <p className="text-sm text-muted-foreground">
                Click "Open in Rerun Viewer" to view this session's telemetry,
                camera feed, and LiDAR data in the Rerun web viewer.
              </p>
              <div className="p-3 bg-muted rounded-lg text-xs text-muted-foreground">
                <strong>Tip:</strong> The Rerun viewer provides timeline scrubbing,
                3D visualization, and detailed telemetry charts.
              </div>
              {(() => {
                const session = sessions.find(
                  (s) => s.session_id === selectedSessionId
                );
                if (!session) return null;
                return (
                  <Button
                    className="w-full gap-2"
                    onClick={() => handlePlayback(session)}
                  >
                    <Play className="h-4 w-4" weight="fill" />
                    Open in Rerun Viewer
                  </Button>
                );
              })()}
            </CardContent>
          </Card>
        ) : (
          <div className="text-center text-muted-foreground">
            <VideoCamera
              className="h-16 w-16 mx-auto mb-4 opacity-30"
              weight="thin"
            />
            <p className="text-lg font-medium mb-1">Select a Session</p>
            <p className="text-sm">
              Choose a session from the list to view its recorded data
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
