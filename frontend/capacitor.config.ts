import { CapacitorConfig } from '@capacitor/cli';

const config: CapacitorConfig = {
  appId: 'com.example.app',
  appName: 'capacitorjs-react-tailwind-template',
  webDir: 'dist',
  server: {
    androidScheme: 'https'
  }
};

export default config;
