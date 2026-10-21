import {usePage} from '@farmfe/docs/client';
import {useState} from 'react';
export default function Marker() {
  const [count,setCount]=useState(0);
  return <section className="orchard-marker font-bold" data-testid="orchard-marker">
    <p data-testid="original-page">{usePage().routePath}</p>
    <button type="button" onClick={()=>setCount(value=>value+1)}>Harvest {count}</button>
  </section>;
}
