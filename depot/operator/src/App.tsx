import { Toast } from "@/components/ui/Toast";
import { HomeScreen } from "@/components/screens/HomeScreen";
import { TeleopScreen } from "@/components/screens/TeleopScreen";
import { MapsScreen } from "@/components/screens/MapsScreen";
import { useOperatorStore } from "@/store";
import { View } from "@/lib/types";

function App() {
  const { currentView } = useOperatorStore();

  const renderScreen = () => {
    switch (currentView) {
      case View.Home:
        return <HomeScreen />;
      case View.Teleop:
        return <TeleopScreen />;
      case View.Maps:
        return <MapsScreen />;
      default:
        return <HomeScreen />;
    }
  };

  return (
    <div className="dark">
      {renderScreen()}

      {/* Toast notifications (global) */}
      <Toast />
    </div>
  );
}

export default App;
