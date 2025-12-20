import { Toast } from "@/components/ui/Toast";
import { HomeScreen } from "@/components/screens/HomeScreen";
import { TeleopScreen } from "@/components/screens/TeleopScreen";
import { useOperatorStore } from "@/store";
import { View } from "@/lib/types";

function App() {
  const { currentView } = useOperatorStore();

  return (
    <div className="dark">
      {currentView === View.Home ? <HomeScreen /> : <TeleopScreen />}

      {/* Toast notifications (global) */}
      <Toast />
    </div>
  );
}

export default App;
