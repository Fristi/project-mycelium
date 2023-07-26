import { CapacitorConfig } from "@capacitor/cli";

const config: CapacitorConfig = {
  appId: "co.mycelium.app",
  appName: "mycelium-app",
  webDir: "dist",
  server: {
    androidScheme: "https",
  },
};

export default config;
