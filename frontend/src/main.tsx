import React from "react"
import ReactDOM from "react-dom/client"
import { Auth0Provider } from '@auth0/auth0-react';
import App from "./App.tsx"
import "./styles/global.css"
import { Capacitor } from '@capacitor/core';
import { AuthContext } from "./AuthContext.tsx";

const platform = Capacitor.getPlatform()
const iosOrAndroid = platform === 'ios' || platform === 'android';

const callbackUri = iosOrAndroid
  ? 'co.mycelium.app://dev-plq6-asi.eu.auth0.com/capacitor/co.mycelium.app/callback'
  : 'http://localhost:8080';

const cacheLocation = iosOrAndroid
  ? "memory"
  : "localstorage"


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
          scope: "read:current_user update:current_user_metadata"
        }}
      >
      <AuthContext>
        <App />
      </AuthContext>
    </Auth0Provider>
  </React.StrictMode>
)
