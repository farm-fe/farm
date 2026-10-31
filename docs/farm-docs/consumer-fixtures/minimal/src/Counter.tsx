import {usePage} from '@farmfe/docs/client';
import {useState} from 'react';
export default function Counter() {
  const [count,setCount]=useState(0);
  return <section>
    <p data-testid="page-identity">{usePage().routePath}</p>
    <button type="button" onClick={()=>setCount(value=>value+1)}>Harvest {count}</button>
  </section>;
}
