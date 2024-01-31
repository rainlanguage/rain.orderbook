import { derived, get, writable, type Invalidator, type Subscriber } from 'svelte/store';
import { toasts } from './toasts';


type Unsubscriber = () => void;

export interface PaginatedCachedStore<T> {
    subscribe: ( subscriber: Subscriber<Page<T>>, invalidate?: Invalidator<Page<T>>) => Unsubscriber,
    fetchPage: (page?: number, pageSize?: number) => Promise<void>;
    fetchPrev: () => Promise<void>;
    fetchNext: () => Promise<void>;
}

export interface Page<T> {
  index: number; currentPage: T[]; page: (page: number) => T[]; isFetching: boolean;
}

export interface AllPages<T> {
  [pageIndex: number]: Array<T>
}

export function usePaginatedCachedStore<T>(key: string, fetchPageHandler: (page: number, pageSize: number) => Promise<Array<T>>) {
  const allPages = writable<AllPages<T>>(localStorage.getItem(key) ? JSON.parse(localStorage.getItem(key) as string) : []);
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
    if(res.length === 0) {
      toasts.error("No results found");
      throw Error("No results found");
    }

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
      // eslint-disable-next-line no-empty
      } catch(e) {}
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
  } as PaginatedCachedStore<T>;
}
