import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import SelectToken from '../lib/components/deployment/SelectToken.svelte';
import type { ComponentProps } from 'svelte';
import type { DotrainOrderGui } from '@rainlanguage/orderbook';
import { useGui } from '$lib/hooks/useGui';
import { readable } from 'svelte/store';

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
	})
} as unknown as DotrainOrderGui;

vi.mock('../lib/hooks/useGui', () => ({
	useGui: vi.fn()
}));

describe('SelectToken', () => {
	let mockStateUpdateCallback: Mock;

	const mockTokens = [
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
		}
	];

	const mockProps: SelectTokenComponentProps = {
		token: {
			key: 'input',
			name: 'test input',
			description: 'test description'
		},
		onSelectTokenSelect: vi.fn(),
		availableTokens: mockTokens,
		loading: false,
		account: readable('0x1234567890123456789012345678901234567890')
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

	it('calls setSelectToken and updates token info when input changes', async () => {
		const user = userEvent.setup();
		const mockGuiWithNoToken = {
			...mockGui,
			getTokenInfo: vi.fn().mockResolvedValue({ value: null })
		} as unknown as DotrainOrderGui;

		(useGui as Mock).mockReturnValue(mockGuiWithNoToken);

		const { getByTestId, getByRole } = render(SelectToken, {
			...mockProps
		});

		const customButton = getByTestId('custom-mode-button');
		await user.click(customButton);

		const input = getByRole('textbox');

		await userEvent.clear(input);
		await user.paste('0x456');

		await waitFor(() => {
			expect(mockGuiWithNoToken.setSelectToken).toHaveBeenCalledWith('input', '0x456');
		});
		expect(mockStateUpdateCallback).toHaveBeenCalledTimes(1);
	});

	it('shows error message for invalid address, and removes the selectToken', async () => {
		const user = userEvent.setup();
		const mockGuiWithError = {
			...mockGui,
			setSelectToken: vi.fn().mockRejectedValue(new Error('Invalid address'))
		} as unknown as DotrainOrderGui;

		(useGui as Mock).mockReturnValue(mockGuiWithError);

		const screen = render(SelectToken, {
			...mockProps
		});

		const customButton = screen.getByTestId('custom-mode-button');
		await user.click(customButton);

		const input = screen.getByRole('textbox');
		await userEvent.clear(input);
		await user.paste('invalid');
		await waitFor(() => {
			expect(screen.getByTestId('error')).toBeInTheDocument();
		});
	});

	it('does nothing if gui is not defined', async () => {
		const user = userEvent.setup();
		(useGui as Mock).mockReturnValue(null);

		const { queryByRole } = render(SelectToken, {
			...mockProps,
			availableTokens: []
		});

		const input = queryByRole('textbox');
		if (input) {
			await userEvent.clear(input);
			await user.paste('0x456');
		}

		await waitFor(() => {
			expect(mockGui.setSelectToken).not.toHaveBeenCalled();
		});
	});

	it('replaces the token and triggers state update twice if the token is already set', async () => {
		const mockGuiWithTokenSet = {
			...mockGui,
			isSelectTokenSet: vi.fn().mockResolvedValue(true)
		} as unknown as DotrainOrderGui;

		(useGui as Mock).mockReturnValue(mockGuiWithTokenSet);

		const user = userEvent.setup();

		const { getByRole, getByTestId } = render(SelectToken, {
			...mockProps
		});

		const customButton = getByTestId('custom-mode-button');
		await user.click(customButton);

		const input = getByRole('textbox');
		await userEvent.clear(input);
		await user.paste('invalid');
		await waitFor(() => {
			expect(mockGuiWithTokenSet.setSelectToken).toHaveBeenCalled();
			expect(mockStateUpdateCallback).toHaveBeenCalledTimes(1);
		});
	});

	it('calls onSelectTokenSelect after input changes', async () => {
		const user = userEvent.setup();
		const { getByRole, getByTestId } = render(SelectToken, mockProps);

		const customButton = getByTestId('custom-mode-button');
		await user.click(customButton);

		const input = getByRole('textbox');

		await userEvent.clear(input);
		await user.paste('0x456');

		await waitFor(() => {
			expect(mockProps.onSelectTokenSelect).toHaveBeenCalled();
		});
	});

	it('switches to custom mode automatically if selected token is not in available tokens', async () => {
		mockGui.getTokenInfo = vi.fn().mockResolvedValue({
			value: {
				name: 'Custom Token',
				symbol: 'CUSTOM',
				address: '0xCustomTokenAddress',
				decimals: 18
			}
		});

		render(SelectToken, mockProps);

		await waitFor(() => {
			expect(screen.queryByText('Select a token...')).not.toBeInTheDocument();
			expect(screen.getByPlaceholderText('Enter token address (0x...)')).toBeInTheDocument();
		});
	});

	describe('Dropdown Mode', () => {
		beforeEach(() => {
			(useGui as Mock).mockReturnValue(mockGui);
		});

		it('shows dropdown and custom mode buttons when tokens are available', () => {
			render(SelectToken, mockProps);

			expect(screen.getByTestId('dropdown-mode-button')).toBeInTheDocument();
			expect(screen.getByTestId('custom-mode-button')).toBeInTheDocument();
		});

		it('shows dropdown mode as active by default', () => {
			render(SelectToken, mockProps);

			const dropdownButton = screen.getByTestId('dropdown-mode-button');
			const customButton = screen.getByTestId('custom-mode-button');

			expect(dropdownButton).toHaveClass('border-blue-300');
			expect(customButton).not.toHaveClass('border-blue-300');
		});

		it('switches to custom mode when custom button is clicked', async () => {
			const user = userEvent.setup();
			render(SelectToken, mockProps);

			const customButton = screen.getByTestId('custom-mode-button');
			await user.click(customButton);

			expect(customButton).toHaveClass('border-blue-300');
			expect(screen.getByTestId('dropdown-mode-button')).not.toHaveClass('border-blue-300');
		});

		it('shows TokenSelectionModal component in dropdown mode', () => {
			render(SelectToken, mockProps);

			expect(screen.getByText('Select a token...')).toBeInTheDocument();
		});

		it('shows custom input in custom mode', async () => {
			const user = userEvent.setup();
			render(SelectToken, mockProps);

			const customButton = screen.getByTestId('custom-mode-button');
			await user.click(customButton);

			expect(screen.getByPlaceholderText('Enter token address (0x...)')).toBeInTheDocument();
		});

		it('clears state when switching from dropdown to custom mode', async () => {
			const user = userEvent.setup();
			const mockGuiNoToken = {
				...mockGui,
				getTokenInfo: vi.fn().mockResolvedValue({ value: null })
			} as unknown as DotrainOrderGui;

			(useGui as Mock).mockReturnValue(mockGuiNoToken);

			render(SelectToken, {
				...mockProps,
				availableTokens: [
					{
						address: '0x456',
						name: 'Test Token 1',
						symbol: 'TEST1',
						decimals: 18
					}
				]
			});

			const dropdownButton = screen.getByText('Select a token...');
			await user.click(dropdownButton);

			const firstToken = screen.getByText('Test Token 1');
			await user.click(firstToken);

			const customButton = screen.getByTestId('custom-mode-button');
			await user.click(customButton);

			const customInput = screen.getByPlaceholderText('Enter token address (0x...)');
			expect(customInput).toHaveValue('');

			expect(mockGuiNoToken.unsetSelectToken).toHaveBeenCalledWith('input');
		});

		it('clears state when switching from custom to dropdown mode', async () => {
			const user = userEvent.setup();
			render(SelectToken, mockProps);

			const customButton = screen.getByTestId('custom-mode-button');
			await user.click(customButton);

			const customInput = screen.getByPlaceholderText('Enter token address (0x...)');
			await user.type(customInput, '0x1234567890123456789012345678901234567890');

			const dropdownButton = screen.getByTestId('dropdown-mode-button');
			await user.click(dropdownButton);

			expect(mockGui.unsetSelectToken).toHaveBeenCalledWith('input');
		});

		it('handles token selection from dropdown', async () => {
			const user = userEvent.setup();
			const mockGuiNoToken = {
				...mockGui,
				getTokenInfo: vi.fn().mockResolvedValue({ value: null })
			} as unknown as DotrainOrderGui;

			(useGui as Mock).mockReturnValue(mockGuiNoToken);

			render(SelectToken, {
				...mockProps,
				availableTokens: [
					{
						address: '0x456',
						name: 'Test Token 1',
						symbol: 'TEST1',
						decimals: 18
					},
					{
						address: '0x789',
						name: 'Test Token 2',
						symbol: 'TEST2',
						decimals: 18
					}
				]
			});

			const dropdownButton = screen.getByText('Select a token...');
			await user.click(dropdownButton);

			const secondToken = screen.getByText('Test Token 2');
			await user.click(secondToken);

			expect(mockGuiNoToken.setSelectToken).toHaveBeenCalledWith('input', '0x789');
		});

		it('shows loading state when tokens are loading', () => {
			render(SelectToken, {
				...mockProps,
				loading: true
			});

			expect(screen.getByText('Loading tokens...')).toBeInTheDocument();
		});

		it('defaults to custom mode when no tokens are available', () => {
			render(SelectToken, {
				...mockProps,
				availableTokens: []
			});

			expect(screen.getByPlaceholderText('Enter token address (0x...)')).toBeInTheDocument();
			expect(screen.queryByTestId('dropdown-mode-button')).not.toBeInTheDocument();
			expect(screen.queryByTestId('custom-mode-button')).not.toBeInTheDocument();
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
		it('displays balance when token is selected and account is connected', async () => {
			mockGui.getTokenInfo = vi.fn().mockResolvedValue({
				value: {
					name: 'Test Token',
					symbol: 'TEST',
					address: '0x1234567890123456789012345678901234567890',
					decimals: 18
				}
			});
			mockGui.getTokenBalance = vi.fn().mockResolvedValue({
				value: '1000000000000000000' // 1 TEST token
			});

			render(SelectToken, mockProps);

			await waitFor(() => {
				expect(screen.getByText('Test Token')).toBeInTheDocument();
			});

			await waitFor(() => {
				expect(screen.getByText('Balance: 1')).toBeInTheDocument();
			});
		});

		it('shows error message when balance fetch fails', async () => {
			mockGui.getTokenInfo = vi.fn().mockResolvedValue({
				value: {
					name: 'Test Token',
					symbol: 'TEST',
					address: '0x1234567890123456789012345678901234567890',
					decimals: 18
				}
			});

			(mockGui.getTokenBalance as Mock).mockRejectedValue(new Error('Network error'));

			render(SelectToken, mockProps);

			await waitFor(() => {
				expect(screen.getByText('Test Token')).toBeInTheDocument();
			});

			await waitFor(() => {
				expect(screen.getByText('Failed to fetch balance')).toBeInTheDocument();
			});
		});

		it('does not display balance when account is not connected', async () => {
			mockGui.getTokenInfo = vi.fn().mockResolvedValue({
				value: {
					name: 'Test Token',
					symbol: 'TEST',
					address: '0x1234567890123456789012345678901234567890',
					decimals: 18
				}
			});

			render(SelectToken, {
				...mockProps,
				account: readable(null) // Simulate disconnected account
			});

			await waitFor(() => {
				expect(screen.getByText('Test Token')).toBeInTheDocument();
			});

			expect(screen.queryByText(/Balance:/)).not.toBeInTheDocument();
		});

		it('formats balance correctly with token decimals', async () => {
			mockGui.getTokenInfo = vi.fn().mockResolvedValue({
				value: {
					name: 'USDC',
					symbol: 'USDC',
					address: '0x1234567890123456789012345678901234567890',
					decimals: 6
				}
			});

			mockGui.getTokenBalance = vi.fn().mockResolvedValue({ value: '1500000' }); // 1.5 USDC

			render(SelectToken, mockProps);

			await waitFor(() => {
				expect(screen.getByText('USDC')).toBeInTheDocument();
			});

			await waitFor(() => {
				expect(screen.getByText('Balance: 1.5')).toBeInTheDocument();
			});
		});
	});
});
