import { useEffect } from 'react';
import { apis } from '../../shared';

export function Inspect() {
  // temporary resolution just for check ui render.
  useEffect(() => {
    apis.getStats().then((res) => {
      console.log(res);
    });
  }, []);
  return <div>Inspect</div>;
}
