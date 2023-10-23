import React from "react";
import ReactDOM from "react-dom/client";
import { Auth0Provider } from "@auth0/auth0-react";
import App from "./App.tsx";
import "./styles/global.css";
import { Capacitor } from "@capacitor/core";

const platform = Capacitor.getPlatform();
const iosOrAndroid = platform === "ios" || platform === "android";
const host = import.meta.env.MODE == "production" ? "https://mycelium.fly.dev" : "http://localhost:8080";
const callbackUri = iosOrAndroid ? "co.mycelium.app://dev-plq6-asi.eu.auth0.com/capacitor/co.mycelium.app/callback" : host;

const cacheLocation = iosOrAndroid ? "memory" : "localstorage";

console.log("Bootstrapping app");

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Auth0Provider
      domain="dev-plq6-asi.eu.auth0.com"
      clientId="G2SXj6FDnbe6YUOPVfXcszLu5jxTl2nj"
      // useRefreshTokens={true}
      // useRefreshTokensFallback={false}
      cacheLocation={cacheLocation}
      authorizationParams={{
        redirect_uri: callbackUri,
        audience: "https://dev-plq6-asi.eu.auth0.com/api/v2/",
        scope: "read:current_user update:current_user_metadata",
      }}
    >
      <App />
    </Auth0Provider>
  </React.StrictMode>,
);
