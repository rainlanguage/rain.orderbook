import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { get, writable, type Writable } from 'svelte/store';
import { beforeEach, expect, test, describe } from 'vitest';
import DropdownActiveNetworks from '../lib/components/dropdown/DropdownActiveNetworks.svelte';
import { mockConfig } from '../lib/__mocks__/settings';
import type { NewConfig } from '@rainlanguage/orderbook';

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
	let selectedChainIdsStore: Writable<number[]>;

	beforeEach(() => {
		selectedChainIdsStore = writable([]);
	});

	test('renders correctly', () => {
		render(DropdownActiveNetworks, {
			props: {
				settings: mockSettings,
				selectedChainIds: selectedChainIdsStore
			}
		});
		expect(screen.getByText('Networks')).toBeInTheDocument();
	});

	test('displays the correct number of options', async () => {
		render(DropdownActiveNetworks, {
			props: {
				settings: mockSettings,
				selectedChainIds: selectedChainIdsStore
			}
		});

		await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));

		await waitFor(() => {
			const options = screen.getAllByTestId('dropdown-checkbox-option');
			expect(options).toHaveLength(3);
		});
	});

	test('updates selected chain ids when an option is selected', async () => {
		render(DropdownActiveNetworks, {
			props: {
				settings: mockSettings,
				selectedChainIds: selectedChainIdsStore
			}
		});

		await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));
		await fireEvent.click(screen.getByText('Ethereum'));
		await waitFor(() => {
			expect(get(selectedChainIdsStore)).toEqual([1]);
		});

		await fireEvent.click(screen.getByText('Expanse Network'));
		await waitFor(() => {
			expect(get(selectedChainIdsStore)).toEqual([1, 2]);
		});

		await fireEvent.click(screen.getByText('local'));
		await waitFor(() => {
			expect(get(selectedChainIdsStore)).toEqual([1, 2, 3]);
		});
	});
});
