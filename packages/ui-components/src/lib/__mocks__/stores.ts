import type { AccountCfg, NewConfig, SubgraphCfg } from '@rainlanguage/orderbook';
import { parseYaml } from '@rainlanguage/orderbook';
import { writable } from 'svelte/store';
import settingsYamlContent from '../__fixtures__/settings.yaml?raw';

import { type Config } from '@wagmi/core';
import { mockWeb3Config } from './mockWeb3Config';

vi.mock(import('@rainlanguage/orderbook'), async (importOriginal) => {
	const actual = await importOriginal();
	return {
		...actual
	};
});

// Parse the YAML settings
const parseResult = parseYaml([settingsYamlContent]);
if (parseResult.error) {
	throw new Error(`Failed to parse settings YAML: ${parseResult.error.readableMsg}`);
}
const settingsFixture = parseResult.value;

const initialPageState = {
	data: {
		stores: { settings: {} },
		dotrain: 'some dotrain content',
		deployment: { key: 'deploy-key' },
		strategyDetail: {}
	},
	url: new URL('http://localhost:3000/deploy'),
	params: {},
	form: {},
	status: 200,
	error: null,
	route: {
		id: null
	}
};

const mockPageWritable = writable<typeof initialPageState>(initialPageState);
const mockSettingsWritable = writable<NewConfig>(settingsFixture as unknown as NewConfig);
const mockActiveSubgraphsWritable = writable<Record<string, SubgraphCfg>>({});
const mockAccountsWritable = writable<Record<string, AccountCfg>>({});
const mockActiveAccountsItemsWritable = writable<Record<string, string>>({});
const mockShowInactiveOrdersWritable = writable<boolean>(true);
const mockOrderHashWritable = writable<string>('');
const mockHideZeroBalanceVaultsWritable = writable<boolean>(false);
const mockActiveNetworkRefWritable = writable<string>('');
const mockActiveOrderbookRefWritable = writable<string>('');
const mockActiveAccountsWritable = writable<Record<string, string>>({});
const mockSubgraphUrlWritable = writable<string>('');
const mockChainIdWritable = writable<number>(0);
const mockConnectedWritable = writable<boolean>(true);
const mockWagmiConfigWritable = writable<Config>(mockWeb3Config);
const mockShowMyItemsOnlyWritable = writable<boolean>(false);

export const mockSettingsStore = {
	subscribe: mockSettingsWritable.subscribe,
	set: mockSettingsWritable.set,
	mockSetSubscribeValue: (value: NewConfig): void => mockSettingsWritable.set(value)
};

export const mockActiveSubgraphsStore = {
	subscribe: mockActiveSubgraphsWritable.subscribe,
	set: mockActiveSubgraphsWritable.set,
	mockSetSubscribeValue: (value: Record<string, SubgraphCfg>): void =>
		mockActiveSubgraphsWritable.set(value)
};

export const mockAccountsStore = {
	subscribe: mockAccountsWritable.subscribe
};

export const mockActiveAccountsItemsStore = {
	subscribe: mockActiveAccountsItemsWritable.subscribe,
	set: mockActiveAccountsItemsWritable.set,
	mockSetSubscribeValue: (value: Record<string, string>): void =>
		mockActiveAccountsItemsWritable.set(value)
};

export const mockShowInactiveOrdersStore = {
	subscribe: mockShowInactiveOrdersWritable.subscribe,
	set: mockShowInactiveOrdersWritable.set,
	mockSetSubscribeValue: (value: boolean): void => mockShowInactiveOrdersWritable.set(value)
};

export const mockOrderHashStore = {
	subscribe: mockOrderHashWritable.subscribe,
	set: mockOrderHashWritable.set,
	mockSetSubscribeValue: (value: string): void => mockOrderHashWritable.set(value)
};

export const mockHideZeroBalanceVaultsStore = {
	subscribe: mockHideZeroBalanceVaultsWritable.subscribe,
	set: mockHideZeroBalanceVaultsWritable.set,
	mockSetSubscribeValue: (value: boolean): void => mockHideZeroBalanceVaultsWritable.set(value)
};

export const mockActiveNetworkRefStore = {
	subscribe: mockActiveNetworkRefWritable.subscribe,
	set: mockActiveNetworkRefWritable.set,
	mockSetSubscribeValue: (value: string): void => mockActiveNetworkRefWritable.set(value)
};

export const mockActiveOrderbookRefStore = {
	subscribe: mockActiveOrderbookRefWritable.subscribe,
	set: mockActiveOrderbookRefWritable.set,
	mockSetSubscribeValue: (value: string): void => mockActiveOrderbookRefWritable.set(value)
};

export const mockActiveAccountsStore = {
	subscribe: mockActiveAccountsWritable.subscribe,
	set: mockActiveAccountsWritable.set,
	mockSetSubscribeValue: (value: Record<string, string>): void =>
		mockActiveAccountsWritable.set(value)
};

export const mockSubgraphUrlStore = {
	subscribe: mockSubgraphUrlWritable.subscribe,
	set: mockSubgraphUrlWritable.set,
	mockSetSubscribeValue: (value: string): void => mockSubgraphUrlWritable.set(value)
};

export const mockChainIdStore = {
	subscribe: mockChainIdWritable.subscribe,
	set: mockChainIdWritable.set,
	mockSetSubscribeValue: (value: number): void => mockChainIdWritable.set(value)
};

export const mockConnectedStore = {
	subscribe: mockConnectedWritable.subscribe,
	set: mockConnectedWritable.set,
	update: mockConnectedWritable.update,
	mockSetSubscribeValue: (value: boolean): void => mockConnectedWritable.set(value)
};

export const mockWagmiConfigStore = {
	subscribe: mockWagmiConfigWritable.subscribe,
	set: mockWagmiConfigWritable.set,
	update: mockWagmiConfigWritable.update,
	mockSetSubscribeValue: (value: Config): void => mockWagmiConfigWritable.set(value)
};

export const mockShowMyItemsOnlyStore = {
	subscribe: mockShowMyItemsOnlyWritable.subscribe,
	set: mockShowMyItemsOnlyWritable.set,
	mockSetSubscribeValue: (value: boolean): void => mockShowMyItemsOnlyWritable.set(value)
};

export const mockPageStore = {
	subscribe: mockPageWritable.subscribe,
	set: mockPageWritable.set,
	mockSetSubscribeValue: (newValue: Partial<typeof initialPageState>): void => {
		mockPageWritable.update((currentValue) => ({
			...currentValue,
			...newValue
		}));
	},
	reset: () => mockPageWritable.set(initialPageState)
};
