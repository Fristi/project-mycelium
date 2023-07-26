import { App as CapApp } from "@capacitor/app";
import { Browser } from "@capacitor/browser";
import { useEffect, useState } from "react";
import { useAuth0 } from "@auth0/auth0-react";
import { QueryClient, QueryClientProvider } from "react-query";
import Shell from "./Shell";

const App: React.FC = () => {
  const { handleRedirectCallback } = useAuth0();
  const queryClient = new QueryClient();

  useEffect(() => {
    // Handle the 'appUrlOpen' event and call `handleRedirectCallback`
    CapApp.addListener("appUrlOpen", async ({ url }) => {
      if (
        url.includes("state") &&
        (url.includes("code") || url.includes("error"))
      ) {
        await handleRedirectCallback(url);
      }
      // No-op on Android
      await Browser.close();
    });
  }, [handleRedirectCallback]);

  return (
    <div>
      <QueryClientProvider client={queryClient}>
        <Shell />
      </QueryClientProvider>
    </div>
  );
};

export default App;
