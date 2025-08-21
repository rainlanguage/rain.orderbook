import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { readable } from 'svelte/store';
import DropdownTokensFilter from '../lib/components/dropdown/DropdownTokensFilter.svelte';
import { expect, test, describe, beforeEach, vi } from 'vitest';
import type { Address, RaindexVaultToken } from '@rainlanguage/orderbook';
import type { QueryObserverResult } from '@tanstack/svelte-query';

describe('DropdownTokensFilter', () => {
	let selectedTokens: Address[];
	let onChange: ReturnType<typeof vi.fn>;

	const mockTokensData = [
		{
			id: 'TOKEN1',
			address: '0x1234567890123456789012345678901234567890',
			symbol: 'TOKEN1',
			name: 'Test Token 1',
			decimals: BigInt(18),
			chainId: 1
		},
		{
			id: 'TOKEN2',
			address: '0x2345678901234567890123456789012345678901',
			symbol: 'TOKEN2',
			name: 'Test Token 2',
			decimals: BigInt(18),
			chainId: 1
		},
		{
			id: 'ETH',
			address: '0x3456789012345678901234567890123456789012',
			symbol: 'ETH',
			name: 'Ethereum',
			decimals: BigInt(18),
			chainId: 1
		}
	] as unknown as RaindexVaultToken[];

	beforeEach(() => {
		selectedTokens = [];
		onChange = vi.fn();
	});

	function createMockTokensQuery(
		data: RaindexVaultToken[] | undefined = undefined,
		isLoading = false,
		isError = false,
		error: Error | null = null
	) {
		return readable({
			data,
			isLoading,
			isError,
			error
		} as QueryObserverResult<RaindexVaultToken[], Error>);
	}

	describe('Loading state', () => {
		test('displays loading message when tokens are loading', async () => {
			const tokensQuery = createMockTokensQuery(undefined, true);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					selectedTokens,
					onChange
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			await waitFor(() => {
				expect(screen.getByText('Loading tokens...')).toBeInTheDocument();
			});
		});

		test('displays custom loading message', async () => {
			const tokensQuery = createMockTokensQuery(undefined, true);
			const customLoadingMessage = 'Fetching token data...';

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					selectedTokens,
					onChange,
					loadingMessage: customLoadingMessage
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			await waitFor(() => {
				expect(screen.getByText(customLoadingMessage)).toBeInTheDocument();
			});
		});
	});

	describe('Error state', () => {
		test('displays error message when query fails', async () => {
			const errorMessage = 'Network connection failed';
			const tokensQuery = createMockTokensQuery(undefined, false, true, new Error(errorMessage));

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					selectedTokens,
					onChange
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			await waitFor(() => {
				expect(screen.getByText(/Cannot load tokens list:/)).toBeInTheDocument();
			});
		});
	});

	describe('Empty state', () => {
		test('displays empty message when no tokens are available', async () => {
			const tokensQuery = createMockTokensQuery([]);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					selectedTokens,
					onChange
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			await waitFor(() => {
				expect(screen.getByText('No tokens available')).toBeInTheDocument();
			});
		});

		test('displays custom empty message', async () => {
			const tokensQuery = createMockTokensQuery([]);
			const customEmptyMessage = 'No token data found';

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					selectedTokens,
					onChange,
					emptyMessage: customEmptyMessage
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			await waitFor(() => {
				expect(screen.getByText(customEmptyMessage)).toBeInTheDocument();
			});
		});
	});

	describe('Token selection', () => {
		test('shows "Select tokens" when no tokens are selected', () => {
			const tokensQuery = createMockTokensQuery(mockTokensData);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					selectedTokens: [],
					onChange
				}
			});

			expect(screen.getByText('Select tokens')).toBeInTheDocument();
		});

		test('shows "All tokens" when all tokens are selected', () => {
			const allTokenAddresses = mockTokensData.map((token) => token.address as Address);
			const tokensQuery = createMockTokensQuery(mockTokensData);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					selectedTokens: allTokenAddresses,
					onChange
				}
			});

			expect(screen.getByText('All tokens')).toBeInTheDocument();
		});

		test('shows custom all label when all tokens are selected', () => {
			const allTokenAddresses = mockTokensData.map((token) => token.address as Address);
			const tokensQuery = createMockTokensQuery(mockTokensData);
			const customAllLabel = 'Every token';

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					selectedTokens: allTokenAddresses,
					onChange,
					allLabel: customAllLabel
				}
			});

			expect(screen.getByText(customAllLabel)).toBeInTheDocument();
		});

		test('shows count when some tokens are selected', () => {
			const selectedAddresses = [mockTokensData[0].address, mockTokensData[1].address] as Address[];
			const tokensQuery = createMockTokensQuery(mockTokensData);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					selectedTokens: selectedAddresses,
					onChange
				}
			});

			expect(screen.getByText('2 tokens')).toBeInTheDocument();
		});

		test('shows singular form for single token selection', () => {
			const selectedAddresses = [mockTokensData[0].address] as Address[];
			const tokensQuery = createMockTokensQuery(mockTokensData);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					selectedTokens: selectedAddresses,
					onChange
				}
			});

			expect(screen.getByText('1 token')).toBeInTheDocument();
		});
	});

	describe('Token filtering', () => {
		test('calls onChange when token is selected', async () => {
			const tokensQuery = createMockTokensQuery(mockTokensData);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					selectedTokens: [],
					onChange
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			await waitFor(() => {
				const tokenDiv = screen.getByText('TOKEN1');
				expect(tokenDiv).toBeInTheDocument();
			});

			const tokenDiv = screen.getByText('TOKEN1');
			const checkbox = tokenDiv
				.closest('label')
				?.querySelector('input[type="checkbox"]') as HTMLInputElement;
			await fireEvent.click(checkbox);

			expect(onChange).toHaveBeenCalledWith(['0x1234567890123456789012345678901234567890']);
		});

		test('calls onChange when token is deselected', async () => {
			const selectedAddress = mockTokensData[0].address as Address;
			const tokensQuery = createMockTokensQuery(mockTokensData);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					selectedTokens: [selectedAddress],
					onChange
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			await waitFor(() => {
				const tokenDiv = screen.getByText('TOKEN1');
				const checkbox = tokenDiv
					.closest('label')
					?.querySelector('input[type="checkbox"]') as HTMLInputElement;
				expect(checkbox).toBeChecked();
			});

			const tokenDiv = screen.getByText('TOKEN1');
			const checkbox = tokenDiv
				.closest('label')
				?.querySelector('input[type="checkbox"]') as HTMLInputElement;
			await fireEvent.click(checkbox);

			expect(onChange).toHaveBeenCalledWith([]);
		});
	});

	describe('Search functionality', () => {
		test('filters tokens based on search term', async () => {
			const tokensQuery = createMockTokensQuery(mockTokensData);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					selectedTokens: [],
					onChange
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			const searchInput = screen.getByPlaceholderText('Search tokens...');
			await fireEvent.input(searchInput, { target: { value: 'ETH' } });

			await waitFor(() => {
				expect(screen.getByText('ETH')).toBeInTheDocument();
				expect(screen.queryByText('TOKEN1')).not.toBeInTheDocument();
			});
		});

		test('shows all tokens when search is cleared', async () => {
			const tokensQuery = createMockTokensQuery(mockTokensData);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					selectedTokens: [],
					onChange
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			const searchInput = screen.getByPlaceholderText('Search tokens...');
			await fireEvent.input(searchInput, { target: { value: 'ETH' } });

			await waitFor(() => {
				expect(screen.queryByLabelText('TOKEN1')).not.toBeInTheDocument();
			});

			await fireEvent.input(searchInput, { target: { value: '' } });

			await waitFor(() => {
				expect(screen.getByText('TOKEN1')).toBeInTheDocument();
				expect(screen.getByText('ETH')).toBeInTheDocument();
			});
		});
	});

	describe('Select all functionality', () => {
		test.skip('selects all tokens when "Select All" is clicked', async () => {
			const tokensQuery = createMockTokensQuery(mockTokensData);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					selectedTokens: [],
					onChange
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			await waitFor(() => {
				const selectAllCheckbox = screen.getByLabelText('All tokens');
				expect(selectAllCheckbox).toBeInTheDocument();
			});

			const selectAllCheckbox = screen.getByLabelText('All tokens');
			await fireEvent.click(selectAllCheckbox);

			const expectedAddresses = mockTokensData.map((token) => token.address);
			expect(onChange).toHaveBeenCalledWith(expectedAddresses);
		});

		test.skip('deselects all tokens when "Select All" is clicked and all are selected', async () => {
			const allTokenAddresses = mockTokensData.map((token) => token.address as Address);
			const tokensQuery = createMockTokensQuery(mockTokensData);

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					selectedTokens: allTokenAddresses,
					onChange
				}
			});

			await fireEvent.click(screen.getByTestId('dropdown-tokens-filter-button'));

			await waitFor(() => {
				const selectAllCheckbox = screen.getByLabelText('All tokens');
				expect(selectAllCheckbox).toBeChecked();
			});

			const selectAllCheckbox = screen.getByLabelText('All tokens');
			await fireEvent.click(selectAllCheckbox);

			expect(onChange).toHaveBeenCalledWith([]);
		});
	});

	describe('Custom labels', () => {
		test('uses custom label prop', () => {
			const tokensQuery = createMockTokensQuery(mockTokensData);
			const customLabel = 'Choose tokens';

			render(DropdownTokensFilter, {
				props: {
					tokensQuery,
					selectedTokens: [],
					onChange,
					label: customLabel
				}
			});

			expect(screen.getByText(customLabel)).toBeInTheDocument();
		});
	});
});
