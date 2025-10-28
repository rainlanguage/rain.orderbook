import { writable } from 'svelte/store';

export const dbSyncStatus = writable<string>('');
export const dbSyncLastBlock = writable<string | null>(null);
export const dbSyncLastSyncTime = writable<Date | null>(null);
export const dbSyncIsActive = writable<boolean>(false);
export const dbSyncIsRunning = writable<boolean>(false);
