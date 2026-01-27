import { useState } from "react";
import {
  MapContainer,
  TileLayer,
  Polygon,
  Polyline,
  Marker,
  Tooltip,
  useMapEvents,
} from "react-leaflet";
import L from "leaflet";
import type { Zone } from "@/lib/types";

// Leaflet default icon fix
import "leaflet/dist/leaflet.css";

const ZONE_COLORS = [
  "#3b82f6", // blue
  "#10b981", // emerald
  "#f59e0b", // amber
  "#ef4444", // red
  "#8b5cf6", // violet
  "#ec4899", // pink
];

interface ZoneEditorProps {
  zones: Zone[];
  selectedZoneId: string | null;
  onSelectZone: (id: string | null) => void;
  center?: [number, number];
  zoom?: number;
  /** If true, show drawing tools for creating new zones */
  drawingMode?: boolean;
  onDrawComplete?: (polygon: [number, number][]) => void;
  /** If true, show coverage path overlay for the selected zone */
  showCoveragePath?: boolean;
}

function ZonePolygon({
  zone,
  color,
  selected,
  onClick,
}: {
  zone: Zone;
  color: string;
  selected: boolean;
  onClick: () => void;
}) {
  const polygon = zone.polygonLatlng;
  if (!polygon || polygon.length < 3) return null;

  return (
    <Polygon
      positions={polygon}
      pathOptions={{
        color,
        fillColor: color,
        fillOpacity: selected ? 0.3 : 0.15,
        weight: selected ? 3 : 2,
        dashArray: selected ? undefined : "5 5",
      }}
      eventHandlers={{ click: onClick }}
    >
      <Tooltip permanent direction="center" className="zone-label">
        <span className="text-xs font-medium">{zone.name}</span>
      </Tooltip>
    </Polygon>
  );
}

function WaypointMarkers({
  zone,
  color,
  visible,
}: {
  zone: Zone;
  color: string;
  visible: boolean;
}) {
  if (!visible || !zone.waypoints?.length) return null;

  return (
    <>
      {zone.waypoints.map((wp, i) => {
        // Convert waypoints to approximate lat/lng if polygon exists
        // For now, show numbered markers at polygon vertices
        const polygon = zone.polygonLatlng;
        if (!polygon || i >= polygon.length) return null;

        const icon = L.divIcon({
          className: "waypoint-marker",
          html: `<div style="
            width: 20px; height: 20px; border-radius: 50%;
            background: ${color}; color: white; font-size: 10px;
            display: flex; align-items: center; justify-content: center;
            font-weight: bold; border: 2px solid white; box-shadow: 0 1px 3px rgba(0,0,0,0.3);
          ">${i + 1}</div>`,
          iconSize: [20, 20],
          iconAnchor: [10, 10],
        });

        return (
          <Marker key={i} position={polygon[i]} icon={icon}>
            <Tooltip>
              WP {i + 1}: ({wp.x.toFixed(1)}, {wp.y.toFixed(1)})
            </Tooltip>
          </Marker>
        );
      })}
    </>
  );
}

function DrawingHandler({
  active,
  onComplete,
}: {
  active: boolean;
  onComplete: (polygon: [number, number][]) => void;
}) {
  const [points, setPoints] = useState<[number, number][]>([]);

  useMapEvents({
    click(e) {
      if (!active) return;
      const newPoint: [number, number] = [e.latlng.lat, e.latlng.lng];
      setPoints((prev) => [...prev, newPoint]);
    },
    dblclick(e) {
      if (!active || points.length < 3) return;
      e.originalEvent.preventDefault();
      onComplete(points);
      setPoints([]);
    },
  });

  if (!active || points.length === 0) return null;

  return (
    <>
      <Polygon
        positions={points}
        pathOptions={{
          color: "#3b82f6",
          fillColor: "#3b82f6",
          fillOpacity: 0.2,
          weight: 2,
          dashArray: "5 5",
        }}
      />
      {points.map((p, i) => {
        const icon = L.divIcon({
          className: "draw-vertex",
          html: `<div style="
            width: 10px; height: 10px; border-radius: 50%;
            background: #3b82f6; border: 2px solid white;
            box-shadow: 0 1px 3px rgba(0,0,0,0.3);
          "></div>`,
          iconSize: [10, 10],
          iconAnchor: [5, 5],
        });
        return <Marker key={i} position={p} icon={icon} />;
      })}
    </>
  );
}

