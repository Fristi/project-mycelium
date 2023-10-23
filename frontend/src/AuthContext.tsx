import { useAuth0, User } from "@auth0/auth0-react";
import React, { useEffect, createContext, useContext } from "react";
import { Browser } from '@capacitor/browser';


type Props = {
  children: React.ReactNode;
};

type ContextType = {
  token?: string;
  user?: User;
};

const initialContext: ContextType = { token: undefined };

const AuthenticationContext = createContext<ContextType>(initialContext);

export const AuthContext: React.FC<Props> = ({ children }) => {
  const { user, isAuthenticated, isLoading, loginWithRedirect, getAccessTokenSilently } = useAuth0();
  const [token, setToken] = React.useState<string | null>(null);

  console.log(`Initializing auth context ${isAuthenticated}`, user);

  const login = async () => {
    await loginWithRedirect({
      async openUrl(url) {
         // Redirect using Capacitor's Browser plugin
        await Browser.open({
          url,
          windowName: "_self"
        });
      }
    });
  };

  useEffect(() => {
    (async () => {
      try {
        if (isAuthenticated) {
          const token = await getAccessTokenSilently();
          setToken(token);
        }
      } catch (e) {
        console.error(e);
      }
    })();
  }, [getAccessTokenSilently, isAuthenticated]);

  if (!isAuthenticated || token == null) {
    return (
      <p>
        You should{" "}
        <a href="#" onClick={login}>
          login
        </a>
        !
      </p>
    );
  }

  if (isLoading) {
    return <p>Loading</p>;
  }

  return <AuthenticationContext.Provider value={{ token, user }}>{children}</AuthenticationContext.Provider>;
};

export const useAuth = () => useContext(AuthenticationContext);
