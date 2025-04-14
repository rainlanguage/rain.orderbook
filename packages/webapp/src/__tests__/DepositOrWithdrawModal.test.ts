import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import DepositOrWithdrawModal from '$lib/components/DepositOrWithdrawModal.svelte';
import { transactionStore, useAccount } from '@rainlanguage/ui-components';
import { readContract, switchChain } from '@wagmi/core';
import type { ComponentProps } from 'svelte';
import {
	getVaultApprovalCalldata,
	type SgVault,
	getVaultDepositCalldata,
	getVaultWithdrawCalldata
} from '@rainlanguage/orderbook';
import { get, readable } from 'svelte/store';

type ModalProps = ComponentProps<DepositOrWithdrawModal>;

const { mockAppKitModalStore, mockConnectedStore, mockWagmiConfigStore } = await vi.hoisted(
	() => import('../lib/__mocks__/stores')
);

vi.mock('@rainlanguage/orderbook', async (importOriginal) => {
	const actual = await importOriginal<typeof import('@rainlanguage/orderbook')>();
	return {
		...actual,
		getVaultDepositCalldata: vi.fn(),
		getVaultApprovalCalldata: vi.fn(),
		getVaultWithdrawCalldata: vi.fn()
	};
});

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	return {
		...(await importOriginal()),
		useAccount: vi.fn()
	};
});

vi.mock('../lib/stores/wagmi', () => ({
	appKitModal: mockAppKitModalStore,
	connected: mockConnectedStore,
	wagmiConfig: mockWagmiConfigStore
}));

