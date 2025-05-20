import { get } from 'svelte/store';
import type { AppStoresInterface } from '@rainlanguage/ui-components';

/**
 * Resets the active network reference based on available networks in settings.
 * If there are networks available, sets the active network to the first one.
 * If no networks are available, sets the active network to undefined.
 *
 * @param activeNetworkRef - The store reference for the active network
 * @param settingsStore - The store containing network settings
 */
export async function resetActiveNetworkRef(
	activeNetworkRef: AppStoresInterface['activeNetworkRef'],
	settingsStore: AppStoresInterface['settings']
) {
	const $settings = get(settingsStore);
	const networks = $settings?.networks;

	if (networks !== undefined && Object.keys(networks).length > 0) {
		activeNetworkRef.set(Object.keys(networks)[0]);
	} else {
		activeNetworkRef.set(undefined);
	}
}
