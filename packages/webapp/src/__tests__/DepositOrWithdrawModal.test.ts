import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import DepositOrWithdrawModal from '$lib/components/DepositOrWithdrawModal.svelte';
import { transactionStore } from '@rainlanguage/ui-components';
import { readContract, switchChain } from '@wagmi/core';
import type { ComponentProps } from 'svelte';
import { getVaultApprovalCalldata, type SgVault } from '@rainlanguage/orderbook';
import { getVaultDepositCalldata } from '@rainlanguage/orderbook';
import type { Hex } from 'viem';
import truncateEthAddress from 'truncate-eth-address';
import { mockWeb3Config } from '$lib/__mocks__/mockWeb3Config';

type ModalProps = ComponentProps<DepositOrWithdrawModal>;

const { mockAppKitModalStore, mockConnectedStore, mockWagmiConfigStore } = await vi.hoisted(
	() => import('../lib/__mocks__/stores')
);

vi.mock('@rainlanguage/orderbook', () => ({
	getVaultDepositCalldata: vi.fn().mockResolvedValue({ to: '0x123', data: '0x456' }),
	getVaultApprovalCalldata: vi.fn().mockResolvedValue({ to: '0x789', data: '0xabc' }),
	getVaultWithdrawCalldata: vi.fn().mockResolvedValue({ to: '0xdef', data: '0xghi' })
}));

vi.mock('../lib/stores/wagmi', () => ({
	appKitModal: mockAppKitModalStore,
	connected: mockConnectedStore,
	wagmiConfig: mockWagmiConfigStore
}));

vi.mock('@wagmi/core', async (importOriginal) => {
	const original = (await importOriginal()) as object;
	return {
		...original,
		readContract: vi.fn(),
		switchChain: vi.fn().mockResolvedValue({ id: 1 })
	};
});

