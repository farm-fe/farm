export function instrumentPluginHooks(plugins, pending) {
  let sequence = 0;
  for (const plugin of plugins) for (const [name, hook] of Object.entries(plugin)) {
    if (!hook || typeof hook !== 'object' || typeof hook.executor !== 'function') continue;
    const original = hook.executor;
    hook.executor = function (...args) {
      const id = ++sequence;
      pending.set(id, { plugin: plugin.name, hook: name, path: args[0]?.resolvedPath ?? args[0]?.moduleId ?? null });
      try {
        const result = original.apply(this, args);
        if (result && typeof result.then === 'function') return Promise.resolve(result).finally(() => pending.delete(id));
        pending.delete(id);
        return result;
      } catch (error) { pending.delete(id); throw error; }
    };
  }
}
