import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import TokenDropdown from '../lib/components/deployment/TokenDropdown.svelte';
import type { ComponentProps } from 'svelte';
import type { TokenInfo } from '@rainlanguage/orderbook';

type TokenDropdownProps = ComponentProps<TokenDropdown>;

const mockTokens: TokenInfo[] = [
	{
		address: '0x1234567890123456789012345678901234567890',
		name: 'Test Token 1',
		symbol: 'TEST1',
		decimals: 18
	},
	{
		address: '0x0987654321098765432109876543210987654321',
		name: 'Another Token',
		symbol: 'ANOTHER',
		decimals: 6
	},
	{
		address: '0x1111222233334444555566667777888899990000',
		name: 'Third Token',
		symbol: 'THIRD',
		decimals: 18
	}
];

describe('TokenDropdown', () => {
	let mockOnSelect: ReturnType<typeof vi.fn>;
	let mockOnSearch: ReturnType<typeof vi.fn>;

	const defaultProps: TokenDropdownProps = {
		tokens: mockTokens,
		selectedToken: null,
		onSelect: vi.fn(),
		searchValue: '',
		onSearch: vi.fn()
	};

	beforeEach(() => {
		mockOnSelect = vi.fn();
		mockOnSearch = vi.fn();
		vi.clearAllMocks();
	});

	it('renders dropdown button with default text when no token is selected', () => {
		render(TokenDropdown, {
			...defaultProps,
			onSelect: mockOnSelect,
			onSearch: mockOnSearch
		});

		expect(screen.getByText('Select a token...')).toBeInTheDocument();
	});

	it('renders dropdown button with selected token info when token is selected', () => {
		const selectedToken = mockTokens[0];
		render(TokenDropdown, {
			...defaultProps,
			selectedToken,
			onSelect: mockOnSelect,
			onSearch: mockOnSearch
		});

		expect(screen.getByText('Test Token 1 (TEST1)')).toBeInTheDocument();
	});

	it('opens dropdown when button is clicked', async () => {
		const user = userEvent.setup();
		render(TokenDropdown, {
			...defaultProps,
			onSelect: mockOnSelect,
			onSearch: mockOnSearch
		});

		const button = screen.getByRole('button');
		await user.click(button);

		expect(screen.getByPlaceholderText('Search tokens...')).toBeInTheDocument();
	});

	it('displays all tokens in the dropdown list', async () => {
		const user = userEvent.setup();
		render(TokenDropdown, {
			...defaultProps,
			onSelect: mockOnSelect,
			onSearch: mockOnSearch
		});

		const button = screen.getByRole('button');
		await user.click(button);

		expect(screen.getByText('Test Token 1')).toBeInTheDocument();
		expect(screen.getByText('TEST1')).toBeInTheDocument();
		expect(screen.getByText('Another Token')).toBeInTheDocument();
		expect(screen.getByText('ANOTHER')).toBeInTheDocument();
		expect(screen.getByText('Third Token')).toBeInTheDocument();
		expect(screen.getByText('THIRD')).toBeInTheDocument();
	});

	it('displays formatted addresses in token list', async () => {
		const user = userEvent.setup();
		render(TokenDropdown, {
			...defaultProps,
			onSelect: mockOnSelect,
			onSearch: mockOnSearch
		});

		const button = screen.getByRole('button');
		await user.click(button);

		expect(screen.getByText('0x1234...7890')).toBeInTheDocument();
		expect(screen.getByText('0x0987...4321')).toBeInTheDocument();
		expect(screen.getByText('0x1111...0000')).toBeInTheDocument();
	});

	it('highlights selected token in the list', async () => {
		const user = userEvent.setup();
		const selectedToken = mockTokens[1];
		render(TokenDropdown, {
			...defaultProps,
			selectedToken,
			onSelect: mockOnSelect,
			onSearch: mockOnSearch
		});

		const button = screen.getByRole('button');
		await user.click(button);

		const selectedTokenItem = screen.getByText('Another Token').closest('[role="button"]');
		expect(selectedTokenItem).toHaveClass('bg-blue-50');

		expect(screen.getByRole('img', { name: /check circle solid/i })).toBeInTheDocument();
	});

	it('calls onSelect when token is clicked', async () => {
		const user = userEvent.setup();
		render(TokenDropdown, {
			...defaultProps,
			onSelect: mockOnSelect,
			onSearch: mockOnSearch
		});

		const button = screen.getByRole('button');
		await user.click(button);

		const firstToken = screen.getByText('Test Token 1').closest('[role="button"]');
		expect(firstToken).toBeInTheDocument();

		if (firstToken) {
			await user.click(firstToken);
		}

		expect(mockOnSelect).toHaveBeenCalledWith(mockTokens[0]);
	});

	it('closes dropdown after token selection', async () => {
		const user = userEvent.setup();
		render(TokenDropdown, {
			...defaultProps,
			onSelect: mockOnSelect,
			onSearch: mockOnSearch
		});

		const button = screen.getByRole('button');
		await user.click(button);

		expect(screen.getByPlaceholderText('Search tokens...')).toBeInTheDocument();

		const firstToken = screen.getByText('Test Token 1').closest('[role="button"]');
		if (firstToken) {
			await user.click(firstToken);
		}

		await waitFor(() => {
			expect(screen.queryByPlaceholderText('Search tokens...')).not.toBeInTheDocument();
		});
	});

	it('filters tokens based on search input', async () => {
		const user = userEvent.setup();
		render(TokenDropdown, {
			...defaultProps,
			searchValue: 'test',
			onSelect: mockOnSelect,
			onSearch: mockOnSearch
		});

		const button = screen.getByRole('button');
		await user.click(button);

		expect(screen.getByText('Test Token 1')).toBeInTheDocument();
		expect(screen.queryByText('Another Token')).not.toBeInTheDocument();
		expect(screen.queryByText('Third Token')).not.toBeInTheDocument();
	});

	it('calls onSearch when search input changes', async () => {
		const user = userEvent.setup();
		render(TokenDropdown, {
			...defaultProps,
			onSelect: mockOnSelect,
			onSearch: mockOnSearch
		});

		const button = screen.getByRole('button');
		await user.click(button);

		const searchInput = screen.getByPlaceholderText('Search tokens...');
		await user.type(searchInput, 'another');

		expect(mockOnSearch).toHaveBeenCalledWith('another');
	});

	it('filters tokens by symbol', async () => {
		const user = userEvent.setup();
		render(TokenDropdown, {
			...defaultProps,
			searchValue: 'ANOTHER',
			onSelect: mockOnSelect,
			onSearch: mockOnSearch
		});

		const button = screen.getByRole('button');
		await user.click(button);

		expect(screen.getByText('Another Token')).toBeInTheDocument();
		expect(screen.queryByText('Test Token 1')).not.toBeInTheDocument();
		expect(screen.queryByText('Third Token')).not.toBeInTheDocument();
	});

	it('filters tokens by address', async () => {
		const user = userEvent.setup();
		render(TokenDropdown, {
			...defaultProps,
			searchValue: '0x1234',
			onSelect: mockOnSelect,
			onSearch: mockOnSearch
		});

		const button = screen.getByRole('button');
		await user.click(button);

		expect(screen.getByText('Test Token 1')).toBeInTheDocument();
		expect(screen.queryByText('Another Token')).not.toBeInTheDocument();
		expect(screen.queryByText('Third Token')).not.toBeInTheDocument();
	});

	it('shows "no results" message when no tokens match search', async () => {
		const user = userEvent.setup();
		render(TokenDropdown, {
			...defaultProps,
			searchValue: 'nonexistent',
			onSelect: mockOnSelect,
			onSearch: mockOnSearch
		});

		const button = screen.getByRole('button');
		await user.click(button);

		expect(screen.getByText('No tokens found matching your search.')).toBeInTheDocument();
		expect(screen.getByText('Clear search')).toBeInTheDocument();
	});

	it('clears search when "Clear search" button is clicked', async () => {
		const user = userEvent.setup();
		render(TokenDropdown, {
			...defaultProps,
			searchValue: 'nonexistent',
			onSelect: mockOnSelect,
			onSearch: mockOnSearch
		});

		const button = screen.getByRole('button');
		await user.click(button);

		const clearButton = screen.getByText('Clear search');
		await user.click(clearButton);

		expect(mockOnSearch).toHaveBeenCalledWith('');
	});

	it('handles token selection via keyboard (Enter key)', async () => {
		const user = userEvent.setup();
		render(TokenDropdown, {
			...defaultProps,
			onSelect: mockOnSelect,
			onSearch: mockOnSearch
		});

		const button = screen.getByRole('button');
		await user.click(button);

		const firstToken = screen.getByText('Test Token 1').closest('[role="button"]') as HTMLElement;
		if (firstToken) {
			firstToken.focus();
			await user.keyboard('{Enter}');
		}

		expect(mockOnSelect).toHaveBeenCalledWith(mockTokens[0]);
	});

	it('displays empty state when no tokens are provided', async () => {
		const user = userEvent.setup();
		render(TokenDropdown, {
			...defaultProps,
			tokens: [],
			onSelect: mockOnSelect,
			onSearch: mockOnSearch
		});

		const button = screen.getByRole('button');
		await user.click(button);

		expect(screen.getByText('No tokens found matching your search.')).toBeInTheDocument();
	});

	it('maintains search value in input field', async () => {
		const user = userEvent.setup();
		render(TokenDropdown, {
			...defaultProps,
			searchValue: 'initial search',
			onSelect: mockOnSelect,
			onSearch: mockOnSearch
		});

		const button = screen.getByRole('button');
		await user.click(button);

		const searchInput = screen.getByPlaceholderText('Search tokens...') as HTMLInputElement;
		expect(searchInput.value).toBe('initial search');
	});

	it('search is case insensitive', async () => {
		const user = userEvent.setup();
		render(TokenDropdown, {
			...defaultProps,
			searchValue: 'TEST',
			onSelect: mockOnSelect,
			onSearch: mockOnSearch
		});

		const button = screen.getByRole('button');
		await user.click(button);

		expect(screen.getByText('Test Token 1')).toBeInTheDocument();
		expect(screen.queryByText('Another Token')).not.toBeInTheDocument();
	});
});