describe('DepositOrWithdrawModal', () => {
	const mockVault = {
		token: {
			address: '0x123',
			symbol: 'TEST',
			decimals: '18'
		},
		vaultId: '1',
		balance: BigInt(1)
	};

	const defaultProps = {
		open: true,
		args: {
			action: 'deposit' as const,
			vault: mockVault,
			chainId: 1,
			rpcUrl: 'https://example.com',
			onDepositOrWithdraw: vi.fn(),
			account: '0x0000000000000000000000000000000000000000'
		}
	} as unknown as ModalProps;

	beforeEach(() => {
		vi.clearAllMocks();
		vi.spyOn(transactionStore, 'reset');

		(readContract as Mock).mockReset();
		(switchChain as Mock).mockReset();
		(readContract as Mock).mockResolvedValue(BigInt(1000000000000000000)); // 1 token by default
	});

	it('renders deposit modal correctly', () => {
		render(DepositOrWithdrawModal, defaultProps);
		expect(screen.getByTestId('modal-title')).toHaveTextContent('Deposit');
		expect(screen.getByTestId('deposit-withdraw-button')).toBeInTheDocument();
	});

	it('renders withdraw modal correctly', () => {
		render(DepositOrWithdrawModal, {
			...defaultProps,
			args: {
				...defaultProps.args,
				action: 'withdraw'
			}
		});
		expect(screen.getByTestId('modal-title')).toHaveTextContent('Withdraw');
		expect(screen.getByTestId('deposit-withdraw-button')).toBeInTheDocument();
	});

	it('handles deposit transaction correctly', async () => {
		const handleTransactionSpy = vi.spyOn(transactionStore, 'handleDepositOrWithdrawTransaction');
		render(DepositOrWithdrawModal, defaultProps);

		// Wait for balance to be displayed
		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		// Use the InputTokenAmount to set value
		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const depositButton = screen.getByTestId('deposit-withdraw-button');
		await fireEvent.click(depositButton);

		expect(handleTransactionSpy).toHaveBeenCalledWith({
			action: 'deposit',
			chainId: 1,
			vault: mockVault,
			config: mockWeb3Config,
			subgraphUrl: undefined,
			approvalCalldata: { to: '0x789', data: '0xabc' },
			transactionCalldata: { to: '0x123', data: '0x456' }
		});
	});

	it('handles withdraw transaction correctly', async () => {
		const handleTransactionSpy = vi.spyOn(transactionStore, 'handleDepositOrWithdrawTransaction');
		render(DepositOrWithdrawModal, {
			...defaultProps,
			args: {
				...defaultProps.args,
				action: 'withdraw'
			}
		});

		// Wait for balance to be displayed
		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const withdrawButton = screen.getByTestId('deposit-withdraw-button');
		await fireEvent.click(withdrawButton);

		expect(handleTransactionSpy).toHaveBeenCalledWith({
			config: mockWeb3Config,
			transactionCalldata: { to: '0xdef', data: '0xghi' },
			action: 'withdraw',
			chainId: 1,
			vault: mockVault,
			subgraphUrl: undefined
		});
	});

	it('shows error when amount exceeds balance for deposit', async () => {
		(readContract as Mock).mockResolvedValue(BigInt(500000000000000000)); // 0.5 tokens
		render(DepositOrWithdrawModal, defaultProps);

		// Wait for balance to be displayed
		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		expect(screen.getByTestId('amount-error')).toHaveTextContent(
			'Amount cannot exceed available balance.'
		);
	});

	it('shows error when amount exceeds balance for withdraw', async () => {
		const mockVaultWithBalance = {
			...mockVault,
			balance: BigInt(500000000000000000) // 0.5 tokens
		};

		render(DepositOrWithdrawModal, {
			...defaultProps,
			args: {
				...defaultProps.args,
				action: 'withdraw',
				vault: mockVaultWithBalance as unknown as SgVault
			}
		});

		// Wait for balance to be displayed
		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		expect(screen.getByTestId('amount-error')).toHaveTextContent(
			'Amount cannot exceed available balance.'
		);
	});

	it('shows chain switch error when switching fails', async () => {
		(switchChain as Mock).mockRejectedValue(new Error('Failed to switch chain'));
		render(DepositOrWithdrawModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText(/Switch to .* to check your balance/)).toBeInTheDocument();
		});
	});

	it('disables continue button when amount is 0', async () => {
		render(DepositOrWithdrawModal, defaultProps);

		// Wait for balance to be displayed
		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '0' } });

		const continueButton = screen.getByTestId('deposit-withdraw-button');
		expect(continueButton).toBeDisabled();
	});

	it('disables continue button when amount exceeds balance', async () => {
		(readContract as Mock).mockResolvedValue(BigInt(500000000000000000)); // 0.5 tokens
		render(DepositOrWithdrawModal, defaultProps);

		// Wait for balance to be displayed
		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const continueButton = screen.getByTestId('deposit-withdraw-button');
		expect(continueButton).toBeDisabled();
	});

	it('shows loading state while checking calldata', async () => {
		render(DepositOrWithdrawModal, defaultProps);

		// Wait for balance to be displayed
		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const depositButton = screen.getByTestId('deposit-withdraw-button');
		await fireEvent.click(depositButton);

		expect(screen.getByText('Checking...')).toBeInTheDocument();
	});

	it('handles failed calldata fetch', async () => {
		vi.mocked(getVaultDepositCalldata).mockRejectedValueOnce(new Error('Failed to fetch'));

		render(DepositOrWithdrawModal, defaultProps);

		// Wait for balance to be displayed
		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const depositButton = screen.getByTestId('deposit-withdraw-button');
		await fireEvent.click(depositButton);

		await waitFor(() => {
			expect(screen.getByText('Failed to get calldata.')).toBeInTheDocument();
		});
	});

	it('handles deposit without approval when approval fails', async () => {
		const handleTransactionSpy = vi.spyOn(transactionStore, 'handleDepositOrWithdrawTransaction');
		vi.mocked(getVaultApprovalCalldata).mockRejectedValueOnce(new Error('Approval not needed'));

		render(DepositOrWithdrawModal, defaultProps);

		// Wait for balance to be displayed
		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const depositButton = screen.getByTestId('deposit-withdraw-button');
		await fireEvent.click(depositButton);

		expect(handleTransactionSpy).toHaveBeenCalledWith({
			action: 'deposit',
			chainId: 1,
			vault: mockVault,
			config: mockWeb3Config,
			subgraphUrl: undefined,
			approvalCalldata: undefined,
			transactionCalldata: { to: '0x123', data: '0x456' }
		});
	});

	it('handles zero user balance correctly for deposit action', async () => {
		(readContract as Mock).mockResolvedValue(BigInt(0));

		render(DepositOrWithdrawModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
			expect(screen.getByText('0 TEST')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '0.1' } });

		const depositButton = screen.getByTestId('deposit-withdraw-button');
		expect(depositButton).toBeDisabled();
		expect(screen.getByTestId('amount-error')).toHaveTextContent(
			'Amount cannot exceed available balance.'
		);
	});

	it('handles zero vault balance correctly for withdraw action', async () => {
		const mockVaultWithZeroBalance = {
			...mockVault,
			balance: BigInt(0)
		};

		render(DepositOrWithdrawModal, {
			...defaultProps,
			args: {
				...defaultProps.args,
				action: 'withdraw',
				vault: mockVaultWithZeroBalance as unknown as SgVault
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Balance of vault')).toBeInTheDocument();
			expect(screen.getByText('0 TEST')).toBeInTheDocument();
		});

		const withdrawInput = screen.getByRole('textbox');
		await fireEvent.input(withdrawInput, { target: { value: '0.1' } });

		const withdrawButton = screen.getByTestId('deposit-withdraw-button');
		expect(withdrawButton).toBeDisabled();
		expect(screen.getByTestId('amount-error')).toHaveTextContent(
			'Amount cannot exceed available balance.'
		);
	});

	it('displays user balance correctly on deposit screen', async () => {
		const userBalanceAmount = BigInt(2500000000000000000); // 2.5 tokens
		(readContract as Mock).mockResolvedValue(userBalanceAmount);

		render(DepositOrWithdrawModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Balance of connected wallet')).toBeInTheDocument();
			expect(screen.getByText('2.5 TEST')).toBeInTheDocument();
		});
	});

	it('displays vault balance correctly on withdraw screen', async () => {
		const mockVaultWithBalance = {
			...mockVault,
			balance: BigInt(3700000000000000000) // 3.7 tokens
		};

		render(DepositOrWithdrawModal, {
			...defaultProps,
			args: {
				...defaultProps.args,
				action: 'withdraw',
				vault: mockVaultWithBalance as unknown as SgVault
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Balance of vault')).toBeInTheDocument();
			expect(screen.getByText('3.7 TEST')).toBeInTheDocument();
		});
	});

	it('shows error message when getUserBalance fails', async () => {
		vi.mocked(readContract).mockRejectedValue(new Error('Failed to get balance'));

		render(DepositOrWithdrawModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Failed to get user balance.')).toBeInTheDocument();
		});
	});

	it('shows wallet address in truncated form', async () => {
		render(DepositOrWithdrawModal, defaultProps);

		await waitFor(() => {
			expect(
				screen.getByText(truncateEthAddress('0x0000000000000000000000000000000000000000'))
			).toBeInTheDocument();
		});
	});

	it('shows the cancel button that closes the modal', async () => {
		render(DepositOrWithdrawModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Cancel')).toBeInTheDocument();
		});

		const cancelButton = screen.getByText('Cancel');
		await fireEvent.click(cancelButton);
		await waitFor(() => {
			expect(transactionStore.reset).toHaveBeenCalled();
		});
	});

	it('shows the WalletConnect button when account is not provided', async () => {
		render(DepositOrWithdrawModal, {
			...defaultProps,
			args: {
				...defaultProps.args,
				account: null as unknown as Hex
			}
		});

		await waitFor(() => {
			expect(screen.getByText('Connect your wallet to continue.')).toBeInTheDocument();
			expect(screen.queryByTestId('deposit-withdraw-button')).not.toBeInTheDocument();
		});
	});
});
