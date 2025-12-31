import { Map } from "lucide-react";

/**
 * Placeholder for the maps view.
 * This will be fully implemented when migrating components from the operator app.
 */
export function MapsView() {
  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-4xl mx-auto space-y-8">
        {/* Header */}
        <div>
          <h1 className="text-2xl font-bold">Maps</h1>
          <p className="text-muted-foreground">
            3D Gaussian splat maps
          </p>
        </div>

        {/* Placeholder */}
        <div className="bg-muted/50 border border-border p-8 text-center">
          <Map className="h-12 w-12 mx-auto mb-4 text-muted-foreground opacity-50" />
          <h3 className="font-medium mb-2">Maps View</h3>
          <p className="text-sm text-muted-foreground max-w-md mx-auto">
            This view will display a list of available maps with 3D splat viewer.
            Components will be migrated from depot/operator/.
          </p>
        </div>
      </div>
    </div>
  );
}
