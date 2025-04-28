import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { get, writable, type Writable } from 'svelte/store';
import { beforeEach, expect, test, describe } from 'vitest';
import DropdownActiveSubgraphs from '../lib/components/dropdown/DropdownActiveSubgraphs.svelte';
import { mockConfig } from '../lib/__mocks__/settings';
import type { Config, SubgraphCfg } from '@rainlanguage/orderbook';

describe('DropdownActiveSubgraphs', () => {
	const mockSettings = {
		orderbook: {
			...mockConfig.orderbook,
			subgraphs: {
				mainnet: {
					key: 'mainnet',
					url: 'mainnet'
				},
				testnet: {
					key: 'testnet',
					url: 'testnet'
				},
				local: {
					key: 'local',
					url: 'local'
				}
			}
		}
	} as unknown as Config;
	let activeSubgraphsStore: Writable<Record<string, SubgraphCfg>>;

	beforeEach(() => {
		activeSubgraphsStore = writable({});
	});

	test('renders correctly', () => {
		render(DropdownActiveSubgraphs, {
			props: {
				settings: mockSettings,
				activeSubgraphs: activeSubgraphsStore
			}
		});
		expect(screen.getByText('Networks')).toBeInTheDocument();
	});

	test('displays the correct number of options', async () => {
		render(DropdownActiveSubgraphs, {
			props: {
				settings: mockSettings,
				activeSubgraphs: activeSubgraphsStore
			}
		});

		await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));

		await waitFor(() => {
			const options = screen.getAllByTestId('dropdown-checkbox-option');
			expect(options).toHaveLength(3);
		});
	});

	test('updates active subgraphs when an option is selected', async () => {
		render(DropdownActiveSubgraphs, {
			props: {
				settings: mockSettings,
				activeSubgraphs: activeSubgraphsStore
			}
		});

		await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));
		await fireEvent.click(screen.getByText('mainnet'));
		await waitFor(() => {
			expect(get(activeSubgraphsStore)).toEqual({
				mainnet: {
					key: 'mainnet',
					url: 'mainnet'
				}
			});
		});

		await fireEvent.click(screen.getByText('testnet'));
		await waitFor(() => {
			expect(get(activeSubgraphsStore)).toEqual({
				mainnet: {
					key: 'mainnet',
					url: 'mainnet'
				},
				testnet: { key: 'testnet', url: 'testnet' }
			});
		});

		await fireEvent.click(screen.getByText('local'));
		await waitFor(() => {
			expect(get(activeSubgraphsStore)).toEqual({
				mainnet: {
					key: 'mainnet',
					url: 'mainnet'
				},
				testnet: { key: 'testnet', url: 'testnet' },
				local: { key: 'local', url: 'local' }
			});
		});
	});
});
