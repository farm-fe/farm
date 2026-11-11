// Page-local channels connect isolated Farm runtimes; MF still owns loading/sharing.
function createFederationHot(configuration) {
  const symbol = Symbol.for("farm.module-federation.hot.v1");
  const channels = (globalThis[symbol] ||= new Map());
  const consumed = new Map();
  const accepted = new Map();
  const channel = (key) => {
    if (!channels.has(key))
      channels.set(key, { current: undefined, listeners: new Set() });
    return channels.get(key);
  };
  const status = (state, reason) =>
    window.dispatchEvent(
      new CustomEvent("farm:mf:hot-status", {
        detail: { name: configuration.name, state, reason },
      }),
    );
  const reject = (reason) => {
    status("reload-required", reason);
    throw new Error(reason);
  };
  const notify = async (entry, value) => {
    entry.current = value;
    for (const listener of entry.listeners) await listener(value);
  };
  return {
    consume(id, value) {
      if (consumed.has(id)) return consumed.get(id).facade;
      const remote = configuration.remotes.find((r) =>
        id.startsWith(r.alias + "/"),
      );
      if (!remote) return value;
      const entry = channel(
        remote.name + "/./" + id.slice(remote.alias.length + 1),
      );
      const record = { current: entry.current ?? value };
      // A stable namespace keeps cached static/dynamic proxy modules current.
      record.facade = new Proxy(Object.create(null), {
        get: (_, key) => record.current[key],
        has: (_, key) => key in record.current,
        ownKeys: () => Reflect.ownKeys(record.current),
        getOwnPropertyDescriptor: (_, key) =>
          key in record.current
            ? {
                enumerable: true,
                configurable: true,
                value: record.current[key],
              }
            : undefined,
      });
      consumed.set(id, record);
      entry.current ||= value;
      entry.listeners.add(async (next) => {
        const callbacks = accepted.get(id);
        if (!callbacks?.size)
          reject(
            `Remote ${id} has no accepted consumer boundary; reload required`,
          );
        record.current = next;
        for (const callback of callbacks) await callback(record.facade);
      });
      return record.facade;
    },
    accept(id, callback) {
      if (typeof callback !== "function")
        throw new TypeError("acceptRemote requires a callback");
      const callbacks = accepted.get(id) || new Set();
      accepted.set(id, callbacks);
      callbacks.add(callback);
      return () => callbacks.delete(callback);
    },
    async publish(key, value) {
      const entry = channel(configuration.name + "/" + key);
      if (!value) throw new Error(`Empty hot exports for ${key}`);
      if (
        entry.current &&
        Object.keys(entry.current).sort().join() !==
          Object.keys(value).sort().join()
      )
        reject(`Export shape changed for ${key}; reload required`);
      if (!entry.listeners.size)
        reject(`No Federation HMR consumer for ${key}; reload required`);
      await notify(entry, value);
    },
    install(context) {
      // Capture channels before the first update too, not only after publish.
      context.registerUpdateParticipant(() => {
        const entries = [...channels.entries()]
          .filter(([key]) => key.startsWith(configuration.name + "/"))
          .map(([, entry]) => entry);
        const snapshots = entries.map((entry) => [entry, entry.current]);
        return {
          rollback() {
            for (const [entry, value] of snapshots) entry.current = value;
          },
          async recover() {
            for (const [entry, value] of snapshots)
              if (value) await notify(entry, value);
            status("rolled-back", "Update rejected; previous exports restored");
          },
          commit() {
            status("applied");
          },
        };
      });
      context.on("farm:error", () =>
        status(
          "compile-error",
          "Remote compilation failed; previous application remains active",
        ),
      );
      context.on("farm:ws:disconnect", () =>
        status(
          "reload-required",
          "Remote connection lost; Farm reconnect recovery reloads the document",
        ),
      );
    },
  };
}
