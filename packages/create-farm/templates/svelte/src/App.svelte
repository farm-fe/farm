<script lang="ts">
  import svelteLogo from './assets/svelte.svg'
  import farmLogo from './assets/logo.png'
  import Counter from './lib/Counter.svelte'
</script>

<main>
  <div>
    <a href="https://farmfe.org" target="_blank" rel="noreferrer">
      <img src={farmLogo} class="logo" alt="Farm Logo" />
    </a>
    <a href="https://svelte.dev" target="_blank" rel="noreferrer">
      <img src={svelteLogo} class="logo svelte" alt="Svelte Logo" />
    </a>
  </div>
  <h1>Farm + Svelte</h1>

  <div class="card">
    <Counter />
  </div>

  <p>
    Powered by <a href="https://farmfe.org">Farm</a> and svelte vite plugin
  </p>

  <p class="read-the-docs">
    Click on the Farm and Svelte logos to learn more
  </p>
</main>

<style>
  .logo {
    height: 6em;
    padding: 1.5em;
    will-change: filter;
    transition: filter 300ms;
  }
  .logo:hover {
    filter: drop-shadow(0 0 2em #646cffaa);
  }
  .logo.svelte:hover {
    filter: drop-shadow(0 0 2em #ff3e00aa);
  }
  .read-the-docs {
    color: #888;
  }
</style>
