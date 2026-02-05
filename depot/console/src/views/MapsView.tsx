import { useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { MapTrifold, ArrowLeft, Planet, Cube, Clock } from "@phosphor-icons/react";
import { Scene } from "@/components/scene/Scene";
import { useMap3D } from "@/hooks/useMap3D";

export function MapsView() {
  const { mapId } = useParams<{ mapId: string }>();
  const navigate = useNavigate();
  const { maps, selectedMapId, loading, error, selectMap, refresh } = useMap3D();

  // Auto-select map when navigating to /maps/:mapId
  useEffect(() => {
    if (mapId && mapId !== selectedMapId) {
      selectMap(mapId);
    }
  }, [mapId, selectedMapId, selectMap]);

  // Map viewer mode
  if (mapId) {
    return (
      <div className="relative w-full h-full">
        <Scene mode="dispatch" />
        <button
          onClick={() => navigate("/maps")}
          className="absolute top-4 left-4 z-10 flex items-center gap-2 px-3 py-1.5 bg-card/90 backdrop-blur-sm border border-border rounded-lg text-sm hover:bg-accent transition-colors"
        >
          <ArrowLeft className="h-4 w-4" />
          Maps
        </button>
      </div>
    );
  }

  // Map list mode
  return (
    <div className="min-h-full p-6">
      <div className="max-w-4xl mx-auto space-y-8">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">Maps</h1>
            <p className="text-muted-foreground">3D Gaussian splat maps</p>
          </div>
          <button
            onClick={refresh}
            disabled={loading}
            className="px-3 py-1.5 text-sm border border-border rounded-lg hover:bg-accent transition-colors disabled:opacity-50"
          >
            {loading ? "Loading..." : "Refresh"}
          </button>
        </div>

        {/* Error */}
        {error && (
          <div className="bg-destructive/10 border border-destructive/20 rounded-lg p-4 text-sm text-destructive">
            {error}
          </div>
        )}

        {/* Map cards */}
        {maps.length > 0 ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {maps.map((map) => (
              <button
                key={map.id}
                onClick={() => navigate(`/maps/${map.id}`)}
                className="text-left bg-card border border-border rounded-lg overflow-hidden hover:border-primary/50 hover:shadow-md transition-all"
              >
                {/* Thumbnail or placeholder */}
                <div className="aspect-video bg-muted flex items-center justify-center">
                  {map.thumbnailUrl ? (
                    <img
                      src={map.thumbnailUrl}
                      alt={map.name}
                      className="w-full h-full object-cover"
                    />
                  ) : (
                    <MapTrifold className="h-10 w-10 text-muted-foreground opacity-40" />
                  )}
                </div>
                <div className="p-3 space-y-1.5">
                  <h3 className="font-medium truncate">{map.name}</h3>
                  <div className="flex items-center gap-3 text-xs text-muted-foreground">
                    <span className="flex items-center gap-1">
                      <Cube className="h-3 w-3" />
                      {map.sessionCount} sessions
                    </span>
                    <span className="flex items-center gap-1">
                      <Planet className="h-3 w-3" />
                      v{map.version}
                    </span>
                  </div>
                  <div className="flex items-center gap-1 text-xs text-muted-foreground">
                    <Clock className="h-3 w-3" />
                    {new Date(map.updatedAt).toLocaleDateString()}
                  </div>
                </div>
              </button>
            ))}
          </div>
        ) : !loading ? (
          <div className="bg-muted/50 border border-border p-8 text-center">
            <MapTrifold className="h-12 w-12 mx-auto mb-4 text-muted-foreground opacity-50" />
            <h3 className="font-medium mb-2">No maps yet</h3>
            <p className="text-sm text-muted-foreground max-w-md mx-auto">
              Maps are created by the mapper service from recorded sessions.
              Start a mapping session with SLAM enabled to build your first map.
            </p>
          </div>
        ) : null}
      </div>
    </div>
  );
}
