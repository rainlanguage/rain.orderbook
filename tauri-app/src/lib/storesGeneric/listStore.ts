import { derived, get, writable, type Invalidator, type Subscriber } from 'svelte/store';
import { toasts } from '../stores/toasts';
import { save } from '@tauri-apps/api/dialog';
import dayjs from 'dayjs';
import { ToastMessageType } from '$lib/typeshare/toast';
import { cachedWritableStore } from '$lib/storesGeneric/cachedWritableStore';

type Unsubscriber = () => void;

export interface PaginatedCachedStore<T> {
    subscribe: ( subscriber: Subscriber<Page<T>>, invalidate?: Invalidator<Page<T>>) => Unsubscriber,
    fetchPage: (page?: number) => Promise<void>;
    fetchFirst: () => Promise<void>;
    fetchPrev: () => Promise<void>;
    fetchNext: () => Promise<void>;
    exportCsv: () => void;
}

export interface Page<T> {
  index: number;
  currentPage: T[];
  page: (page: number) => T[];
  isFetching: boolean;
  isExporting: boolean;
}

export interface AllPages<T> {
  [pageIndex: number]: Array<T>
}


const cachedWritablePages = <T>(key: string) => cachedWritableStore<AllPages<T>>(key, [], (value) => JSON.stringify(value), (value) => JSON.parse(value));

export function listStore<T>(key: string, fetchPageHandler: (page: number) => Promise<Array<T>>, writeCsvHandler:  (path: string) => Promise<void>) {
  const allPages = cachedWritablePages<T>(key);
  const pageIndex = writable(1);
  const isFetching = writable(false);
  const isExporting = writable(false);

  const page = derived(allPages, $allPages => (page: number) => $allPages[page] || []);

  const { subscribe } = derived([page, pageIndex, isFetching, isExporting], ([$page, $pageIndex, $isFetching, $isExporting]) => ({
    index: $pageIndex,
    currentPage: $page($pageIndex),
    page: $page,
    isFetching: $isFetching,
    isExporting: $isExporting
  }));

  async function fetchPage(page: number = 1) {
    const res: Array<T> = await fetchPageHandler(page);
    if(res.length === 0) {
      throw Error("No results found");
    }

    allPages.update((val) => {
      val[page] = res;
      return val;
    });
  }

  async function swrvPage(newPage: number, displayError: boolean = false) {
    if(newPage <= 0) return;
    if(get(isFetching)) return;

    isFetching.set(true);
    const promise = fetchPage(newPage);
    if(get(page)(newPage)?.length === 0) {
      try {
        await promise;
        pageIndex.set(newPage);
      // eslint-disable-next-line no-empty
      } catch(e) {
        if(displayError) {
          toasts.error((e as Error).message);
        }
      }
    } else {
      pageIndex.set(newPage);
    }
    isFetching.set(false);
  }

  const fetchPrev = () => swrvPage(get(pageIndex) - 1, true);
  const fetchNext = () => swrvPage(get(pageIndex) + 1, true);
  const fetchFirst = () => swrvPage(1);


  async function exportCsv() {
    isExporting.set(true);
    try {
      const path = await save({
        title: 'Save CSV As',
        defaultPath: `${key}_${dayjs().toISOString()}.csv`,
      });
      if(path) {
        await writeCsvHandler(path);
        toasts.add({
          message_type: ToastMessageType.Success,
          text: `Exported to CSV at ${path}`,
          break_text: true
        });
      }
    } catch(e) {
      toasts.error(e as string);
    }
    isExporting.set(false);
  }

  return {
    subscribe,
    fetchFirst,
    fetchPage,
    fetchPrev,
    fetchNext,
    exportCsv,
  } as PaginatedCachedStore<T>;
}
