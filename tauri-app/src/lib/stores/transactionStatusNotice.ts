import { derived, writable } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import sortBy from 'lodash/sortBy';

import type { TransactionStatusNotice } from '../types/tauriBindings';

export type TransactionStatusNoticeStore = { [id: string]: TransactionStatusNotice };

function useTransactionStatusNoticeStore(autoCloseMs = 5000) {
  const { subscribe, update } = writable<TransactionStatusNoticeStore>({});

  listen<TransactionStatusNotice>('transaction_status_notice', (event) =>
    handleNotice(event.payload),
  );

  function handleNotice(payload: TransactionStatusNotice) {
    update((val) => {
      val[payload.id] = { ...payload };
      return val;
    });

    // Auto remove transaction status notice once transaction is failed or complete
    if (payload.status.type === 'Failed' || payload.status.type === 'Confirmed') {
      setTimeout(() => {
        update((val) => {
          const newVal = { ...val };
          delete newVal[payload.id];
          return newVal;
        });
      }, autoCloseMs);
    }
  }

  return {
    subscribe,
  };
}

export const transactionStatusNotices = useTransactionStatusNoticeStore();

export const transactionStatusNoticesList = derived(transactionStatusNotices, (obj) =>
  sortBy(Object.values(obj), [(val) => new Date(val.created_at), (val) => val.id]),
);
