import { useOperatorStore } from "@/store";
import { View, type MapSummary, type MapManifest } from "@/lib/types";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import {
  ArrowLeft,
  MapTrifold,
  Cube,
  Calendar,
  Database,
  Robot,
  Download,
  ArrowsClockwise,
} from "@phosphor-icons/react";
import { useState } from "react";
import { useMaps, useMapDetails, getMapAssetUrl } from "@/hooks/useMaps";
import { SplatViewer } from "@/components/scene/SplatViewer";

function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

function formatNumber(n: number): string {
  if (n >= 1_000_000) {
    return `${(n / 1_000_000).toFixed(1)}M`;
  }
  if (n >= 1_000) {
    return `${(n / 1_000).toFixed(1)}K`;
  }
  return n.toString();
}

function MapListItem({
  map,
  isSelected,
  onSelect,
}: {
  map: MapSummary;
  isSelected: boolean;
  onSelect: () => void;
}) {
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
        <div className="flex items-center gap-2">
          <MapTrifold
            className={`h-5 w-5 ${
              map.hasSplat ? "text-primary" : "text-muted-foreground"
            }`}
            weight="fill"
          />
          <div>
            <div className="font-medium text-sm">{map.name}</div>
            <div className="text-xs text-muted-foreground">
              v{map.version} · {formatDate(map.updatedAt)}
            </div>
          </div>
        </div>
        {map.hasSplat && (
          <Badge variant="default" className="text-xs">
            <Cube className="h-3 w-3 mr-1" weight="fill" />
            3D
          </Badge>
        )}
      </div>

      <div className="mt-2 flex items-center gap-3 text-xs text-muted-foreground">
        <span className="flex items-center gap-1">
          <Robot className="h-3 w-3" weight="fill" />
          {map.sessionCount} sessions
        </span>
      </div>
    </div>
  );
}

function MapDetails({ manifest }: { manifest: MapManifest }) {
  const [showViewer, setShowViewer] = useState(false);

  return (
    <div className="p-6 space-y-6">
      {/* 3D Viewer (when enabled) */}
      {showViewer && manifest.assets.splat && (
        <div className="relative">
          <SplatViewer mapId={manifest.id} className="h-96 rounded-lg overflow-hidden" />
          <Button
            variant="outline"
            size="sm"
            className="absolute top-2 right-2"
            onClick={() => setShowViewer(false)}
          >
            Close Viewer
          </Button>
        </div>
      )}

      {/* Header */}
      <div className="flex items-start justify-between">
        <div>
          <h2 className="text-2xl font-bold">{manifest.name}</h2>
          {manifest.description && (
            <p className="text-muted-foreground mt-1">{manifest.description}</p>
          )}
        </div>
        {manifest.assets.splat && !showViewer && (
          <Button onClick={() => setShowViewer(true)} className="gap-2">
            <Cube className="h-4 w-4" weight="fill" />
            View 3D
          </Button>
        )}
      </div>

      {/* Stats */}
      <div className="grid grid-cols-3 gap-4">
        <Card>
          <CardContent className="pt-4">
            <div className="text-2xl font-bold">
              {formatNumber(manifest.stats.totalPoints)}
            </div>
            <div className="text-xs text-muted-foreground">Total Points</div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-4">
            <div className="text-2xl font-bold">{manifest.sessions.length}</div>
            <div className="text-xs text-muted-foreground">Sessions</div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-4">
            <div className="text-2xl font-bold">v{manifest.version}</div>
            <div className="text-xs text-muted-foreground">Version</div>
          </CardContent>
        </Card>
      </div>

      {/* GPS Bounds */}
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm">Coverage Area</CardTitle>
        </CardHeader>
        <CardContent className="text-sm font-mono">
          <div className="grid grid-cols-2 gap-2">
            <div>
              <span className="text-muted-foreground">Min: </span>
              {manifest.bounds.minLat.toFixed(6)}, {manifest.bounds.minLon.toFixed(6)}
            </div>
            <div>
              <span className="text-muted-foreground">Max: </span>
              {manifest.bounds.maxLat.toFixed(6)}, {manifest.bounds.maxLon.toFixed(6)}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Downloads */}
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm">Assets</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          {manifest.assets.splat && (
            <Button variant="outline" size="sm" className="w-full justify-start" asChild>
              <a href={getMapAssetUrl(manifest.id, "splat.ply")} download>
                <Download className="h-4 w-4 mr-2" />
                Download Gaussian Splat (.ply)
              </a>
            </Button>
          )}
          {manifest.assets.pointcloud && (
            <Button variant="outline" size="sm" className="w-full justify-start" asChild>
              <a href={getMapAssetUrl(manifest.id, "pointcloud.laz")} download>
                <Download className="h-4 w-4 mr-2" />
                Download Point Cloud (.laz)
              </a>
            </Button>
          )}
          {manifest.assets.mesh && (
            <Button variant="outline" size="sm" className="w-full justify-start" asChild>
              <a href={getMapAssetUrl(manifest.id, "mesh.glb")} download>
                <Download className="h-4 w-4 mr-2" />
                Download Mesh (.glb)
              </a>
            </Button>
          )}
          {!manifest.assets.splat && !manifest.assets.pointcloud && !manifest.assets.mesh && (
            <div className="text-sm text-muted-foreground py-2">
              No assets available yet. Map is still processing.
            </div>
          )}
        </CardContent>
      </Card>

      {/* Sessions */}
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm">Source Sessions</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            {manifest.sessions.map((session) => (
              <div
                key={session.sessionId}
                className="flex items-center justify-between text-sm py-1"
              >
                <div className="flex items-center gap-2">
                  <Robot className="h-4 w-4 text-muted-foreground" weight="fill" />
                  <span>{session.roverId}</span>
                </div>
                <span className="text-muted-foreground">
                  {formatDate(session.date)}
                </span>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Timestamps */}
      <div className="text-xs text-muted-foreground space-y-1">
        <div className="flex items-center gap-2">
          <Calendar className="h-3 w-3" />
          Created: {formatDate(manifest.createdAt)}
        </div>
        <div className="flex items-center gap-2">
          <ArrowsClockwise className="h-3 w-3" />
          Updated: {formatDate(manifest.updatedAt)}
        </div>
      </div>
    </div>
  );
}

