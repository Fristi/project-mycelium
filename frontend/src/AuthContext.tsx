import { useAuth0, User } from '@auth0/auth0-react';
import React, { useEffect, createContext, useContext } from 'react';

type Props = {
    children: React.ReactNode
}

type ContextType = {
    token?: string
    user?: User
}

const initialContext: ContextType = { token: undefined }

const Context = createContext<ContextType>(initialContext);
  
export const AuthContext: React.FC<Props> = ({ children }) =>  {
    const { user, isAuthenticated, isLoading, loginWithRedirect, getAccessTokenSilently } = useAuth0();
    const [token, setToken] = React.useState<string | null>(null);

    useEffect(() => {
        (async () => {
          try {
            if(isAuthenticated) {
                const token = await getAccessTokenSilently();
                setToken(token);
            }
          } catch (e) {
            console.error(e);
          }
        })();
      }, [getAccessTokenSilently, isAuthenticated]);

    if(!isAuthenticated) {
        return (<p>You should <a href="#" onClick={() => loginWithRedirect()}>login</a>!</p>);
    }

    if(isLoading || token == null) {
        return (<p>Loading</p>)
    }

    return <Context.Provider value={{token, user}}>{children}</Context.Provider> 
}

export const useAuth = () => useContext(Context);