import type { AppStoresInterface } from '@rainlanguage/ui-components';
import { get } from 'svelte/store';

/**
 * Resets the active orderbook reference based on available network orderbooks.
 * If there are orderbooks available, sets the active orderbook to the first one.
 * If no orderbooks are available, sets the active orderbook to undefined.
 *
 * @param activeOrderbookRef - The store reference for the active orderbook
 * @param activeNetworkOrderbooksStore - The store containing available network orderbooks
 */
export function resetActiveOrderbookRef(
	activeOrderbookRef: AppStoresInterface['activeOrderbookRef'],
	activeNetworkOrderbooksStore: AppStoresInterface['activeNetworkOrderbooks']
) {
	try {
		const $activeNetworkOrderbooks = get(activeNetworkOrderbooksStore);

		if (
			$activeNetworkOrderbooks !== undefined &&
			Object.keys($activeNetworkOrderbooks).length > 0
		) {
			activeOrderbookRef.set(Object.keys($activeNetworkOrderbooks)[0]);
		} else {
			activeOrderbookRef.set(undefined);
		}
	} catch {
		throw new Error('Error resetting active orderbook');
	}
}
