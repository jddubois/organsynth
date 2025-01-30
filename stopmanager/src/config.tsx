import { useEffect, useState } from "react";

export const useConfig: (url?: string) => {
    config: any;
    loading: boolean;
    error: any;
} = (url = 'http://192.168.1.21:8080/config') => {
    const [config, setConfig] = useState(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    useEffect(() => {
      const fetchConfig = async () => {
        setLoading(true);
        setError(null);
        try {
          const response = await fetch(url);
          if (!response.ok) {
            throw new Error(`Failed to fetch config: ${response.statusText}`);
          }
          const data = await response.json();
          setConfig(data);
        } catch (err: any) {
          setError(err.message);
        } finally {
          setLoading(false);
        }
      };
      fetchConfig();
    }, [url]);
    return { config, loading, error };
  };