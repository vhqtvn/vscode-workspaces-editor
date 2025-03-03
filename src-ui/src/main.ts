import { mount } from 'svelte';
import App from './App.svelte';
import '../styles.css';
// Initialize the app
const target = document.getElementById('app');
if (target) {
  mount(App, { target });
}

// This is needed for hot module replacement
export { };
