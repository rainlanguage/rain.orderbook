import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import WithdrawModal from '$lib/components/WithdrawModal.svelte';
import { transactionStore } from '@rainlanguage/ui-components';
import { signerAddress } from '$lib/stores/wagmi';

import type { ComponentProps } from 'svelte';
import { getVaultWithdrawCalldata, type SgVault } from '@rainlanguage/orderbook/js_api';

export type ModalProps = ComponentProps<WithdrawModal>;

vi.mock('@rainlanguage/orderbook/js_api', () => ({
	getVaultWithdrawCalldata: vi.fn().mockResolvedValue({ to: '0xdef', data: '0xghi' })
}));

describe('WithdrawModal', () => {
	const mockVault = {
		token: {
			address: '0x123',
			symbol: 'TEST',
			decimals: '18'
		},
		vaultId: '1',
		balance: '1000000000000000000' // 1 TEST token
	};

	const defaultProps = {
		open: true,
		args: {
			vault: mockVault as unknown as SgVault,
			chainId: 1,
			subgraphUrl: 'https://example.com/subgraph'
		}
	} as unknown as ModalProps;

	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetAllMocks();
		transactionStore.reset();
		signerAddress.set('0x123');
	});

	it('renders withdraw modal correctly', () => {
		render(WithdrawModal, defaultProps);
		expect(screen.getByText('Enter Withdraw Amount')).toBeInTheDocument();
		expect(screen.getByText('Withdraw')).toBeInTheDocument();
	});

	it('handles withdraw transaction correctly', async () => {
		const handleTransactionSpy = vi.spyOn(transactionStore, 'handleDepositOrWithdrawTransaction');
		render(WithdrawModal, defaultProps);

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const withdrawButton = screen.getByText('Withdraw');
		await fireEvent.click(withdrawButton);

		expect(handleTransactionSpy).toHaveBeenCalledWith({
			config: undefined,
			transactionCalldata: { to: '0xdef', data: '0xghi' },
			action: 'withdraw',
			chainId: 1,
			vault: mockVault,
			subgraphUrl: 'https://example.com/subgraph'
		});
	});

	it('shows error when amount exceeds balance for withdraw', async () => {
		render(WithdrawModal, defaultProps);

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '2' } });

		expect(screen.getByTestId('error')).toHaveTextContent(
			'Amount cannot exceed available balance.'
		);
	});

	it('disables continue button when amount is 0', () => {
		render(WithdrawModal, defaultProps);

		const input = screen.getByRole('textbox');
		fireEvent.input(input, { target: { value: '0' } });

		const continueButton = screen.getByText('Withdraw');
		expect(continueButton).toBeDisabled();
	});

	it('disables continue button when amount exceeds balance', async () => {
		render(WithdrawModal, defaultProps);

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '2' } });

		const continueButton = screen.getByText('Withdraw');
		expect(continueButton).toBeDisabled();
	});

	it('shows loading state while checking calldata', async () => {
		render(WithdrawModal, defaultProps);

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const withdrawButton = screen.getByText('Withdraw');
		await fireEvent.click(withdrawButton);

		expect(screen.getByText('Checking...')).toBeInTheDocument();
	});

	it('handles failed calldata fetch', async () => {
		vi.mocked(getVaultWithdrawCalldata).mockRejectedValueOnce(new Error('Failed to fetch'));

		render(WithdrawModal, defaultProps);

		const input = screen.getByRole('textbox');
		await fireEvent.input(input, { target: { value: '1' } });

		const withdrawButton = screen.getByText('Withdraw');
		await fireEvent.click(withdrawButton);

		await waitFor(() => {
			expect(screen.getByTestId('error-message')).toHaveTextContent('Failed to get calldata.');
		});
	});

	it('shows correct vault balance for withdraw', async () => {
		const mockVaultWithBalance = {
			...mockVault,
			balance: '3000000000000000000' // 3 TEST tokens
		};

		render(WithdrawModal, {
			...defaultProps,
			args: {
				...defaultProps.args,
				vault: mockVaultWithBalance as unknown as SgVault
			}
		});

		const balanceBadge = screen.getByTestId('balance-badge');
		expect(balanceBadge).toHaveTextContent('Vault balance: 3 TEST');
	});
});