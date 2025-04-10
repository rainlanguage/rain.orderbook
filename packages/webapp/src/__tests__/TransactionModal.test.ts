import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import TransactionModal from '../lib/components/TransactionModal.svelte';
import { TransactionStatus } from '@rainlanguage/ui-components';
import userEvent from '@testing-library/user-event';
import { initialState } from '../../../ui-components/dist/stores/transactionStore';

// Add hoisted mock import
const { mockTransactionStore } = await vi.hoisted(() => import('@rainlanguage/ui-components'));

// Mock the transaction store
vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		transactionStore: mockTransactionStore
	};
});
const mockDispatch = vi.fn();

vi.mock('svelte', async (importOriginal) => {
	const original = (await importOriginal()) as object;
	return {
		...original,
		createEventDispatcher: () => mockDispatch
	};
});

describe('TransactionModal Component', () => {
	const mockError = 'There was a problem with the transaction!';
	const messages = {
		success: 'Transaction Successful',
		pending: 'Transaction Pending'
	};
	const resetSpy = vi.spyOn(mockTransactionStore, 'reset');

	beforeEach(() => {
		mockTransactionStore.mockSetSubscribeValue(initialState);
	});

	it('should render correctly in IDLE state', async () => {
		render(TransactionModal, { props: { open: true, messages } });
		expect(screen.queryByTestId('transaction-modal')).toBeInTheDocument();
		// In IDLE state, modal should be empty
		expect(screen.queryByText(messages.pending)).not.toBeInTheDocument();
		expect(screen.queryByText(messages.success)).not.toBeInTheDocument();
		expect(screen.queryByText(mockError)).not.toBeInTheDocument();
	});

	it('should display an error when transaction fails', async () => {
		mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatus.ERROR,
			error: mockError,
			hash: '0xMockTransactionHash'
		});

		render(TransactionModal, { props: { open: true, messages } });

		await waitFor(() => {
			expect(screen.getByTestId('error-icon')).toBeInTheDocument();
			expect(screen.getByText(mockError)).toBeInTheDocument();
		});

		// Test modal close behavior
		const dismissButton = screen.getByText('Dismiss');
		await userEvent.click(dismissButton);
		expect(resetSpy).toHaveBeenCalled();
	});

	it('should display success message when transaction succeeds', async () => {
		const successMessage = 'Transaction succeeded';
		mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatus.SUCCESS,
			message: successMessage,
			hash: '0xMockTransactionHash'
		});

		render(TransactionModal, { props: { open: true, messages } });

		await waitFor(() => {
			expect(screen.getByText('✅')).toBeInTheDocument();
			expect(screen.getByText(messages.success)).toBeInTheDocument();
			expect(screen.getByText(successMessage)).toBeInTheDocument();
		});

		const dismissButton = screen.getByText('Dismiss');
		await userEvent.click(dismissButton);
		expect(resetSpy).toHaveBeenCalled();
	});

	it('should display pending state with a spinner for pending transactions', async () => {
		const pendingMessage = 'Waiting for wallet confirmation...';
		mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatus.PENDING_WALLET,
			message: pendingMessage
		});

		render(TransactionModal, { props: { open: true, messages } });

		await waitFor(() => {
			expect(screen.getByText(messages.pending)).toBeInTheDocument();
			expect(screen.getByText(pendingMessage)).toBeInTheDocument();
			expect(document.querySelector('[role="status"]')).toBeInTheDocument();
		});
	});

	it('should handle multiple statuses like CHECKING_ALLOWANCE and PENDING_APPROVAL', async () => {
		const checkingMessage = 'Checking your allowance...';
		mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatus.CHECKING_ALLOWANCE,
			message: checkingMessage
		});

		render(TransactionModal, { props: { open: true, messages } });

		await waitFor(() => {
			expect(screen.getByText(messages.pending)).toBeInTheDocument();
			expect(screen.getByText(checkingMessage)).toBeInTheDocument();
		});

		const approvalMessage = 'Approving token spend...';
		mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatus.PENDING_APPROVAL,
			message: approvalMessage
		});

		await waitFor(() => {
			expect(screen.getByText(messages.pending)).toBeInTheDocument();
			expect(screen.getByText(approvalMessage)).toBeInTheDocument();
		});
	});

	it('should reset transaction store when modal is closed', async () => {
		render(TransactionModal, { props: { open: true, messages } });

		// Simulate closing the modal by changing the prop
		await render(TransactionModal, { props: { open: false, messages } });

		expect(resetSpy).toHaveBeenCalled();
	});
	it('should display a blockExplorerLink when it is provided', async () => {
		mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatus.SUCCESS,
			explorerLink: 'https://www.google.com'
		});

		render(TransactionModal, { props: { open: true, messages } });

		await waitFor(() => {
			const link = screen.getByText('View transaction on block explorer');
			expect(link).toHaveAttribute('href', 'https://www.google.com');
		});
	});
	it('should dispatch success event when transaction succeeds', async () => {
		const successMessage = 'Transaction succeeded';

		mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatus.SUCCESS,
			message: successMessage,
			hash: '0xMockTransactionHash'
		});

		render(TransactionModal, { props: { open: true, messages } });

		await waitFor(() => {
			expect(mockDispatch).toHaveBeenCalledWith('success');
		});
	});

	it('should display View Order button when newOrderHash and network are provided', async () => {
		mockTransactionStore.mockSetSubscribeValue({
			status: TransactionStatus.SUCCESS,
			newOrderHash: '0xMockOrderHash',
			network: 'testnet'
		});

		render(TransactionModal, { props: { open: true, messages } });

		await waitFor(() => {
			const viewOrderButton = screen.getByText('View Order');
			expect(viewOrderButton).toBeInTheDocument();
			expect(viewOrderButton.closest('a')).toHaveAttribute(
				'href',
				'/orders/testnet-0xMockOrderHash'
			);
		});
	});
});
