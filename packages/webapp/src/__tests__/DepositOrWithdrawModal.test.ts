import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, fireEvent, screen, waitFor, cleanup } from '@testing-library/svelte';
import DepositOrWithdrawModal from '$lib/components/DepositOrWithdrawModal.svelte';
import { transactionStore } from '@rainlanguage/ui-components';
import { readContract, switchChain } from '@wagmi/core';

import type { ComponentProps } from 'svelte';
import { getVaultApprovalCalldata } from '@rainlanguage/orderbook/js_api';
import { getVaultDepositCalldata } from '@rainlanguage/orderbook/js_api';
import { get } from 'svelte/store';

export type ModalProps = ComponentProps<DepositOrWithdrawModal>;

const { mockWagmiConfigStore, mockSignerAddressStore, mockAppKitModalStore, mockConnectedStore } =
	await vi.hoisted(() => import('../lib/__mocks__/stores'));

vi.mock('@rainlanguage/orderbook/js_api', () => ({
	getVaultDepositCalldata: vi.fn().mockResolvedValue({ to: '0x123', data: '0x456' }),
	getVaultApprovalCalldata: vi.fn().mockResolvedValue({ to: '0x789', data: '0xabc' }),
	getVaultWithdrawCalldata: vi.fn().mockResolvedValue({ to: '0xdef', data: '0xghi' })
}));

vi.mock('../lib/stores/wagmi', () => ({
	appKitModal: mockAppKitModalStore,
	wagmiConfig: mockWagmiConfigStore,
	signerAddress: mockSignerAddressStore,
	connected: mockConnectedStore
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
			onDepositOrWithdraw: vi.fn()
		}
	} as unknown as ModalProps;

	beforeEach(() => {
		vi.clearAllMocks();
		transactionStore.reset();
		mockSignerAddressStore.mockSetSubscribeValue('0x123');
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
		vi.mocked(getVaultDepositCalldata).mockRejectedValueOnce(new Error('Failed to fetch'));

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
		vi.mocked(getVaultApprovalCalldata).mockRejectedValueOnce(new Error('Approval not needed'));

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
});
