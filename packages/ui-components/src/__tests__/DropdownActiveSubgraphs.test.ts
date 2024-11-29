import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { get, writable, type Writable } from 'svelte/store';
import { beforeEach, expect, test, describe } from 'vitest';
import DropdownActiveSubgraphs from '../lib/components/dropdown/DropdownActiveSubgraphs.svelte';
import { mockConfigSource } from '../lib/__mocks__/settings';

describe('DropdownActiveSubgraphs', () => {
	const mockSettings = {
		...mockConfigSource,
		subgraphs: {
			mainnet: 'mainnet',
			testnet: 'testnet',
			local: 'local'
		}
	};
	let activeSubgraphsStore: Writable<Record<string, string>>;

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
			expect(get(activeSubgraphsStore)).toEqual({ mainnet: 'mainnet' });
		});

		await fireEvent.click(screen.getByText('testnet'));
		await waitFor(() => {
			expect(get(activeSubgraphsStore)).toEqual({ mainnet: 'mainnet', testnet: 'testnet' });
		});

		await fireEvent.click(screen.getByText('local'));
		await waitFor(() => {
			expect(get(activeSubgraphsStore)).toEqual({
				mainnet: 'mainnet',
				testnet: 'testnet',
				local: 'local'
			});
		});
	});
});
