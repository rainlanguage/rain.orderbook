import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { get, writable, type Writable } from 'svelte/store';
import { beforeEach, expect, test, describe } from 'vitest';
import DropdownActiveNetworks from '../lib/components/dropdown/DropdownActiveNetworks.svelte';
import { mockConfig } from '../lib/__mocks__/settings';
import type { NetworkCfg, NewConfig } from '@rainlanguage/orderbook';

describe('DropdownActiveNetworks', () => {
	const mockSettings = {
		...mockConfig,
		orderbook: {
			...mockConfig.orderbook,
			networks: {
				mainnet: {
					key: 'mainnet',
					url: 'mainnet',
					chainId: 1
				},
				testnet: {
					key: 'testnet',
					url: 'testnet',
					chainId: 2
				},
				local: {
					key: 'local',
					url: 'local',
					chainId: 3
				}
			}
		}
	} as unknown as NewConfig;
	let activeNetworksStore: Writable<Record<string, NetworkCfg>>;

	beforeEach(() => {
		activeNetworksStore = writable({});
	});

	test('renders correctly', () => {
		render(DropdownActiveNetworks, {
			props: {
				settings: mockSettings,
				activeNetworks: activeNetworksStore
			}
		});
		expect(screen.getByText('Networks')).toBeInTheDocument();
	});

	test('displays the correct number of options', async () => {
		render(DropdownActiveNetworks, {
			props: {
				settings: mockSettings,
				activeNetworks: activeNetworksStore
			}
		});

		await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));

		await waitFor(() => {
			const options = screen.getAllByTestId('dropdown-checkbox-option');
			expect(options).toHaveLength(3);
		});
	});

	test('updates active subgraphs when an option is selected', async () => {
		render(DropdownActiveNetworks, {
			props: {
				settings: mockSettings,
				activeNetworks: activeNetworksStore
			}
		});

		await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));
		await fireEvent.click(screen.getByText('mainnet'));
		await waitFor(() => {
			expect(get(activeNetworksStore)).toEqual({
				mainnet: { key: 'mainnet', url: 'mainnet', chainId: 1 }
			});
		});

		await fireEvent.click(screen.getByText('testnet'));
		await waitFor(() => {
			expect(get(activeNetworksStore)).toEqual({
				mainnet: { key: 'mainnet', url: 'mainnet', chainId: 1 },
				testnet: { key: 'testnet', url: 'testnet', chainId: 2 }
			});
		});

		await fireEvent.click(screen.getByText('local'));
		await waitFor(() => {
			expect(get(activeNetworksStore)).toEqual({
				mainnet: { key: 'mainnet', url: 'mainnet', chainId: 1 },
				testnet: { key: 'testnet', url: 'testnet', chainId: 2 },
				local: { key: 'local', url: 'local', chainId: 3 }
			});
		});
	});
});
