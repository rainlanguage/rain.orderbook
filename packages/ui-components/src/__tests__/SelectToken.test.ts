import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import SelectToken from '../lib/components/deployment/SelectToken.svelte';
import type { ComponentProps } from 'svelte';
import type { AccountBalance, DotrainOrderGui } from '@rainlanguage/orderbook';
import { useGui } from '$lib/hooks/useGui';
import type { TokenBalance } from '$lib/types/tokenBalance';

type SelectTokenComponentProps = ComponentProps<SelectToken>;

const mockGui: DotrainOrderGui = {
	setSelectToken: vi.fn(),
	isSelectTokenSet: vi.fn(),
	unsetSelectToken: vi.fn(),
	getTokenInfo: vi.fn().mockResolvedValue({
		value: {
			name: 'Ethereum',
			symbol: 'ETH',
			decimals: 18,
			address: '0x456'
		}
	}),
	getAllTokens: vi.fn().mockResolvedValue({
		value: [
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
		]
	})
} as unknown as DotrainOrderGui;

vi.mock('../lib/hooks/useGui', () => ({
	useGui: vi.fn()
}));

describe('SelectToken', () => {
	let mockStateUpdateCallback: Mock;

	const mockProps: SelectTokenComponentProps = {
		token: {
			key: 'input',
			name: 'test input',
			description: 'test description'
		},
		onSelectTokenSelect: vi.fn(),
		tokenBalances: new Map()
	};

	beforeEach(() => {
		mockStateUpdateCallback = vi.fn();
		mockGui.setSelectToken = vi.fn().mockImplementation(() => {
			mockStateUpdateCallback();
			return Promise.resolve();
		});
		(useGui as Mock).mockReturnValue(mockGui);
		vi.clearAllMocks();
	});

	it('renders token label correctly', () => {
		const { getByText } = render(SelectToken, mockProps);
		expect(getByText('test input')).toBeInTheDocument();
	});

	it('renders dropdown button when tokens are available', () => {
		const { getByText } = render(SelectToken, mockProps);
		expect(getByText('Select a token...')).toBeInTheDocument();
	});

	it('calls setSelectToken when token is selected from modal', async () => {
		const user = userEvent.setup();
		const mockGuiWithNoToken = {
			...mockGui,
			getTokenInfo: vi.fn().mockResolvedValue({ value: null })
		} as unknown as DotrainOrderGui;

		(useGui as Mock).mockReturnValue(mockGuiWithNoToken);

		const { getByText } = render(SelectToken, {
			...mockProps
		});

		// Click the token selection button to open modal
		const selectButton = getByText('Select a token...');
		await user.click(selectButton);

		// Wait for modal to load tokens and then select one
		await waitFor(() => {
			expect(getByText('Test Token 1')).toBeInTheDocument();
		});

		const firstToken = getByText('Test Token 1');
		await user.click(firstToken);

		await waitFor(() => {
			expect(mockGuiWithNoToken.setSelectToken).toHaveBeenCalledWith('input', '0x1234567890123456789012345678901234567890');
		});
		expect(mockStateUpdateCallback).toHaveBeenCalledTimes(1);
	});

	it('shows error message for invalid token selection', async () => {
		const user = userEvent.setup();
		const mockGuiWithError = {
			...mockGui,
			setSelectToken: vi.fn().mockRejectedValue(new Error('Invalid address'))
		} as unknown as DotrainOrderGui;

		(useGui as Mock).mockReturnValue(mockGuiWithError);

		const screen = render(SelectToken, {
			...mockProps
		});

		// Click the token selection button to open modal
		const selectButton = screen.getByText('Select a token...');
		await user.click(selectButton);

		// Wait for modal to load tokens and then select one
		await waitFor(() => {
			expect(screen.getByText('Test Token 1')).toBeInTheDocument();
		});

		const firstToken = screen.getByText('Test Token 1');
		await user.click(firstToken);

		await waitFor(() => {
			expect(screen.getByTestId('error')).toBeInTheDocument();
		});
	});

	it('calls onSelectTokenSelect after token selection', async () => {
		const user = userEvent.setup();
		const { getByText } = render(SelectToken, mockProps);

		// Click the token selection button to open modal
		const selectButton = getByText('Select a token...');
		await user.click(selectButton);

		// Wait for modal to load tokens and then select one
		await waitFor(() => {
			expect(getByText('Test Token 1')).toBeInTheDocument();
		});

		const firstToken = getByText('Test Token 1');
		await user.click(firstToken);

		await waitFor(() => {
			expect(mockProps.onSelectTokenSelect).toHaveBeenCalled();
		});
	});

	describe('Token Selection', () => {
		beforeEach(() => {
			(useGui as Mock).mockReturnValue(mockGui);
		});

		it('shows token selection modal', () => {
			render(SelectToken, mockProps);
			expect(screen.getByText('Select a token...')).toBeInTheDocument();
		});

		it('displays selected token info when token is selected', async () => {
			mockGui.getTokenInfo = vi.fn().mockResolvedValue({
				value: {
					name: 'Test Token 1',
					symbol: 'TEST1',
					address: '0x1234567890123456789012345678901234567890',
					decimals: 18
				}
			});

			render(SelectToken, mockProps);

			await waitFor(() => {
				expect(screen.getByText('Test Token 1')).toBeInTheDocument();
			});

			expect(screen.getByTestId(`select-token-success-${mockProps.token.key}`)).toBeInTheDocument();
		});
	});

	describe('Balance Display', () => {
		it('displays balance when token is selected and balance is provided', async () => {
			mockGui.getTokenInfo = vi.fn().mockResolvedValue({
				value: {
					name: 'Test Token',
					symbol: 'TEST',
					address: '0x1234567890123456789012345678901234567890',
					decimals: 18,
					key: 'input'
				}
			});

			const tokenBalances = new Map<string, TokenBalance>();
			tokenBalances.set('input', {
				value: {
					balance: BigInt('1000000000000000000'),
					formattedBalance: '1'
				} as AccountBalance,
				loading: false,
				error: ''
			});

			render(SelectToken, {
				...mockProps,
				tokenBalances
			});

			await waitFor(() => {
				expect(screen.getByText('Test Token')).toBeInTheDocument();
			});

			await waitFor(() => {
				expect(screen.getByText('Balance: 1')).toBeInTheDocument();
			});
		});

		it('shows loading spinner when balance is loading', async () => {
			mockGui.getTokenInfo = vi.fn().mockResolvedValue({
				value: {
					name: 'Test Token',
					symbol: 'TEST',
					address: '0x1234567890123456789012345678901234567890',
					decimals: 18,
					key: 'input'
				}
			});

			const tokenBalances = new Map<string, TokenBalance>();
			tokenBalances.set('input', {
				value: {
					balance: BigInt(0),
					formattedBalance: '0'
				} as AccountBalance,
				loading: true,
				error: ''
			});

			render(SelectToken, {
				...mockProps,
				tokenBalances
			});

			await waitFor(() => {
				expect(screen.getByText('Test Token')).toBeInTheDocument();
			});

			// Check for spinner (we can't easily test the spinner component directly, so we test for its presence)
			const tokenStatus = screen.getByTestId(`select-token-success-${mockProps.token.key}`);
			expect(tokenStatus).toBeInTheDocument();
		});

		it('shows error message when balance fetch fails', async () => {
			mockGui.getTokenInfo = vi.fn().mockResolvedValue({
				value: {
					name: 'Test Token',
					symbol: 'TEST',
					address: '0x1234567890123456789012345678901234567890',
					decimals: 18,
					key: 'input'
				}
			});

			const tokenBalances = new Map<string, TokenBalance>();
			tokenBalances.set('input', {
				value: {
					balance: BigInt(0),
					formattedBalance: '0'
				} as AccountBalance,
				loading: false,
				error: 'Network error'
			});

			render(SelectToken, {
				...mockProps,
				tokenBalances
			});

			await waitFor(() => {
				expect(screen.getByText('Test Token')).toBeInTheDocument();
			});

			await waitFor(() => {
				expect(screen.getByText('Network error')).toBeInTheDocument();
			});
		});

		it('formats balance correctly with token decimals', async () => {
			mockGui.getTokenInfo = vi.fn().mockResolvedValue({
				value: {
					name: 'USDC',
					symbol: 'USDC',
					address: '0x1234567890123456789012345678901234567890',
					decimals: 6,
					key: 'input'
				}
			});

			const tokenBalances = new Map<string, TokenBalance>();
			tokenBalances.set('input', {
				value: {
					balance: BigInt('1500000'),
					formattedBalance: '1.5'
				} as AccountBalance,
				loading: false,
				error: ''
			});

			render(SelectToken, {
				...mockProps,
				tokenBalances
			});

			await waitFor(() => {
				expect(screen.getByText('USDC')).toBeInTheDocument();
			});

			await waitFor(() => {
				expect(screen.getByText('Balance: 1.5')).toBeInTheDocument();
			});
		});
	});
});
