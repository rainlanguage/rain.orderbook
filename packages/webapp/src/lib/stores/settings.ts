import { cachedWritableStore } from '@rainlanguage/ui-components';

/**
 * A persistent store that controls whether vaults with zero balance should be hidden in the UI.
 *
 * This setting is saved to local storage and persists between sessions.
 *
 * @default true - Zero balance vaults are hidden by default
 * @returns A writable store containing a boolean value
 */
export const hideZeroBalanceVaults = cachedWritableStore<boolean>(
	'settings.hideZeroBalanceVaults',
	true, // default value is true
	(value) => JSON.stringify(value),
	(str) => {
		try {
			const value = JSON.parse(str);
			return typeof value === 'boolean' ? value : true;
		} catch {
			return true;
		}
	}
);