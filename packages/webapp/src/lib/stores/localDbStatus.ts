import type {
	NetworkSyncStatus,
	LocalDbStatus,
	OrderbookSyncStatus
} from '@rainlanguage/orderbook';
import { writable, derived } from 'svelte/store';

export const networkStatuses = writable<Map<number, NetworkSyncStatus>>(new Map());

export const orderbookStatuses = writable<Map<string, OrderbookSyncStatus>>(new Map());

function orderbookStatusKey(status: OrderbookSyncStatus): string {
	return `${status.obId.chainId}:${status.obId.orderbookAddress}`;
}

function isOrderbookSyncStatus(
	status: NetworkSyncStatus | OrderbookSyncStatus
): status is OrderbookSyncStatus {
	return 'obId' in status;
}

export function updateNetworkStatus(status: NetworkSyncStatus) {
	networkStatuses.update((map) => {
		map.set(status.chainId, status);
		return new Map(map);
	});
}

export function updateOrderbookStatus(status: OrderbookSyncStatus) {
	orderbookStatuses.update((map) => {
		const key = orderbookStatusKey(status);
		map.set(key, status);
		return new Map(map);
	});
}

export function updateStatus(status: NetworkSyncStatus | OrderbookSyncStatus) {
	if (isOrderbookSyncStatus(status)) {
		updateOrderbookStatus(status);
	} else {
		updateNetworkStatus(status);
	}
}

export const aggregateStatus = derived(
	[networkStatuses, orderbookStatuses],
	([$networkMap, $orderbookMap]) => {
		const allStatuses: { status: LocalDbStatus; error?: string }[] = [
			...Array.from($networkMap.values()),
			...Array.from($orderbookMap.values())
		];

		if (allStatuses.length === 0) {
			return { status: 'active' as LocalDbStatus, error: undefined };
		}

		const failure = allStatuses.find((s) => s.status === 'failure');
		if (failure) {
			return { status: 'failure' as LocalDbStatus, error: failure.error };
		}

		const syncing = allStatuses.find((s) => s.status === 'syncing');
		if (syncing) {
			return { status: 'syncing' as LocalDbStatus, error: undefined };
		}

		return { status: 'active' as LocalDbStatus, error: undefined };
	}
);

export const localDbStatus = aggregateStatus;
