// Application state shared by the screens: the greeting, the status, the route, the "not
// connected" flag, the proposal kept during setup. Routing is by path so that a typed address
// opens the right screen through the daemon's fallback to the entry page (ENK-API-026).

import { api, NotConnected, ApiError, getToken, setToken } from './api';
import type { Greeting, Proposal, Status } from './types';

export type Route = 'status' | 'settings' | 'management' | 'erase' | 'details' | 'protects' | 'notice';
const ROUTES: Route[] = ['status', 'settings', 'management', 'erase', 'details', 'protects', 'notice'];

function routeFromPath(path: string): Route {
  const first = path.replace(/^\/+/, '').split('/')[0] ?? '';
  return (ROUTES as string[]).includes(first) ? (first as Route) : 'status';
}

class AppState {
  greeting = $state<Greeting | null>(null);
  status = $state<Status | null>(null);
  notConnected = $state(false);
  route = $state<Route>(typeof location !== 'undefined' ? routeFromPath(location.pathname) : 'status');
  /** The proposal of the running setup (S-6, S-7); memory only (ENK-OWN-007). */
  proposal = $state<Proposal | null>(null);
  /** The management Wi-Fi name last seen, for the "not connected" sentence. */
  managementName = $state<string | null>(null);
  /** Forces the login screen after the token was refused. */
  loggedOut = $state(false);

  async refreshGreeting(): Promise<Greeting | null> {
    try {
      const g = await api.greeting();
      this.greeting = g;
      this.notConnected = false;
      return g;
    } catch (e) {
      if (e instanceof NotConnected) this.notConnected = true;
      return null;
    }
  }

  async loadStatus(): Promise<Status | null> {
    try {
      const s = await api.status();
      this.status = s;
      this.notConnected = false;
      if (s.management.name) this.managementName = s.management.name;
      else this.managementName = s.settings.management.name;
      return s;
    } catch (e) {
      this.handle(e);
      return null;
    }
  }

  navigate(route: Route): void {
    this.route = route;
    try {
      history.pushState(null, '', `/${route === 'status' ? '' : route}`);
    } catch {
      // history unavailable: the route still changes in memory
    }
    try {
      window.scrollTo(0, 0);
    } catch {
      // not a browser
    }
  }

  /** Common handling of an API failure on any screen. Returns true when it was handled here. */
  handle(e: unknown): boolean {
    if (e instanceof NotConnected) {
      this.notConnected = true;
      return true;
    }
    if (e instanceof ApiError) {
      if (e.detail.code === 'no_session') {
        setToken(null);
        this.loggedOut = true;
        return true;
      }
      if (e.detail.code === 'wrong_state') {
        void this.refreshGreeting();
        return true;
      }
    }
    return false;
  }

  get loggedIn(): boolean {
    return getToken() !== null && !this.loggedOut;
  }

  loginSucceeded(token: string): void {
    setToken(token);
    this.loggedOut = false;
  }
}

export const app = new AppState();

if (typeof window !== 'undefined') {
  window.addEventListener('popstate', () => {
    app.route = routeFromPath(location.pathname);
  });
}
