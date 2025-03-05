import { derived, get, writable, type Invalidator, type Subscriber } from 'svelte/store';
import { toasts } from '../stores/toasts';
import { save } from '@tauri-apps/api/dialog';
import dayjs from 'dayjs';
import { ToastMessageType } from '../types/tauriBindings';
import { cachedWritableStore } from '$lib/storesGeneric/cachedWritableStore';
import { flatten } from 'lodash';
import { reportErrorToSentry, SentrySeverityLevel } from '$lib/services/sentry';

type Unsubscriber = () => void;

export interface ListStore<T> {
  subscribe: (
    subscriber: Subscriber<ListStoreData<T>>,
    invalidate?: Invalidator<ListStoreData<T>>,
  ) => Unsubscriber;
  fetchPage: (page?: number) => Promise<void>;
  fetchFirst: () => Promise<void>;
  fetchPrev: () => Promise<void>;
  fetchNext: () => Promise<void>;
  fetchAll: (firstPage?: number) => Promise<void>;
  exportCsv: () => void;
}

export interface ListStoreData<T> {
  all: T[];
  index: number;
  currentPage: T[];
  isFetching: boolean;
  isFetchingAll: boolean;
  isExporting: boolean;
  isFetchingFirst: boolean;
}

export type AllPages<T> = Array<Array<T>>;

const cachedWritablePages = <T>(key: string) =>
  cachedWritableStore<AllPages<T>>(
    key,
    [],
    (value) => JSON.stringify(value),
    (value) => JSON.parse(value),
  );

export function listStore<T>(
  key: string,
  fetchPageHandler: (page: number) => Promise<Array<T>>,
  writeCsvHandler: (path: string) => Promise<void>,
) {
  const allPages = cachedWritablePages<T>(key);
  const pageIndex = writable(0);
  const isFetching = writable(false);
  const isFetchingAll = writable(false);
  const isExporting = writable(false);

  const page = derived(allPages, ($allPages) => (page: number) => $allPages[page] || []);

  const { subscribe } = derived(
    [allPages, page, pageIndex, isFetching, isFetchingAll, isExporting],
    ([$allPages, $page, $pageIndex, $isFetching, $isFetchingAll, $isExporting]) => ({
      all: flatten<T>(Object.values($allPages) as T[]) || [],
      index: $pageIndex,
      currentPage: $page($pageIndex),
      isFetching: $isFetching,
      isFetchingAll: $isFetchingAll,
      isExporting: $isExporting,
      isFetchingFirst: $isFetching && $allPages.length === 0,
    }),
  );

  async function fetchPage(page: number = 1) {
    const res: Array<T> = await fetchPageHandler(page);
    if (res.length === 0) {
      throw Error('No results found');
    }

    allPages.update((val) => {
      val[page] = res;
      return val;
    });
  }

  async function swrvPage(newPage: number, displayError: boolean = false) {
    if (newPage < 0) return;
    if (get(isFetching)) return;

    isFetching.set(true);
    const promise = fetchPage(newPage);
    if (get(page)(newPage)?.length === 0) {
      try {
        await promise;
        pageIndex.set(newPage);
      } catch (e) {
        reportErrorToSentry(e, SentrySeverityLevel.Info);
        if (displayError) {
          toasts.error((e as Error).message);
        }
      }
    } else {
      pageIndex.set(newPage);
    }
    isFetching.set(false);
  }

  async function fetchAll(firstPage = 0) {
    if (get(isFetchingAll)) return;

    let newPage = firstPage;
    let hasMorePages = true;

    isFetchingAll.set(true);
    while (hasMorePages) {
      try {
        await fetchPage(newPage);
        newPage += 1;
      } catch (e) {
        // Because we don't know the total number of pages in advance,
        // we have to attempt to fetch the next page until we receive an error.
        reportErrorToSentry(e, SentrySeverityLevel.Info);
        hasMorePages = false;
      }
    }
    isFetchingAll.set(false);
  }

  const fetchPrev = () => swrvPage(get(pageIndex) - 1, true);
  const fetchNext = () => swrvPage(get(pageIndex) + 1, true);
  const fetchFirst = () => swrvPage(0);

  async function exportCsv() {
    isExporting.set(true);
    try {
      const path = await save({
        title: 'Save CSV As',
        defaultPath: `${key}_${dayjs().toISOString()}.csv`,
      });
      if (path) {
        await writeCsvHandler(path);
        toasts.add({
          message_type: ToastMessageType.Success,
          text: `Exported to CSV at ${path}`,
        });
      }
    } catch (e) {
      reportErrorToSentry(e);
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
    fetchAll,
    exportCsv,
  } as ListStore<T>;
}