vi.mock('@wagmi/core', () => ({
	readContract: vi.fn(),
	switchChain: vi.fn().mockResolvedValue({ id: 1 })
}));

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
			account: readable('0x123')
		}
	} as unknown as ModalProps;

	beforeEach(() => {
		vi.clearAllMocks();
		transactionStore.reset();
		vi.mocked(useAccount).mockReturnValue({
			account: readable('0x')
		});
		(readContract as Mock).mockReset();
		(switchChain as Mock).mockReset();
		(getVaultApprovalCalldata as Mock).mockResolvedValue({ to: '0x789', data: '0xabc' });
		(getVaultDepositCalldata as Mock).mockResolvedValue({ to: '0x123', data: '0x456' });
		(getVaultWithdrawCalldata as Mock).mockResolvedValue({ to: '0xdef', data: '0xghi' });
	});

	it('renders deposit modal correctly', () => {
		render(DepositOrWithdrawModal, defaultProps);
		expect(screen.getByText('Enter Amount')).toBeInTheDocument();
		expect(screen.getByText('Deposit')).toBeInTheDocument();
	});

	it('renders withdraw modal correctly', () => {
		render(DepositOrWithdrawModal, {
			...defaultProps,
			args: {
				...defaultProps.args,
				action: 'withdraw'
			}
		});
		expect(screen.getByText('Enter Amount')).toBeInTheDocument();
		expect(screen.getByText('Withdraw')).toBeInTheDocument();
	});

	it('handles deposit transaction correctly', async () => {
		const handleTransactionSpy = vi.spyOn(transactionStore, 'handleDepositOrWithdrawTransaction');
		render(DepositOrWithdrawModal, defaultProps);

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const depositButton = screen.getByText('Deposit');
		await fireEvent.click(depositButton);

		const wagmiConfig = get(mockWagmiConfigStore);

		expect(handleTransactionSpy).toHaveBeenCalledWith({
			action: 'deposit',
			chainId: 1,
			vault: mockVault,
			config: wagmiConfig,
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

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const withdrawButton = screen.getByText('Withdraw');
		await fireEvent.click(withdrawButton);
		const wagmiConfig = get(mockWagmiConfigStore);

		expect(handleTransactionSpy).toHaveBeenCalledWith({
			config: wagmiConfig,
			transactionCalldata: { to: '0xdef', data: '0xghi' },
			action: 'withdraw',
			chainId: 1,
			vault: mockVault,
			subgraphUrl: undefined
		});
	});

	it('shows error when amount exceeds balance for deposit', async () => {
		(readContract as Mock).mockResolvedValue(BigInt(0));
		render(DepositOrWithdrawModal, defaultProps);

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '2' } });

		expect(screen.getByTestId('amount-error')).toHaveTextContent(
			'Amount cannot exceed available balance.'
		);
	});

	it('shows error when amount exceeds balance for withdraw', async () => {
		render(DepositOrWithdrawModal, {
			...defaultProps,
			args: {
				...defaultProps.args,
				action: 'withdraw'
			}
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '2' } });

		expect(screen.getByTestId('amount-error')).toHaveTextContent(
			'Amount cannot exceed available balance.'
		);
	});

	it('shows chain switch error when switching fails', async () => {
		(switchChain as Mock).mockRejectedValue(new Error('Failed to switch chain'));
		render(DepositOrWithdrawModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByTestId('error-message')).toHaveTextContent(
				'Switch to Ethereum to check your balance.'
			);
		});
	});

	it('disables continue button when amount is 0', () => {
		render(DepositOrWithdrawModal, defaultProps);

		const input = screen.getByRole('textbox');
		fireEvent.input(input, { target: { value: '0' } });

		const continueButton = screen.getByText('Deposit');
		expect(continueButton).toBeDisabled();
	});

	it('disables continue button when amount exceeds balance', async () => {
		(readContract as Mock).mockResolvedValue(BigInt(0));
		render(DepositOrWithdrawModal, defaultProps);

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const continueButton = screen.getByText('Deposit');
		expect(continueButton).toBeDisabled();
	});

	it('shows loading state while checking calldata', async () => {
		render(DepositOrWithdrawModal, defaultProps);

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const depositButton = screen.getByText('Deposit');
		await fireEvent.click(depositButton);

		expect(screen.getByText('Checking...')).toBeInTheDocument();
	});

	it('handles failed calldata fetch', async () => {
		(getVaultDepositCalldata as Mock).mockRejectedValueOnce(new Error('Failed to fetch'));

		render(DepositOrWithdrawModal, defaultProps);

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const depositButton = screen.getByText('Deposit');
		await fireEvent.click(depositButton);

		await waitFor(() => {
			expect(screen.getByTestId('error-message')).toHaveTextContent('Failed to get calldata.');
		});
	});

	it('handles deposit without approval when approval fails', async () => {
		const handleTransactionSpy = vi.spyOn(transactionStore, 'handleDepositOrWithdrawTransaction');
		(getVaultApprovalCalldata as Mock).mockRejectedValueOnce(new Error('Approval not needed'));

		render(DepositOrWithdrawModal, defaultProps);

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const depositButton = screen.getByText('Deposit');
		await fireEvent.click(depositButton);
		const wagmiConfig = get(mockWagmiConfigStore);
		expect(handleTransactionSpy).toHaveBeenCalledWith({
			action: 'deposit',
			chainId: 1,
			vault: mockVault,
			config: wagmiConfig,
			subgraphUrl: undefined,
			approvalCalldata: undefined,
			transactionCalldata: { to: '0x123', data: '0x456' }
		});
	});
	it('handles zero user balance correctly for deposit action', async () => {
		(readContract as Mock).mockResolvedValue(BigInt(0));

		render(DepositOrWithdrawModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Your Balance:')).toBeInTheDocument();
			expect(screen.getByText('0')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '0.1' } });

		const depositButton = screen.getByText('Deposit');
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
			expect(screen.getByText('Vault Balance:')).toBeInTheDocument();
			expect(screen.getByText('0')).toBeInTheDocument();
		});

		const withdrawInput = screen.getByRole('textbox');
		await fireEvent.input(withdrawInput, { target: { value: '0.1' } });

		const withdrawButton = screen.getByText('Withdraw');
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
			expect(screen.getByText('Your Balance:')).toBeInTheDocument();
			expect(screen.getByText('2.5')).toBeInTheDocument();
		});
	});

	it('displays vault balance correctly on withdraw screen', async () => {
		const mockVaultWithBalance = {
			...mockVault,
			balance: BigInt(3700000000000000000)
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
			expect(screen.getByText('Vault Balance:')).toBeInTheDocument();
			expect(screen.getByText('3.7')).toBeInTheDocument();
		});
	});

	it('shows error message when getUserBalance fails', async () => {
		vi.mocked(readContract).mockRejectedValue(new Error('Failed to get balance'));

		render(DepositOrWithdrawModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByTestId('error-message')).toBeInTheDocument();
			expect(screen.getByTestId('error-message')).toHaveTextContent('Failed to get user balance.');
		});
	});
});
