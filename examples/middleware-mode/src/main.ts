const app = document.querySelector('#app');

if (app) {
  app.innerHTML = `
    <p>123 Farm middlewares are mounted on an external Express app.</p>
    <p>Open <code>/api/ping</code> to verify custom routes still work.</p>
    <p>Open <code>/api/runner</code> to verify ModuleRunner SSR import.</p>
    <p>Edit <code>src/main.ts</code> and check HMR update.</p>
    <button id="runner-check">Check runner route</button>
    <pre id="runner-result"></pre>
  `;

  const button = document.querySelector<HTMLButtonElement>('#runner-check');
  const result = document.querySelector<HTMLElement>('#runner-result');

  button?.addEventListener('click', async () => {
    const response = await fetch('/api/runner');
    const payload = await response.json();
    if (result) {
      result.textContent = JSON.stringify(payload, null, 2);
    }
  });
}
