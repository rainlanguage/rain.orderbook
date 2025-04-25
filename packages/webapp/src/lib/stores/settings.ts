import { cachedWritableStore, type ConfigSource } from '@rainlanguage/ui-components';

export const settings = cachedWritableStore<ConfigSource | undefined>(
  'settings',
  undefined,
  (value) => JSON.stringify(value),
  (str) => {
    try {
      return JSON.parse(str) as ConfigSource;
    } catch {
      return undefined;
    }
  },
);

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


// subgraphs
export const activeSubgraphs = cachedWritableStore<Record<string, string>>(
  'settings.activeSubgraphs',
  {},
  JSON.stringify,
  (s) => {
    try {
      return JSON.parse(s);
    } catch {
      return {};
    }
  },
);