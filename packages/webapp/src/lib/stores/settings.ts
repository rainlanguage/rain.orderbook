import { cachedWritableStore } from '@rainlanguage/ui-components';

export const hideZeroBalanceVaults = cachedWritableStore<boolean>(
	'settings.hideZeroBalanceVaults',
	true, // default value is true
	(value) => JSON.stringify(value),
	(str) => {
		try {
			return JSON.parse(str) as boolean;
		} catch {
			return true;
		}
	}
);