export function MapsScreen() {
  const { setView } = useOperatorStore();
  const { maps, loading, error, refresh } = useMaps();
  const [selectedMapId, setSelectedMapId] = useState<string | null>(null);
  const { manifest, loading: manifestLoading } = useMapDetails(selectedMapId);

  return (
    <div className="h-screen w-screen overflow-hidden bg-background flex dark">
      {/* Map list panel (left side) */}
      <div className="w-80 h-full flex flex-col border-r border-border bg-card">
        <div className="p-4 border-b border-border">
          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8"
              onClick={() => setView(View.Home)}
            >
              <ArrowLeft className="h-4 w-4" />
            </Button>
            <div>
              <div className="flex items-center gap-2">
                <MapTrifold className="h-5 w-5 text-primary" weight="fill" />
                <h1 className="font-semibold text-lg">Maps</h1>
              </div>
              <div className="text-sm text-muted-foreground">
                {maps.length} maps available
              </div>
            </div>
          </div>
        </div>

        <div className="p-2 border-b border-border">
          <Button
            variant="outline"
            size="sm"
            className="w-full"
            onClick={refresh}
            disabled={loading}
          >
            <ArrowsClockwise
              className={`h-4 w-4 mr-2 ${loading ? "animate-spin" : ""}`}
            />
            Refresh
          </Button>
        </div>

        <div className="flex-1 overflow-y-auto">
          {loading && maps.length === 0 ? (
            <div className="p-6 text-center text-muted-foreground">
              <ArrowsClockwise className="h-8 w-8 mx-auto mb-2 animate-spin" />
              <p className="text-sm">Loading maps...</p>
            </div>
          ) : error ? (
            <div className="p-6 text-center text-muted-foreground">
              <Database className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p className="text-sm text-destructive">{error}</p>
              <Button variant="link" size="sm" onClick={refresh}>
                Retry
              </Button>
            </div>
          ) : maps.length === 0 ? (
            <div className="p-6 text-center text-muted-foreground">
              <MapTrifold className="h-12 w-12 mx-auto mb-3 opacity-50" weight="thin" />
              <p className="text-sm">No maps yet</p>
              <p className="text-xs mt-1">
                Maps are created from rover sessions
              </p>
            </div>
          ) : (
            maps.map((map) => (
              <div key={map.id}>
                <MapListItem
                  map={map}
                  isSelected={selectedMapId === map.id}
                  onSelect={() => setSelectedMapId(map.id)}
                />
                <Separator />
              </div>
            ))
          )}
        </div>

        <div className="p-4 border-t border-border">
          <div className="text-xs text-muted-foreground text-center">
            Maps are generated from rover sessions
          </div>
        </div>
      </div>

      {/* Map details (main content) */}
      <div className="flex-1 overflow-y-auto">
        {selectedMapId ? (
          manifestLoading ? (
            <div className="h-full flex items-center justify-center">
              <ArrowsClockwise className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
          ) : manifest ? (
            <MapDetails manifest={manifest} />
          ) : (
            <div className="h-full flex items-center justify-center text-muted-foreground">
              Failed to load map details
            </div>
          )
        ) : (
          <div className="h-full flex flex-col items-center justify-center text-muted-foreground">
            <MapTrifold className="h-16 w-16 mb-4 opacity-30" weight="thin" />
            <p className="text-lg">Select a map to view details</p>
            <p className="text-sm mt-1">
              Download splats, point clouds, and see session history
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
