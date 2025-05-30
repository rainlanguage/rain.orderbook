import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import TransactionConfirmationModal from '$lib/components/TransactionConfirmationModal.svelte';
import type { TransactionConfirmationProps } from '@rainlanguage/ui-components';
import type { SgOrder } from '@rainlanguage/orderbook';
import { handleWalletConfirmation } from '$lib/services/handleWalletConfirmation';

vi.mock('$lib/services/handleWalletConfirmation', () => ({
	handleWalletConfirmation: vi.fn()
}));

describe('TransactionConfirmationModal', () => {
	const mockCalldata = '0x1234567890abcdef';
	const mockTxHash = '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890';

	const mockOrder: SgOrder = {
		id: '0x1',
		orderBytes: '0x2',
		orderHash: '0x3',
		owner: '0x4',
		outputs: [],
		inputs: [],
		orderbook: { id: '0x5' },
		active: true,
		timestampAdded: '1234567890',
		addEvents: [],
		trades: [],
		removeEvents: []
	};

	const testModalTitle = 'Test Modal Title';
	const defaultProps: TransactionConfirmationProps = {
		open: true,
		modalTitle: testModalTitle,
		args: {
			chainId: 1,
			toAddress: '0x789',
			calldata: mockCalldata,
			onConfirm: vi.fn(),
			entity: mockOrder
		}
	};

	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetAllMocks();
		vi.mocked(handleWalletConfirmation).mockResolvedValue({
			state: { status: 'confirmed' },
			hash: mockTxHash
		});
	});

	it('shows awaiting confirmation state initially and correct title', () => {
		render(TransactionConfirmationModal, defaultProps);

		expect(screen.getByText(defaultProps.modalTitle as string)).toBeInTheDocument();
		expect(screen.getByText(testModalTitle)).toBeInTheDocument();
		expect(screen.getByText('Please confirm this transaction in your wallet.')).toBeInTheDocument();
		expect(screen.getByTestId('transaction-modal')).toBeInTheDocument();
	});

	it('handles successful transaction flow', async () => {
		render(TransactionConfirmationModal, defaultProps);

		await waitFor(() => {
			expect(handleWalletConfirmation).toHaveBeenCalledWith(defaultProps.args);
			expect(screen.getByText('Transaction submitted')).toBeInTheDocument();
			expect(
				screen.getByText('Transaction has been submitted to the network.')
			).toBeInTheDocument();
			expect(screen.getByText('✅')).toBeInTheDocument();
			expect(screen.queryByText('Dismiss')).toBeInTheDocument();
		});
	});

	it('handles chain switch error', async () => {
		const errorMessage = 'Failed to switch chain';
		vi.mocked(handleWalletConfirmation).mockResolvedValue({
			state: {
				status: 'error',
				reason: errorMessage
			}
		});

		render(TransactionConfirmationModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Confirmation failed')).toBeInTheDocument();
			expect(screen.getByText(errorMessage)).toBeInTheDocument();
			expect(screen.getByText('Dismiss')).toBeInTheDocument();
			expect(screen.getByText('❌')).toBeInTheDocument();
		});
	});

	it('handles transaction rejection', async () => {
		vi.mocked(handleWalletConfirmation).mockResolvedValue({
			state: {
				status: 'rejected',
				reason: 'User rejected transaction'
			}
		});

		render(TransactionConfirmationModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Confirmation failed')).toBeInTheDocument();
			expect(screen.getByText('User rejected transaction')).toBeInTheDocument();
			expect(screen.getByText('Dismiss')).toBeInTheDocument();
			expect(screen.getByText('❌')).toBeInTheDocument();
		});
	});

	it('handles non-Error chain switch failure', async () => {
		vi.mocked(handleWalletConfirmation).mockResolvedValue({
			state: {
				status: 'error',
				reason: 'Failed to switch chain'
			}
		});

		render(TransactionConfirmationModal, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Confirmation failed')).toBeInTheDocument();
			expect(screen.getByText('Failed to switch chain')).toBeInTheDocument();
			expect(screen.getByText('Dismiss')).toBeInTheDocument();
			expect(screen.getByText('❌')).toBeInTheDocument();
		});
	});

	it('closes the modal on success if closeOnConfirm is true', async () => {
		render(TransactionConfirmationModal, { ...defaultProps, closeOnConfirm: true });

		await waitFor(() => {
			expect(handleWalletConfirmation).toHaveBeenCalledWith(defaultProps.args);
			const modal = screen.queryByTestId('transaction-modal');
			expect(modal).not.toBeInTheDocument();
		});
	});
});
