import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import TokenSelectionModal from '../lib/components/deployment/TokenSelectionModal.svelte';
import type { ComponentProps } from 'svelte';
import type { TokenInfo, DotrainOrderGui } from '@rainlanguage/orderbook';
import { useGui } from '$lib/hooks/useGui';

type TokenSelectionModalProps = ComponentProps<TokenSelectionModal>;

const mockTokens: TokenInfo[] = [
	{
		key: 'token1',
		address: '0x1234567890123456789012345678901234567890',
		name: 'Test Token 1',
		symbol: 'TEST1',
		decimals: 18
	},
	{
		key: 'token2',
		address: '0x0987654321098765432109876543210987654321',
		name: 'Another Token',
		symbol: 'ANOTHER',
		decimals: 6
	}
];

const mockGui: DotrainOrderGui = {
	getAllTokens: vi.fn().mockResolvedValue({
		value: mockTokens
	})
} as unknown as DotrainOrderGui;

vi.mock('../lib/hooks/useGui', () => ({
	useGui: vi.fn()
}));

describe('TokenSelectionModal', () => {
	let mockOnSelect: ReturnType<typeof vi.fn>;

	const defaultProps: TokenSelectionModalProps = {
		selectedToken: null,
		onSelect: vi.fn()
	};

	beforeEach(() => {
		mockOnSelect = vi.fn();
		(useGui as Mock).mockReturnValue(mockGui);
		vi.clearAllMocks();
	});

	it('renders modal button with default text when no token is selected', () => {
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		expect(screen.getByText('Select a token...')).toBeInTheDocument();
	});

	it('renders modal button with selected token info when token is selected', () => {
		const selectedToken = mockTokens[0];
		render(TokenSelectionModal, {
			...defaultProps,
			selectedToken,
			onSelect: mockOnSelect
		});

		expect(screen.getByText('Test Token 1 (TEST1)')).toBeInTheDocument();
	});

	it('opens modal when button is clicked', async () => {
		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		expect(screen.getByText('Select a token')).toBeInTheDocument();
	});

	it('shows search input in modal', async () => {
		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		expect(screen.getByPlaceholderText('Search tokens or enter address (0x...)')).toBeInTheDocument();
	});

	it('loads tokens on mount', async () => {
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await userEvent.click(button);

		await waitFor(() => {
			expect(mockGui.getAllTokens).toHaveBeenCalledWith(undefined);
		});
	});

	it('shows tokens in modal after loading', async () => {
		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		await waitFor(() => {
			expect(screen.getByText('Test Token 1')).toBeInTheDocument();
			expect(screen.getByText('Another Token')).toBeInTheDocument();
		});
	});

	it('calls onSelect when token is clicked', async () => {
		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		await waitFor(() => {
			expect(screen.getByText('Test Token 1')).toBeInTheDocument();
		});

		const tokenItem = screen.getByText('Test Token 1');
		await user.click(tokenItem);

		expect(mockOnSelect).toHaveBeenCalledWith(mockTokens[0]);
	});

	it('calls API with search term when searching', async () => {
		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		const searchInput = screen.getByPlaceholderText('Search tokens or enter address (0x...)');
		await user.type(searchInput, 'TEST');

		await waitFor(() => {
			expect(mockGui.getAllTokens).toHaveBeenCalledWith('TEST');
		});
	});

	it('shows loading state while searching', async () => {
		const mockGuiWithDelay = {
			getAllTokens: vi
				.fn()
				.mockImplementation(
					() => new Promise((resolve) => setTimeout(() => resolve({ value: mockTokens }), 100))
				)
		} as unknown as DotrainOrderGui;

		(useGui as Mock).mockReturnValue(mockGuiWithDelay);

		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		const searchInput = screen.getByPlaceholderText('Search tokens or enter address (0x...)');
		await user.type(searchInput, 'TEST');

		expect(screen.getByText('Searching tokens...')).toBeInTheDocument();
	});

	it('shows no results message when search returns empty', async () => {
		const mockGuiNoResults = {
			getAllTokens: vi.fn().mockResolvedValue({ value: [] })
		} as unknown as DotrainOrderGui;

		(useGui as Mock).mockReturnValue(mockGuiNoResults);

		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		await waitFor(() => {
			expect(screen.getByText('No tokens found matching your search.')).toBeInTheDocument();
		});
	});

	it('clears search when clear button is clicked', async () => {
		const mockGuiNoResults = {
			getAllTokens: vi
				.fn()
				.mockResolvedValueOnce({ value: [] })
				.mockResolvedValueOnce({ value: mockTokens })
		} as unknown as DotrainOrderGui;

		(useGui as Mock).mockReturnValue(mockGuiNoResults);

		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		await waitFor(() => {
			expect(screen.getByText('No tokens found matching your search.')).toBeInTheDocument();
		});

		const clearButton = screen.getByText('Clear search');
		await user.click(clearButton);

		await waitFor(() => {
			expect(mockGuiNoResults.getAllTokens).toHaveBeenCalledWith(undefined);
		});
	});

	it('automatically focuses search input when modal opens', async () => {
		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		await waitFor(() => {
			const searchInput = screen.getByPlaceholderText('Search tokens or enter address (0x...)');
			expect(searchInput).toHaveFocus();
		});
	});

	it('validates custom token address when entered', async () => {
		const mockGuiWithCustomToken = {
			getAllTokens: vi.fn().mockResolvedValue({ value: [] }),
			setSelectToken: vi.fn().mockResolvedValue(undefined),
			getTokenInfo: vi.fn().mockResolvedValue({
				value: {
					address: '0x1234567890123456789012345678901234567890',
					name: 'Custom Token',
					symbol: 'CUSTOM',
					decimals: 18
				}
			}),
			unsetSelectToken: vi.fn().mockResolvedValue(undefined)
		} as unknown as DotrainOrderGui;

		(useGui as Mock).mockReturnValue(mockGuiWithCustomToken);

		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		const searchInput = screen.getByPlaceholderText('Search tokens or enter address (0x...)');
		await user.type(searchInput, '0x1234567890123456789012345678901234567890');

		await waitFor(() => {
			expect(screen.getByText('Custom Token')).toBeInTheDocument();
			expect(screen.getByText('Custom token (not in list)')).toBeInTheDocument();
		});
	});

	it('shows error for invalid custom token address', async () => {
		const mockGuiWithError = {
			getAllTokens: vi.fn().mockResolvedValue({ value: [] }),
			setSelectToken: vi.fn().mockRejectedValue(new Error('Invalid token address')),
			unsetSelectToken: vi.fn().mockResolvedValue(undefined)
		} as unknown as DotrainOrderGui;

		(useGui as Mock).mockReturnValue(mockGuiWithError);

		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		const searchInput = screen.getByPlaceholderText('Search tokens or enter address (0x...)');
		await user.type(searchInput, '0x1234567890123456789012345678901234567890');

		await waitFor(() => {
			expect(screen.getByText('Invalid token address')).toBeInTheDocument();
		});
	});

	it('allows selection of custom token', async () => {
		const mockGuiWithCustomToken = {
			getAllTokens: vi.fn().mockResolvedValue({ value: [] }),
			setSelectToken: vi.fn().mockResolvedValue(undefined),
			getTokenInfo: vi.fn().mockResolvedValue({
				value: {
					address: '0x1234567890123456789012345678901234567890',
					name: 'Custom Token',
					symbol: 'CUSTOM',
					decimals: 18
				}
			}),
			unsetSelectToken: vi.fn().mockResolvedValue(undefined)
		} as unknown as DotrainOrderGui;

		(useGui as Mock).mockReturnValue(mockGuiWithCustomToken);

		const user = userEvent.setup();
		render(TokenSelectionModal, {
			...defaultProps,
			onSelect: mockOnSelect
		});

		const button = screen.getByRole('button');
		await user.click(button);

		const searchInput = screen.getByPlaceholderText('Search tokens or enter address (0x...)');
		await user.type(searchInput, '0x1234567890123456789012345678901234567890');

		await waitFor(() => {
			expect(screen.getByText('Custom Token')).toBeInTheDocument();
		});

		const customTokenItem = screen.getByText('Custom Token');
		await user.click(customTokenItem);

		expect(mockOnSelect).toHaveBeenCalledWith({
			key: 'temp-validation-token',
			address: '0x1234567890123456789012345678901234567890',
			name: 'Custom Token',
			symbol: 'CUSTOM',
			decimals: 18
		});
	});
});
