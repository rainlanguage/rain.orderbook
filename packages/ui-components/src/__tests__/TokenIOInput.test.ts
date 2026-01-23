import { render, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import TokenIOInput from '../lib/components/deployment/TokenIOInput.svelte';
import type { ComponentProps } from 'svelte';
import { RaindexAmount, DotrainOrderGui, Float } from '@rainlanguage/orderbook';
import { useGui } from '$lib/hooks/useGui';
import type { TokenBalance } from '$lib/types/tokenBalance';

vi.mock('@rainlanguage/orderbook', async (importOriginal) => {
	return {
		...(await importOriginal()),
		DotrainOrderGui: vi.fn()
	};
});

vi.mock('$lib/hooks/useGui', () => ({
	useGui: vi.fn()
}));

type TokenIOInputComponentProps = ComponentProps<TokenIOInput>;

describe('TokenInput', () => {
	let guiInstance: DotrainOrderGui;
	let mockStateUpdateCallback: Mock;
	let mockProps: TokenIOInputComponentProps;
	let outputMockProps: TokenIOInputComponentProps;

	const mockInput = {
		token: {
			address: '0x123',
			key: 'test'
		}
	};
	const mockTokenInfo = {
		value: {
			symbol: 'MOCK',
			name: 'Mock Token',
			decimals: 18
		}
	};

	beforeEach(() => {
		vi.clearAllMocks();

		// Create a mock instance with all the methods
		guiInstance = {
			getTokenInfo: vi.fn().mockResolvedValue(mockTokenInfo),
			setVaultId: vi.fn().mockImplementation(() => {
				mockStateUpdateCallback();
			}),
			setVaultless: vi.fn().mockReturnValue({ error: null }),
			getCurrentDeployment: vi.fn().mockResolvedValue({
				deployment: {
					order: {
						inputs: [mockInput]
					}
				}
			}),
			getVaultIds: vi.fn().mockReturnValue({
				value: new Map([
					['input', new Map([['test', 'vault1']])],
					['output', new Map([['test', 'vault2']])]
				])
			}),
			getVaultlessStatus: vi.fn().mockReturnValue({
				value: new Map([
					['input', new Map([['test', undefined]])],
					['output', new Map([['test', undefined]])]
				])
			})
		} as unknown as DotrainOrderGui;

		mockStateUpdateCallback = vi.fn();

		(useGui as Mock).mockReturnValue(guiInstance);

		mockProps = {
			label: 'Input',
			vault: mockInput,
			tokenBalances: new Map()
		} as unknown as TokenIOInputComponentProps;
		outputMockProps = {
			label: 'Output',
			vault: mockInput,
			tokenBalances: new Map()
		} as unknown as TokenIOInputComponentProps;
	});

	it('renders with correct label and no token symbol', () => {
		const { getByText } = render(TokenIOInput, mockProps);
		expect(getByText('Input')).toBeInTheDocument();
	});

	it('renders input field with correct placeholder', () => {
		const { getByPlaceholderText } = render(TokenIOInput, mockProps);
		const input = getByPlaceholderText('Enter vault ID');
		expect(input).toBeInTheDocument();
	});

	it('displays the correct vault ID value', async () => {
		const { getByText } = render(TokenIOInput, mockProps);
		await waitFor(() => {
			expect(getByText('MOCK vault ID')).toBeInTheDocument();
		});
	});

	it('calls setVaultId when input changes', async () => {
		const input = render(TokenIOInput, mockProps).getByPlaceholderText('Enter vault ID');
		await fireEvent.input(input, { target: { value: 'vault1' } });
		expect(guiInstance.setVaultId).toHaveBeenCalledWith('input', 'test', 'vault1');
		expect(mockStateUpdateCallback).toHaveBeenCalledTimes(1);
	});

	it('calls setVaultId on output vault when input changes', async () => {
		const input = render(TokenIOInput, outputMockProps).getByPlaceholderText('Enter vault ID');
		await fireEvent.input(input, { target: { value: 'vault2' } });
		expect(guiInstance.setVaultId).toHaveBeenCalledWith('output', 'test', 'vault2');
		expect(mockStateUpdateCallback).toHaveBeenCalledTimes(1);
	});

	it('handles missing token info gracefully', () => {
		const propsWithUnknownToken = {
			...mockProps,
			vault: { token: { address: '0x789' } }
		};
		const { getByText } = render(
			TokenIOInput,
			propsWithUnknownToken as unknown as TokenIOInputComponentProps
		);
		expect(getByText('Input')).toBeInTheDocument();
	});

	describe('Balance Display', () => {
		it('passes token balance to VaultIdInformation component', async () => {
			const tokenBalances = new Map<string, TokenBalance>();
			tokenBalances.set('test', {
				value: {
					amount: Float.parse('1').value,
					formattedAmount: '1'
				} as RaindexAmount,
				loading: false,
				error: ''
			});

			const propsWithBalance = {
				...mockProps,
				tokenBalances
			};

			const { findByText } = render(TokenIOInput, propsWithBalance);

			const labelWithSymbol = await findByText('Input (MOCK)');
			expect(labelWithSymbol).toBeInTheDocument();
		});

		it('passes loading balance state to VaultIdInformation component', async () => {
			const tokenBalances = new Map<string, TokenBalance>();
			tokenBalances.set('test', {
				value: {
					amount: Float.parse('0').value,
					formattedAmount: '0'
				} as RaindexAmount,
				loading: true,
				error: ''
			});

			const propsWithLoadingBalance = {
				...mockProps,
				tokenBalances
			};

			const { findByText } = render(TokenIOInput, propsWithLoadingBalance);

			const labelWithSymbol = await findByText('Input (MOCK)');
			expect(labelWithSymbol).toBeInTheDocument();
		});

		it('passes balance error state to VaultIdInformation component', async () => {
			const tokenBalances = new Map<string, TokenBalance>();
			tokenBalances.set('test', {
				value: {
					amount: Float.parse('0').value,
					formattedAmount: '0'
				} as RaindexAmount,
				loading: false,
				error: 'Network error'
			});

			const propsWithErrorBalance = {
				...mockProps,
				tokenBalances
			};

			const { findByText } = render(TokenIOInput, propsWithErrorBalance);

			const labelWithSymbol = await findByText('Input (MOCK)');
			expect(labelWithSymbol).toBeInTheDocument();
		});

		it('handles missing token balance gracefully', async () => {
			const propsWithoutBalance = {
				...mockProps,
				tokenBalances: new Map() // Empty map
			};

			const { findByText } = render(TokenIOInput, propsWithoutBalance);

			const labelWithSymbol = await findByText('Input (MOCK)');
			expect(labelWithSymbol).toBeInTheDocument();
		});
	});

	it('fetches and displays token symbol when token key is present', async () => {
		const propsWithTokenKey = {
			...mockProps,
			vault: {
				token: {
					key: '0x456'
				}
			}
		} as unknown as TokenIOInputComponentProps;

		const { findByText } = render(TokenIOInput, propsWithTokenKey);

		const labelWithSymbol = await findByText('Input (MOCK)');
		expect(labelWithSymbol).toBeInTheDocument();
		expect(guiInstance.getTokenInfo).toHaveBeenCalledWith('0x456');
	});

	describe('Vaultless Mode', () => {
		it('renders vaultless toggle switch', () => {
			const { getByText } = render(TokenIOInput, mockProps);
			expect(getByText('Vaultless mode (direct wallet transfer)')).toBeInTheDocument();
		});

		it('initializes with vaultless enabled when vault.vaultless is true', async () => {
			const vaultlessProps = {
				...mockProps,
				vault: {
					...mockInput,
					vaultless: true
				}
			} as unknown as TokenIOInputComponentProps;

			const { findByText, queryByPlaceholderText } = render(TokenIOInput, vaultlessProps);

			const infoText = await findByText('Token transfers directly without vault storage.');
			expect(infoText).toBeInTheDocument();
			expect(queryByPlaceholderText('Enter vault ID')).not.toBeInTheDocument();
		});

		it('calls setVaultless when toggle is switched on', async () => {
			const { getByRole } = render(TokenIOInput, mockProps);
			const toggle = getByRole('checkbox');
			await fireEvent.click(toggle);
			expect(guiInstance.setVaultless).toHaveBeenCalledWith('input', 'test', true);
		});

		it('hides vault ID input when vaultless toggle is enabled', async () => {
			const { getByRole, queryByPlaceholderText, findByText } = render(TokenIOInput, mockProps);
			const toggle = getByRole('checkbox');
			await fireEvent.click(toggle);

			const infoText = await findByText('Token transfers directly without vault storage.');
			expect(infoText).toBeInTheDocument();
			expect(queryByPlaceholderText('Enter vault ID')).not.toBeInTheDocument();
		});

		it('shows approval amount input for output when vaultless toggle is enabled', async () => {
			const { getByRole, findByPlaceholderText, findByText } = render(
				TokenIOInput,
				outputMockProps
			);
			const toggle = getByRole('checkbox');
			await fireEvent.click(toggle);

			const label = await findByText('Approval Amount (defaults to unlimited)');
			expect(label).toBeInTheDocument();
			const input = await findByPlaceholderText('Leave empty for unlimited');
			expect(input).toBeInTheDocument();
		});

		it('does not show approval amount input for input when vaultless toggle is enabled', async () => {
			const { getByRole, findByText, queryByText } = render(TokenIOInput, mockProps);
			const toggle = getByRole('checkbox');
			await fireEvent.click(toggle);

			const infoText = await findByText('Token transfers directly without vault storage.');
			expect(infoText).toBeInTheDocument();
			expect(queryByText('Approval Amount (defaults to unlimited)')).not.toBeInTheDocument();
		});

		it('displays error when setVaultless fails', async () => {
			(guiInstance.setVaultless as Mock).mockReturnValue({
				error: { msg: 'Failed to set vaultless' }
			});

			const { getByRole, findByText } = render(TokenIOInput, mockProps);
			const toggle = getByRole('checkbox');
			await fireEvent.click(toggle);

			const errorMessage = await findByText('Failed to set vaultless');
			expect(errorMessage).toBeInTheDocument();
		});

		it('calls onApprovalAmountChange callback when approval amount input changes', async () => {
			const mockOnApprovalAmountChange = vi.fn();
			const propsWithCallback = {
				...outputMockProps,
				onApprovalAmountChange: mockOnApprovalAmountChange
			};

			const { getByRole, findByPlaceholderText } = render(TokenIOInput, propsWithCallback);
			const toggle = getByRole('checkbox');
			await fireEvent.click(toggle);

			const approvalInput = await findByPlaceholderText('Leave empty for unlimited');
			await fireEvent.input(approvalInput, { target: { value: '100' } });

			expect(mockOnApprovalAmountChange).toHaveBeenCalledWith('test', '100');
		});

		it('does not call onApprovalAmountChange when callback is undefined', async () => {
			const { getByRole, findByPlaceholderText } = render(TokenIOInput, outputMockProps);
			const toggle = getByRole('checkbox');
			await fireEvent.click(toggle);

			const approvalInput = await findByPlaceholderText('Leave empty for unlimited');
			await fireEvent.input(approvalInput, { target: { value: '100' } });
		});
	});
});
