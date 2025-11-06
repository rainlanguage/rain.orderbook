import type { LocalDbStatus } from '@rainlanguage/orderbook';
import { writable } from 'svelte/store';

export const localDbStatus = writable<LocalDbStatus>('active');
