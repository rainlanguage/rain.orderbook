import { toasts } from '$lib/stores/toasts';
import { cachedWritableStore } from '$lib/storesGeneric/cachedWritableStore';
import {
  derived,
  writable,
  type Invalidator,
  type Subscriber,
  type Unsubscriber,
} from 'svelte/store';
import { reportErrorToSentry } from '$lib/services/sentry';

export interface DetailStore<T> {
  subscribe: (
    subscriber: Subscriber<DetailStoreData<T>>,
    invalidate?: Invalidator<DetailStoreData<T>>,
  ) => Unsubscriber;
  refetch: (id: string) => void;
}

export interface DetailStoreData<T> {
  [id: string]: T;
}

export function detailStore<T>(key: string, fetchById: (id: string) => Promise<T>) {
  const data = cachedWritableStore<DetailStoreData<T>>(
    key,
    {},
    (value) => JSON.stringify(value),
    (value) => JSON.parse(value),
  );
  const isFetching = writable(false);

  const { subscribe } = derived([data, isFetching], ([$data, $isFetching]) => ({
    data: $data,
    isFetching: $isFetching,
  }));

  async function refetch(id: string) {
    isFetching.set(true);

    try {
      const res: T = await fetchById(id);

      data.update((value) => {
        return { ...value, [id]: res };
      });
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error(e as string);
    }

    isFetching.set(false);
  }

  return {
    subscribe,
    refetch,
  };
}
