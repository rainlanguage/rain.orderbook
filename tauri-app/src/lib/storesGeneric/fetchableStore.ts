import { derived, writable } from 'svelte/store';
import { cachedWritableStore } from '@rainlanguage/ui-components';
import { toasts } from '$lib/stores/toasts';
import { reportErrorToSentry } from '$lib/services/sentry';

interface FetchableStoreData<T> {
  value: T;
  isFetching: boolean;
}

export function fetchableStore<T>(
  key: string,
  defaultValue: T,
  handleFetch: () => Promise<T>,
  serialize: (value: T) => string,
  deserialize: (s: string) => T,
) {
  const value = cachedWritableStore<T>(key, defaultValue, serialize, deserialize);
  const isFetching = writable(false);

  const { subscribe } = derived([value, isFetching], ([$value, $isFetching]) => ({
    value: $value,
    isFetching: $isFetching,
  }));

  async function fetch() {
    isFetching.set(true);
    try {
      const res: T = await handleFetch();
      value.set(res);
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error(e as string);
    }
    isFetching.set(false);
  }

  return {
    subscribe,
    set: (v: FetchableStoreData<T>) => value.set(v.value),
    fetch,
  };
}

export const fetchableIntStore = (key: string, handleFetch: () => Promise<number>) =>
  fetchableStore<number>(
    key,
    0,
    handleFetch,
    (v) => v.toString(),
    (s) => parseInt(s),
  );
