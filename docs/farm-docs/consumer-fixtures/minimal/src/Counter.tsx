import {useState} from 'react';
import {usePage} from '@farmfe/docs/client';
export default function Counter() {
  const [count,setCount]=useState(0);
  return <section>
    <p data-testid="page-identity">{usePage().routePath}</p>
    <button type="button" onClick={()=>setCount(value=>value+1)}>Harvest {count}</button>
  </section>;
}
