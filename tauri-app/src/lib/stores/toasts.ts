import { derived, writable } from 'svelte/store';
import { v4 as uuidv4} from 'uuid';
import { listen } from '@tauri-apps/api/event';
import  sortBy from 'lodash/sortBy';

import type { ToastPayload } from "$lib/typeshare/toast";

export type ToastData = ToastPayload & { timestamp: Date; id: string };

export type ToastDataStore = { [id: string]: ToastData };

function useToastsStore(autohideMs = 5000) {
  const toasts = writable<ToastDataStore>({});

  listen<ToastPayload>('toast', (event) => add(event.payload));

  function add(payload: ToastPayload) {
    const id = uuidv4();
    const timestamp = new Date();
    toasts.update((val) => {
      val[id] = { ...payload, timestamp, id };
      return val;
    });

    setTimeout(() => {
      toasts.update((val) => {
        delete val[id];
        return val;
      });
    }, autohideMs);
  }
  return {
    subscribe: toasts.subscribe,
    add
  }
}

export const toasts = useToastsStore();

export const toastsList = derived(toasts, (toasts) => sortBy(Object.values(toasts), [(val) => val.timestamp, (val) => val.id]))