import React, { createContext, useContext, useState, useEffect } from 'react';
import { createFixtures } from '../../packages/domain/index.mjs';

type State = ReturnType<typeof createFixtures>;
type Context = {
  state: State;
  update: (next: State) => void;
  tenant: string;
  setTenant: (value: string) => void;
  role: string;
  setRole: (value: string) => void;
  reset: () => void;
};
const Store = createContext<Context | null>(null);
const key = 'farm-commerce-lab-v1';

function initial(): State {
  try {
    const stored = JSON.parse(localStorage.getItem(key) ?? 'null');
    if (
      stored?.schema === 1 &&
      Array.isArray(stored.products) &&
      Array.isArray(stored.orders) &&
      Array.isArray(stored.tenants) &&
      Array.isArray(stored.campaigns)
    )
      return stored;
  } catch {
    /* Restore deterministic fixtures after invalid local state. */
  }
  return createFixtures();
}

export function Provider({ children }: { children: React.ReactNode }) {
  const [state, update] = useState(initial);
  const [tenant, setTenant] = useState('north');
  const [role, setRole] = useState('admin');
  useEffect(() => {
    localStorage.setItem(key, JSON.stringify(state));
  }, [state]);
  return (
    <Store.Provider
      value={{
        state,
        update,
        tenant,
        setTenant,
        role,
        setRole,
        reset: () => update(createFixtures()),
      }}
    >
      {children}
    </Store.Provider>
  );
}

export function useStore() {
  const store = useContext(Store);
  if (!store) throw new Error('Commerce provider is missing');
  return store;
}
