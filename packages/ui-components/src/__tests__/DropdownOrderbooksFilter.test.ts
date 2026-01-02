import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { get, writable } from 'svelte/store';
import DropdownOrderbooksFilter from '../lib/components/dropdown/DropdownOrderbooksFilter.svelte';
import { expect, test, describe, beforeEach, vi, type Mock } from 'vitest';
import type { Address } from '@rainlanguage/orderbook';
import { useRaindexClient } from '$lib/hooks/useRaindexClient';

vi.mock('$lib/hooks/useRaindexClient', () => ({
	useRaindexClient: vi.fn()
}));

const mockOrderbooksData = new Map([
	[
		'orderbook1',
		{
			key: 'orderbook1',
			address: '0x1234567890123456789012345678901234567890',
			label: 'Orderbook One',
			network: { chainId: 1 }
		}
	],
	[
		'orderbook2',
		{
			key: 'orderbook2',
			address: '0x2345678901234567890123456789012345678901',
			label: 'Orderbook Two',
			network: { chainId: 1 }
		}
	],
	[
		'orderbook3',
		{
			key: 'orderbook3',
			address: '0x3456789012345678901234567890123456789012',
			label: null,
			network: { chainId: 137 }
		}
	]
]);

describe('DropdownOrderbooksFilter', () => {
	let activeOrderbookAddresses: ReturnType<typeof writable<Address[]>>;

	beforeEach(() => {
		activeOrderbookAddresses = writable([]);

		(useRaindexClient as Mock).mockReturnValue({
			getAllOrderbooks: vi.fn(() => ({
				value: mockOrderbooksData,
				error: undefined
			}))
		});
	});

	describe('Empty state', () => {
		test('displays empty message when no orderbooks available', async () => {
			(useRaindexClient as Mock).mockReturnValue({
				getAllOrderbooks: vi.fn(() => ({
					value: new Map(),
					error: undefined
				}))
			});

			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-orderbooks-filter-button'));

			await waitFor(() => {
				expect(screen.getByText('No orderbooks available')).toBeInTheDocument();
			});
		});

		test('displays custom empty message', async () => {
			(useRaindexClient as Mock).mockReturnValue({
				getAllOrderbooks: vi.fn(() => ({
					value: new Map(),
					error: undefined
				}))
			});

			const customEmptyMessage = 'Orderbook list is empty';

			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: [],
					selectedChainIds: [],
					emptyMessage: customEmptyMessage
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-orderbooks-filter-button'));

			await waitFor(() => {
				expect(screen.getByText(customEmptyMessage)).toBeInTheDocument();
			});
		});
	});

	describe('Error state', () => {
		test('displays error message when getAllOrderbooks returns error', async () => {
			const errorMessage = 'Failed to load orderbooks';
			(useRaindexClient as Mock).mockReturnValue({
				getAllOrderbooks: vi.fn(() => ({
					value: undefined,
					error: { readableMsg: errorMessage }
				}))
			});

			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-orderbooks-filter-button'));

			await waitFor(() => {
				expect(
					screen.getByText(`Cannot load orderbooks list: ${errorMessage}`)
				).toBeInTheDocument();
			});
		});
	});

	describe('Selected orderbooks display', () => {
		test('displays "Select orderbooks" when no orderbooks are selected', () => {
			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: [],
					selectedChainIds: []
				}
			});

			expect(screen.getByText('Select orderbooks')).toBeInTheDocument();
		});

		test('displays "All orderbooks" when all orderbooks are selected', () => {
			const allAddresses = Array.from(mockOrderbooksData.values()).map(
				(ob) => ob.address
			) as Address[];

			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: allAddresses,
					selectedChainIds: []
				}
			});

			expect(screen.getByText('All orderbooks')).toBeInTheDocument();
		});

		test('displays custom all label when all orderbooks are selected', () => {
			const allAddresses = Array.from(mockOrderbooksData.values()).map(
				(ob) => ob.address
			) as Address[];
			const customAllLabel = 'Everything selected';

			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: allAddresses,
					selectedChainIds: [],
					allLabel: customAllLabel
				}
			});

			expect(screen.getByText(customAllLabel)).toBeInTheDocument();
		});

		test('displays count when one orderbook is selected', () => {
			const selectedAddress = Array.from(mockOrderbooksData.values())[0].address as Address;

			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: [selectedAddress],
					selectedChainIds: []
				}
			});

			expect(screen.getByText('1 orderbook')).toBeInTheDocument();
		});

		test('displays plural count when multiple orderbooks are selected', () => {
			const selectedAddresses = Array.from(mockOrderbooksData.values())
				.slice(0, 2)
				.map((ob) => ob.address) as Address[];

			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: selectedAddresses,
					selectedChainIds: []
				}
			});

			expect(screen.getByText('2 orderbooks')).toBeInTheDocument();
		});

		test('updates selected orderbooks when checkbox is clicked', async () => {
			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-orderbooks-filter-button'));

			const checkboxes = screen.getAllByTestId('dropdown-orderbooks-filter-option');
			await fireEvent.click(checkboxes[0]);

			await waitFor(() => {
				const selected = get(activeOrderbookAddresses);
				expect(selected.length).toBe(1);
			});
		});

		test('shows selected orderbooks as checked', async () => {
			const selectedAddress = Array.from(mockOrderbooksData.values())[0].address as Address;

			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: [selectedAddress],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-orderbooks-filter-button'));

			await waitFor(() => {
				expect(screen.getByText('1 orderbook')).toBeInTheDocument();
			});
		});
	});

	describe('Chain filtering', () => {
		test('shows all orderbooks when selectedChainIds is empty', async () => {
			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-orderbooks-filter-button'));

			await waitFor(() => {
				const checkboxes = screen.getAllByTestId('dropdown-orderbooks-filter-option');
				expect(checkboxes.length).toBe(3);
			});
		});

		test('filters orderbooks by selected chain ID', async () => {
			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: [],
					selectedChainIds: [1]
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-orderbooks-filter-button'));

			await waitFor(() => {
				const checkboxes = screen.getAllByTestId('dropdown-orderbooks-filter-option');
				expect(checkboxes.length).toBe(2);
			});
		});

		test('shows orderbooks from multiple selected chains', async () => {
			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: [],
					selectedChainIds: [1, 137]
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-orderbooks-filter-button'));

			await waitFor(() => {
				const checkboxes = screen.getAllByTestId('dropdown-orderbooks-filter-option');
				expect(checkboxes.length).toBe(3);
			});
		});
	});

	describe('Display format', () => {
		test('displays label with truncated address when label exists', async () => {
			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-orderbooks-filter-button'));

			await waitFor(() => {
				expect(screen.getByText(/Orderbook One/)).toBeInTheDocument();
				expect(screen.getByText(/0x1234\.\.\.7890/)).toBeInTheDocument();
			});
		});

		test('displays only truncated address when no label', async () => {
			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: [],
					selectedChainIds: [137]
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-orderbooks-filter-button'));

			await waitFor(() => {
				expect(screen.getByText(/0x3456\.\.\.9012/)).toBeInTheDocument();
			});
		});

		test('displays network name next to each orderbook', async () => {
			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-orderbooks-filter-button'));

			await waitFor(() => {
				expect(screen.getAllByText('Ethereum').length).toBeGreaterThan(0);
				expect(screen.getByText('Polygon')).toBeInTheDocument();
			});
		});
	});

	describe('Search and keyboard navigation', () => {
		test('filters orderbooks based on search term (label)', async () => {
			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-orderbooks-filter-button'));

			const searchInput = screen.getByTestId('orderbooks-filter-search');
			await fireEvent.input(searchInput, { target: { value: 'One' } });

			await waitFor(() => {
				const options = screen.getAllByTestId('dropdown-orderbooks-filter-option');
				expect(options).toHaveLength(1);
				expect(screen.getByText(/Orderbook One/)).toBeInTheDocument();
			});
		});

		test('filters orderbooks based on search term (address)', async () => {
			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-orderbooks-filter-button'));

			const searchInput = screen.getByTestId('orderbooks-filter-search');
			await fireEvent.input(searchInput, { target: { value: '0x1234' } });

			await waitFor(() => {
				const options = screen.getAllByTestId('dropdown-orderbooks-filter-option');
				expect(options).toHaveLength(1);
			});
		});

		test('shows "No orderbooks match your search" when search yields no results', async () => {
			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-orderbooks-filter-button'));

			const searchInput = screen.getByTestId('orderbooks-filter-search');
			await fireEvent.input(searchInput, { target: { value: 'NONEXISTENT' } });

			await waitFor(() => {
				expect(screen.getByText('No orderbooks match your search')).toBeInTheDocument();
			});
		});

		test('keyboard navigation works correctly (ArrowDown/ArrowUp)', async () => {
			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-orderbooks-filter-button'));

			const searchInput = screen.getByTestId('orderbooks-filter-search');

			await fireEvent.keyDown(searchInput, { key: 'ArrowDown' });

			await waitFor(() => {
				const options = screen.getAllByTestId('dropdown-orderbooks-filter-option');
				const secondItemLabel = options[1].closest('label');
				expect(secondItemLabel).toHaveClass('bg-blue-100', 'dark:bg-blue-900');
			});

			await fireEvent.keyDown(searchInput, { key: 'ArrowUp' });

			await waitFor(() => {
				const options = screen.getAllByTestId('dropdown-orderbooks-filter-option');
				const firstItemLabel = options[0].closest('label');
				expect(firstItemLabel).toHaveClass('bg-blue-100', 'dark:bg-blue-900');
			});
		});

		test('Enter key selects highlighted orderbook', async () => {
			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-orderbooks-filter-button'));

			const searchInput = screen.getByTestId('orderbooks-filter-search');

			await fireEvent.input(searchInput, { target: { value: 'One' } });

			await waitFor(() => {
				const options = screen.getAllByTestId('dropdown-orderbooks-filter-option');
				expect(options).toHaveLength(1);
			});

			await fireEvent.keyDown(searchInput, { key: 'Enter' });

			await waitFor(() => {
				const selected = get(activeOrderbookAddresses);
				expect(selected).toContain('0x1234567890123456789012345678901234567890'.toLowerCase());
			});
		});

		test('Escape key clears search', async () => {
			render(DropdownOrderbooksFilter, {
				props: {
					activeOrderbookAddresses,
					selectedOrderbookAddresses: [],
					selectedChainIds: []
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-orderbooks-filter-button'));

			const searchInput = screen.getByTestId('orderbooks-filter-search') as HTMLInputElement;

			await fireEvent.input(searchInput, { target: { value: 'One' } });
			expect(searchInput.value).toBe('One');

			await fireEvent.keyDown(searchInput, { key: 'Escape' });

			await waitFor(() => {
				expect(searchInput.value).toBe('');
			});
		});
	});
});
