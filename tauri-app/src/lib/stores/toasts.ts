import { derived, writable } from 'svelte/store';
import type { ToastDataStore, ToastPayload } from '$lib/types/toast';
import { v4 as uuidv4} from 'uuid';
import { listen } from '@tauri-apps/api/event';
import  sortBy from 'lodash/sortBy';

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
