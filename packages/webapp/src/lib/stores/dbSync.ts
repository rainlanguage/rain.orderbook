import { writable } from 'svelte/store';

export const dbSyncStatus = writable<string>('');
export const dbSyncIsActive = writable<boolean>(false);
export const dbSyncIsRunning = writable<boolean>(false);
