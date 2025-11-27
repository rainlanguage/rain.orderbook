import type { LocalDbStatusSnapshot } from '@rainlanguage/orderbook';
import { writable } from 'svelte/store';

export const localDbStatus = writable<LocalDbStatusSnapshot>({
	status: 'active',
	error: undefined
});
