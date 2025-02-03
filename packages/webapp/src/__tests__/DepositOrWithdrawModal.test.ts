import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import DepositOrWithdrawModal from '$lib/components/DepositOrWithdrawModal.svelte';
import { transactionStore } from '@rainlanguage/ui-components';
import { signerAddress } from '$lib/stores/wagmi';
import { readContract } from '@wagmi/core';

import type { ComponentProps } from 'svelte';
export type ModalProps = ComponentProps<DepositOrWithdrawModal>;

vi.mock('@rainlanguage/orderbook/js_api', () => ({
	getVaultDepositCalldata: vi.fn().mockResolvedValue({ to: '0x123', data: '0x456' }),
	getVaultApprovalCalldata: vi.fn().mockResolvedValue({ to: '0x789', data: '0xabc' }),
	getVaultWithdrawCalldata: vi.fn().mockResolvedValue({ to: '0xdef', data: '0xghi' })
}));

vi.mock('@wagmi/core', () => ({
	readContract: vi.fn()
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
		action: 'deposit' as const,
		vault: mockVault,
		chainId: 1,
		rpcUrl: 'https://example.com',
		onDepositOrWithdraw: vi.fn()
	} as unknown as ModalProps;

	beforeEach(() => {
		vi.clearAllMocks();
		transactionStore.reset();
		signerAddress.set('0x123');
	});

	it('renders deposit modal correctly', () => {
		render(DepositOrWithdrawModal, defaultProps);
		expect(screen.getByText('Enter Amount')).toBeInTheDocument();
		expect(screen.getByText('Deposit')).toBeInTheDocument();
	});

	it('renders withdraw modal correctly', () => {
		render(DepositOrWithdrawModal, {
			...defaultProps,
			action: 'withdraw'
		});
		expect(screen.getByText('Enter Amount')).toBeInTheDocument();
		expect(screen.getByText('Withdraw')).toBeInTheDocument();
	});

	it('disables continue button when amount is 0', () => {
		render(DepositOrWithdrawModal, defaultProps);
		const continueButton = screen.getByText('Deposit');
		expect(continueButton).toBeDisabled();
	});

	it('shows wallet connect button when not connected', () => {
		signerAddress.set('');
		render(DepositOrWithdrawModal, defaultProps);
		expect(screen.getByText('Connect Wallet')).toBeInTheDocument();
	});

	it('handles deposit transaction correctly', async () => {
		const handleTransactionSpy = vi.spyOn(transactionStore, 'handleDepositOrWithdrawTransaction');
		render(DepositOrWithdrawModal, defaultProps);

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const depositButton = screen.getByText('Deposit');
		await fireEvent.click(depositButton);

		expect(handleTransactionSpy).toHaveBeenCalledWith(
			expect.objectContaining({
				action: 'deposit',
				chainId: 1,
				vault: mockVault
			})
		);
	});

	it('Blocks deposit if not enough balance in wallet', async () => {
		(readContract as Mock).mockReturnValue(BigInt(0));

		render(DepositOrWithdrawModal, defaultProps);

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		await waitFor(() => {
			expect(screen.getByTestId('error')).toBeInTheDocument();
		});
	});

	it('Blocks withdrawal if not enough balance in vault', async () => {
		render(DepositOrWithdrawModal, defaultProps);

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '2' } });

		await waitFor(() => {
			expect(screen.getByTestId('error')).toBeInTheDocument();
		});
	});

	it('handles withdraw transaction correctly', async () => {
		const handleTransactionSpy = vi.spyOn(transactionStore, 'handleDepositOrWithdrawTransaction');
		render(DepositOrWithdrawModal, {
			...defaultProps,
			action: 'withdraw'
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const withdrawButton = screen.getByText('Withdraw');
		await fireEvent.click(withdrawButton);

		expect(handleTransactionSpy).toHaveBeenCalledWith(
			expect.objectContaining({
				action: 'withdraw',
				chainId: 1,
				vault: mockVault
			})
		);
	});

	it('closes modal and resets state', async () => {
		render(DepositOrWithdrawModal, defaultProps);

		const cancelButton = screen.getByText('Cancel');
		await fireEvent.click(cancelButton);

		expect(screen.queryByText('Enter Amount')).not.toBeInTheDocument();
	});
});
