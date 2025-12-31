import { Video } from "lucide-react";

/**
 * Placeholder for the sessions view.
 * This will be fully implemented when migrating components from the operator app.
 */
export function SessionsView() {
  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-4xl mx-auto space-y-8">
        {/* Header */}
        <div>
          <h1 className="text-2xl font-bold">Sessions</h1>
          <p className="text-muted-foreground">
            Recorded telemetry and sensor data
          </p>
        </div>

        {/* Placeholder */}
        <div className="bg-muted/50 border border-border p-8 text-center">
          <Video className="h-12 w-12 mx-auto mb-4 text-muted-foreground opacity-50" />
          <h3 className="font-medium mb-2">Sessions View</h3>
          <p className="text-sm text-muted-foreground max-w-md mx-auto">
            This view will display recorded sessions with filtering, playback,
            and metadata. Components will be migrated from depot/operator/.
          </p>
        </div>
      </div>
    </div>
  );
}
