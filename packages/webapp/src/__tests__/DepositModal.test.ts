import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import DepositModal from '$lib/components/DepositModal.svelte';
import { transactionStore } from '@rainlanguage/ui-components';
import { signerAddress } from '$lib/stores/wagmi';
import { readContract, switchChain } from '@wagmi/core';

import type { ComponentProps } from 'svelte';
import { getVaultApprovalCalldata } from '@rainlanguage/orderbook/js_api';
import { getVaultDepositCalldata } from '@rainlanguage/orderbook/js_api';

export type ModalProps = ComponentProps<DepositModal>;

vi.mock('@rainlanguage/orderbook/js_api', () => ({
	getVaultDepositCalldata: vi.fn().mockResolvedValue({ to: '0x123', data: '0x456' }),
	getVaultApprovalCalldata: vi.fn().mockResolvedValue({ to: '0x789', data: '0xabc' })
}));

vi.mock('@wagmi/core', () => ({
	readContract: vi.fn(),
	switchChain: vi.fn()
}));

describe('DepositModal', () => {
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
			vault: mockVault,
			chainId: 1,
			rpcUrl: 'https://example.com',
			subgraphUrl: 'https://example.com/subgraph'
		}
	} as unknown as ModalProps;

	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetAllMocks();
		transactionStore.reset();
		signerAddress.set('0x123');
		(readContract as Mock).mockResolvedValue(BigInt(1000000000000000000)); // 1 token
	});

	it('renders deposit modal correctly', () => {
		render(DepositModal, defaultProps);
		expect(screen.getByText('Enter Deposit Amount')).toBeInTheDocument();
		expect(screen.getByText('Deposit')).toBeInTheDocument();
	});

	it.only('handles deposit transaction correctly', async () => {
	const handleTransactionSpy = vi.spyOn(transactionStore, 'handleDepositOrWithdrawTransaction');
		render(DepositModal, defaultProps);

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const depositButton = screen.getByText('Deposit');
		await fireEvent.click(depositButton);

		expect(handleTransactionSpy).toHaveBeenCalledWith({
			action: 'deposit',
			chainId: 1,
			vault: mockVault,
			config: undefined,
			subgraphUrl: undefined,
			approvalCalldata: { to: '0x789', data: '0xabc' },
			transactionCalldata: { to: '0x123', data: '0x456' }
		});
	});

	it('shows error when amount exceeds balance for deposit', async () => {
		(readContract as Mock).mockResolvedValue(BigInt(0));
		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByTestId('balance-badge')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '2' } });

		expect(screen.getByTestId('error')).toHaveTextContent(
			'Amount cannot exceed available balance.'
		);
	});

	it('shows chain switch error when switching fails', async () => {
		(switchChain as Mock).mockRejectedValue(new Error('Failed to switch chain'));
		render(DepositModal, defaultProps);

		await waitFor(() => {
			// In the new implementation, the error appears in the loading state of the await block
			expect(screen.getByText(/Error loading balance/)).toBeInTheDocument();
		});
	});

	it('disables continue button when amount is 0', async () => {
		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByTestId('balance-badge')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		fireEvent.input(input, { target: { value: '0' } });

		const continueButton = screen.getByText('Deposit');
		expect(continueButton).toBeDisabled();
	});

	it('disables continue button when amount exceeds balance', async () => {
		(readContract as Mock).mockResolvedValue(BigInt(0));
		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByTestId('balance-badge')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const continueButton = screen.getByText('Deposit');
		expect(continueButton).toBeDisabled();
	});

	it('shows loading state while checking calldata', async () => {
		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByTestId('balance-badge')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const depositButton = screen.getByText('Deposit');
		await fireEvent.click(depositButton);

		expect(screen.getByText('Checking...')).toBeInTheDocument();
	});

	it('handles failed calldata fetch', async () => {
		vi.mocked(getVaultDepositCalldata).mockRejectedValueOnce(new Error('Failed to fetch'));

		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByTestId('balance-badge')).toBeInTheDocument();
		});

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

		render(DepositModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByTestId('balance-badge')).toBeInTheDocument();
		});

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const depositButton = screen.getByText('Deposit');
		await fireEvent.click(depositButton);

		expect(handleTransactionSpy).toHaveBeenCalledWith({
			action: 'deposit',
			chainId: 1,
			vault: mockVault,
			config: undefined,
			subgraphUrl: 'https://example.com/subgraph',
			approvalCalldata: undefined,
			transactionCalldata: { to: '0x123', data: '0x456' }
		});
	});

	it('shows correct user balance when making a deposit', async () => {
		(readContract as Mock).mockResolvedValue(BigInt(1000000000000000000));
		render(DepositModal, defaultProps);

		await waitFor(() => {
			const balanceBadge = screen.getByTestId('balance-badge');
			expect(balanceBadge).toHaveTextContent(/Your balance: 1 TEST/);
		});
	});
});