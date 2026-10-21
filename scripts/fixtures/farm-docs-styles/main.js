import './utilities.css';
import './brand.scss';
import styles from './panel.module.scss';
document.getElementById('root').innerHTML = `<main class="flex md:grid p-4 bg-primary dark:bg-background animate-shimmer"><div class="${styles.panel}">Farm styles</div><div class="brand-child">Brand nesting</div></main>`;
