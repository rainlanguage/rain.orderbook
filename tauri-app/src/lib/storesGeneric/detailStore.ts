import { cachedWritableStore } from '$lib/storesGeneric/cachedWritableStore';

export function detailStore<T>(key: string, fetchById: (id: string) => Promise<T>) {
  const {subscribe, update} = cachedWritableStore<{[id: string]: T}>(key, {}, (value) => JSON.stringify(value), (value) => JSON.parse(value));

  subscribe(value => {
    if(value) {
      localStorage.setItem(key, JSON.stringify(value));
    } else {
      localStorage.setItem(key, JSON.stringify({}));
    }
  });

  async function refetch(id: string) {
    const res: T = await fetchById(id);
    update((value) => {
      return {... value, [id]: res};
    });
  }

  return {
    subscribe,
    refetch
  }
}
