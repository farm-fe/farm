import { useEffect, useState } from 'react';

export function useQueryParams() {
  const [queryParams, setQueryParams] = useState(
    () => new URLSearchParams(window.location.search)
  );

  useEffect(() => {
    const handleUrlChange = () => {
      setQueryParams(new URLSearchParams(window.location.search));
    };

    window.addEventListener('popstate', handleUrlChange);
    window.addEventListener('pushstate', handleUrlChange);
    window.addEventListener('replacestate', handleUrlChange);

    return () => {
      window.removeEventListener('popstate', handleUrlChange);
      window.removeEventListener('pushstate', handleUrlChange);
      window.removeEventListener('replacestate', handleUrlChange);
    };
  }, []);

  return queryParams;
}