function CoveragePathOverlay({
  zone,
  visible,
}: {
  zone: Zone;
  visible: boolean;
}) {
  if (!visible || !zone.coverageWaypoints?.length || !zone.polygonLatlng?.length)
    return null;

  // Coverage waypoints are in local frame (x,y) matching the polygon vertices.
  // The polygon vertices in waypoints[] correspond to polygonLatlng[].
  // We need to map coverage waypoints from local to lat/lng.
  // Use an affine approximation from the polygon's local→latlng mapping.
  const wps = zone.waypoints;
  const latlng = zone.polygonLatlng;
  if (wps.length < 2 || latlng.length < 2) return null;

  // Build a simple affine transform from 2 point pairs
  const x0 = wps[0].x, y0 = wps[0].y;
  const x1 = wps[1].x, y1 = wps[1].y;
  const lat0 = latlng[0][0], lng0 = latlng[0][1];
  const lat1 = latlng[1][0], lng1 = latlng[1][1];

  // Use more points if available for better transform
  const dx = x1 - x0, dy = y1 - y0;
  const localDist = Math.sqrt(dx * dx + dy * dy);
  if (localDist < 1e-9) return null;

  const dlat = lat1 - lat0, dlng = lng1 - lng0;
  const geoDist = Math.sqrt(dlat * dlat + dlng * dlng);
  const scale = geoDist / localDist;

  // Rotation from local to geo
  const localAngle = Math.atan2(dy, dx);
  const geoAngle = Math.atan2(dlat, dlng);
  const rot = geoAngle - localAngle;

  const cosR = Math.cos(rot);
  const sinR = Math.sin(rot);

  const positions: [number, number][] = zone.coverageWaypoints.map((wp) => {
    const rx = wp.x - x0;
    const ry = wp.y - y0;
    const tx = (cosR * rx - sinR * ry) * scale;
    const ty = (sinR * rx + cosR * ry) * scale;
    return [lat0 + ty, lng0 + tx] as [number, number];
  });

  return (
    <Polyline
      positions={positions}
      pathOptions={{
        color: "#f59e0b",
        weight: 2,
        opacity: 0.8,
        dashArray: "4 4",
      }}
    />
  );
}

export function ZoneEditor({
  zones,
  selectedZoneId,
  onSelectZone,
  center = [37.7749, -122.4194],
  zoom = 18,
  drawingMode = false,
  onDrawComplete,
  showCoveragePath = false,
}: ZoneEditorProps) {
  return (
    <div className="relative h-full w-full">
      <MapContainer
        center={center}
        zoom={zoom}
        className="h-full w-full z-0"
        doubleClickZoom={!drawingMode}
      >
        <TileLayer
          attribution="&copy; OpenStreetMap"
          url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
        />

        {zones.map((zone, idx) => (
          <ZonePolygon
            key={zone.id}
            zone={zone}
            color={ZONE_COLORS[idx % ZONE_COLORS.length]}
            selected={zone.id === selectedZoneId}
            onClick={() => onSelectZone(zone.id)}
          />
        ))}

        {zones.map((zone, idx) => (
          <WaypointMarkers
            key={`wp-${zone.id}`}
            zone={zone}
            color={ZONE_COLORS[idx % ZONE_COLORS.length]}
            visible={zone.id === selectedZoneId}
          />
        ))}

        {/* Coverage path overlay for selected zone */}
        {zones.map((zone) => (
          <CoveragePathOverlay
            key={`cov-${zone.id}`}
            zone={zone}
            visible={showCoveragePath && zone.id === selectedZoneId}
          />
        ))}

        {drawingMode && onDrawComplete && (
          <DrawingHandler active={drawingMode} onComplete={onDrawComplete} />
        )}
      </MapContainer>

      {drawingMode && (
        <div className="absolute bottom-4 left-4 z-[1000] bg-background/90 backdrop-blur border border-border rounded-lg px-3 py-2 text-xs text-muted-foreground">
          Click to add vertices. Double-click to finish polygon.
        </div>
      )}
    </div>
  );
}
