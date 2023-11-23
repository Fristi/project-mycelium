import { App as CapApp } from "@capacitor/app";
import { Browser } from "@capacitor/browser";
import { useEffect } from "react";
import { useAuth0 } from "@auth0/auth0-react";
import { QueryClient, QueryClientProvider } from "react-query";
import Shell from "./Shell";
import { PlantView } from "./pages/PlantView.tsx";
import { PlantList } from "./pages/PlantList.tsx";
import { PlantEdit } from "./pages/PlantEdit.tsx";
import { PlantAdd, PlantProvisioning } from "./pages/PlantAdd.tsx";
import { createHashRouter, createRoutesFromElements, Route, RouterProvider } from "react-router-dom";


const router = createHashRouter(
  createRoutesFromElements(
    <Route
      path="/"
      element={<Shell />}
      // errorElement={<ErrorPage />}
    >
      <Route>
        <Route index element={<PlantList />} />
        <Route path="plant-add" element={<PlantAdd />} />
        <Route path="plant-add/:deviceId" element={<PlantProvisioning />} />
        <Route path="plants/:plantId/edit" element={<PlantEdit />} />
        <Route path="plants/:plantId" element={<PlantView />} />
      </Route>
    </Route>,
  ),
);

const App: React.FC = () => {
  const { handleRedirectCallback } = useAuth0();
  const queryClient = new QueryClient();

  useEffect(() => {
    // Handle the 'appUrlOpen' event and call `handleRedirectCallback`
    CapApp.addListener("appUrlOpen", async ({ url }) => {
      console.log(`received url ${url}`);
      if (url.includes("state") && (url.includes("code") || url.includes("error"))) {
        console.log(`Handling callback url`);
        await handleRedirectCallback(url);
      }
      // No-op on Android
      await Browser.close();
    });
  }, [handleRedirectCallback]);

  return (
    <div>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </div>
  );
};

export default App;
