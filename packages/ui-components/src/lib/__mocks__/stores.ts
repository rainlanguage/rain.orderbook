import type { ConfigSource } from '@rainlanguage/orderbook';
import { writable } from 'svelte/store';
import settingsFixture from '../__fixtures__/settings-12-11-24.json';

import { type Config } from '@wagmi/core';
import { mockWeb3Config } from './mockWeb3Config';
import type { Page } from '@sveltejs/kit';

const mockSettingsWritable = writable<ConfigSource | undefined>(settingsFixture);
const mockActiveSubgraphsWritable = writable<Record<string, string>>({});
const mockAccountsWritable = writable<Record<string, string>>({});
const mockActiveAccountsItemsWritable = writable<Record<string, string>>({});
const mockActiveOrderStatusWritable = writable<boolean | undefined>(undefined);
const mockOrderHashWritable = writable<string>('');
const mockHideZeroBalanceVaultsWritable = writable<boolean>(false);
const mockActiveNetworkRefWritable = writable<string>('');
const mockActiveOrderbookRefWritable = writable<string>('');
const mockActiveAccountsWritable = writable<Record<string, string>>({});
const mockSubgraphUrlWritable = writable<string>('');
const mockWalletAddressMatchesOrBlankWritable = writable<() => boolean>(() => false);
const mockChainIdWritable = writable<number>(0);
const mockConnectedWritable = writable<boolean>(true);
const mockWagmiConfigWritable = writable<Config>(mockWeb3Config);
const mockShowMyItemsOnlyWritable = writable<boolean>(false);
const mockPageWritable = writable<Page>();

export const mockWalletAddressMatchesOrBlankStore = {
	subscribe: mockWalletAddressMatchesOrBlankWritable.subscribe,
	set: mockWalletAddressMatchesOrBlankWritable.set,
	mockSetSubscribeValue: (value: () => boolean): void =>
		mockWalletAddressMatchesOrBlankWritable.set(value)
};

export const mockSettingsStore = {
	subscribe: mockSettingsWritable.subscribe,
	set: mockSettingsWritable.set,
	mockSetSubscribeValue: (value: ConfigSource | undefined): void => mockSettingsWritable.set(value)
};

export const mockActiveSubgraphsStore = {
	subscribe: mockActiveSubgraphsWritable.subscribe,
	set: mockActiveSubgraphsWritable.set,
	mockSetSubscribeValue: (value: Record<string, string>): void =>
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

export const mockActiveOrderStatusStore = {
	subscribe: mockActiveOrderStatusWritable.subscribe,
	set: mockActiveOrderStatusWritable.set,
	mockSetSubscribeValue: (value: boolean | undefined): void =>
		mockActiveOrderStatusWritable.set(value)
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
	mockSetSubscribeValue: (value: Page): void => mockPageWritable.set(value)
};
