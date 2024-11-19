import type { AppStoresInterface } from '../types/stores';
import { writable } from 'svelte/store';

export interface LayoutData {
  stores: AppStoresInterface;
}

export const load = () => {
  return {
    stores: {
      settings: writable<Record<string, string>>({}),
      activeSubgraphs: writable<Record<string, string>>({})
    }
  };
};

export const ssr = false;
