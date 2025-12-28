import type { NetworkSyncStatus, LocalDbStatus } from '@rainlanguage/orderbook';
import { writable, derived } from 'svelte/store';

export const networkStatuses = writable<Map<string, NetworkSyncStatus>>(new Map());

export function updateNetworkStatus(status: NetworkSyncStatus) {
	networkStatuses.update((map) => {
		map.set(status.networkKey, status);
		return new Map(map);
	});
}

export const aggregateStatus = derived(networkStatuses, ($map) => {
	if ($map.size === 0) {
		return { status: 'active' as LocalDbStatus, error: undefined };
	}

	const statuses = Array.from($map.values());

	const failure = statuses.find((s) => s.status === 'failure');
	if (failure) {
		return { status: 'failure' as LocalDbStatus, error: failure.error };
	}

	const syncing = statuses.find((s) => s.status === 'syncing');
	if (syncing) {
		return { status: 'syncing' as LocalDbStatus, error: undefined };
	}

	return { status: 'active' as LocalDbStatus, error: undefined };
});

export const localDbStatus = aggregateStatus;
