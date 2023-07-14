import { App as CapApp } from '@capacitor/app';
import { Browser } from '@capacitor/browser';
import { useEffect, useState } from 'react';
import { useAuth0 } from '@auth0/auth0-react';
import { useAuth } from './AuthContext';
import axios from 'axios';


const Test = () => {
 const { token } = useAuth();
 const [stations, setStations] = useState([]);

 useEffect(() => {
  axios.get("http://localhost:8080/api/stations", { headers: { "Authorization" : `Bearer ${token}`}} )
  .then(resp => setStations(resp.data));
 }, [setStations])
 

 return (<p>{JSON.stringify(stations)}</p>);
}

const App: React.FC = () => {
  const { handleRedirectCallback } = useAuth0();

  useEffect(() => {
    // Handle the 'appUrlOpen' event and call `handleRedirectCallback`
    CapApp.addListener('appUrlOpen', async ({ url }) => {
      if (url.includes('state') && (url.includes('code') || url.includes('error'))) {
        await handleRedirectCallback(url);
      }
      // No-op on Android
      await Browser.close();
    });
  }, [handleRedirectCallback]);

  return (
    <div>
      <Test />
    </div>
  );
};

export default App
