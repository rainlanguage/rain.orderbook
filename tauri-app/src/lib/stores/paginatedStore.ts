import { derived, get, writable } from 'svelte/store';
import { toasts } from './toasts';

export function usePaginatedCachedStore<T>(key: string, fetchPageHandler: (page: number, pageSize: number) => Promise<Array<T>>) {
  const allPages = writable<{[page: number]: Array<T>}>(localStorage.getItem(key) ? JSON.parse(localStorage.getItem(key) as string) : []);
  const pageIndex = writable(1);
  const isFetching = writable(false);

  allPages.subscribe(value => {
    if(value) {
      localStorage.setItem(key, JSON.stringify(value));
    } else {
      localStorage.setItem(key, JSON.stringify([]));
    }
  });

  const page = derived(allPages, $allPages => (page: number) => $allPages[page] || []);

  const { subscribe } = derived([page, pageIndex, isFetching], ([$page, $pageIndex, $isFetching]) => ({
    index: $pageIndex,
    currentPage: $page($pageIndex),
    page: $page,
    isFetching: $isFetching,
  }));

  async function fetchPage(page: number = 1, pageSize: number = 10) {
    const res: Array<T> = await fetchPageHandler(page, pageSize);
    if(res.length === 0) throw Error("No results found");

    allPages.update((val) => {
      val[page] = res;
      return val;
    });
  }

  async function swrvPage(newPage: number) {
    if(newPage <= 0) return;
    if(get(isFetching)) return;

    isFetching.set(true);
    const promise = fetchPage(newPage);
    if(get(page)(newPage)?.length === 0) {
      try {
        await promise;
        pageIndex.set(newPage);
      } catch(e) {
        toasts.error(e);
      }
    } else {
      pageIndex.set(newPage);
    }
    isFetching.set(false);
  }

  const fetchPrev = () => swrvPage(get(pageIndex) - 1);
  const fetchNext = () => swrvPage(get(pageIndex) + 1);

  return {
    subscribe,
    fetchPage,
    fetchPrev,
    fetchNext,
  };
}
