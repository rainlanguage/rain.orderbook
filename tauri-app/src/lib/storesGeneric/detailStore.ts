import { useCachedWritable } from '$lib/storesGeneric/cachedWritable';

export function useDetailStore<T>(key: string, fetchById: (id: string) => Promise<T>) {
  const {subscribe, update} = useCachedWritable<{[id: string]: T}>(key, {}, (value) => JSON.stringify(value), (value) => JSON.parse(value));

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
