import { mount } from 'svelte';
import App from './App.svelte';
import './app.css';
import { i18n } from './lib/i18n/index.svelte';

document.documentElement.lang = i18n.lang;

mount(App, { target: document.getElementById('app') as HTMLElement });
