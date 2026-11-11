import './button.css';
let count = 0;
export function mount(parent) {
  const button = document.createElement('button');
  button.className = 'remote-b'; button.id = 'button-b'; button.textContent = 'Remote b';
  button.onclick = async () => {
    try { const {label} = await import('./lazy.js'); button.textContent = label + ':' + ++count; }
    catch(error) { window.lastLazyError = error.message; }
  };
  parent.append(button);
}
