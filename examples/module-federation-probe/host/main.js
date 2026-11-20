const hostSystemBeforeRemotes = globalThis.farm_mf_host.m;
const [a, b] = await Promise.all([import('remote/Button'), import('other/Button')]);
a.mount(document.querySelector('#app'));
b.mount(document.querySelector('#app'));
window.probe = {
  ready:true,
  hostUnchanged:hostSystemBeforeRemotes === globalThis.farm_mf_host.m,
  repeat: async () => (await import('remote/Button')) === a,
  missingExpose: () => import('remote/Unknown'),
  missingEntry: () => import('missing/Button')
};
