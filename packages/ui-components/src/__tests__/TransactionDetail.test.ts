import { describe, it, expect } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import TransactionDetail from '../lib/components/transactions/TransactionDetail.svelte';
import { writable } from 'svelte/store';
import type { TransactionStoreState } from '$lib/models/Transaction';
import { TransactionName, TransactionStatusMessage } from '$lib/types/transaction';

describe('TransactionDetail', () => {
	it('should render different status emojis for different states', () => {
		const states = [
			{ status: TransactionStatusMessage.PENDING_RECEIPT, emoji: '🔄' },
			{ status: TransactionStatusMessage.PENDING_SUBGRAPH, emoji: '📊' },
			{ status: TransactionStatusMessage.SUCCESS, emoji: '✅' },
			{ status: TransactionStatusMessage.ERROR, emoji: '❌' }
		] as const;

		states.forEach(({ status, emoji }) => {
			const state = writable<TransactionStoreState>({
				name: TransactionName.REMOVAL,
				status,
				explorerLink:
					'https://etherscan.io/tx/0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef'
			});

			const { container } = render(TransactionDetail, { state });
			expect(container.textContent).toContain(emoji);
		});
	});

	it('should render the explorer link', () => {
		const explorerLink =
			'https://etherscan.io/tx/0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef';
		const state = writable<TransactionStoreState>({
			name: TransactionName.REMOVAL,
			status: TransactionStatusMessage.PENDING_RECEIPT,
			explorerLink
		});

		render(TransactionDetail, { state });

		const link = screen.getByRole('link', { name: 'View transaction explorer' });
		expect(link).toBeInTheDocument();
		expect(link).toHaveAttribute('href', explorerLink);
		expect(link).toHaveAttribute('target', '_blank');
		expect(link).toHaveAttribute('rel', 'noopener noreferrer');
	});

	it('should update when the state changes', async () => {
		const state = writable<TransactionStoreState>({
			name: TransactionName.REMOVAL,
			status: TransactionStatusMessage.PENDING_RECEIPT,
			explorerLink:
				'https://etherscan.io/tx/0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef'
		});

		render(TransactionDetail, { state });

		expect(screen.getByText(TransactionName.REMOVAL)).toBeInTheDocument();
		expect(screen.getByText(`🔄 ${TransactionStatusMessage.PENDING_RECEIPT}`)).toBeInTheDocument();

		state.update((current) => ({
			...current,
			status: TransactionStatusMessage.PENDING_SUBGRAPH
		}));

		await waitFor(() => {
			expect(screen.getByText(TransactionName.REMOVAL)).toBeInTheDocument();
			expect(
				screen.getByText(`📊 ${TransactionStatusMessage.PENDING_SUBGRAPH}`)
			).toBeInTheDocument();
		});

		state.update((current) => ({
			...current,
			status: TransactionStatusMessage.SUCCESS
		}));

		await waitFor(() => {
			expect(screen.getByText(TransactionName.REMOVAL)).toBeInTheDocument();
			expect(screen.getByText(`✅ ${TransactionStatusMessage.SUCCESS}`)).toBeInTheDocument();
		});
	});

	it('should handle unknown status with a question mark emoji', () => {
		const state = writable<TransactionStoreState>({
			status: TransactionStatusMessage.ERROR,
			name: TransactionName.REMOVAL,
			explorerLink:
				'https://etherscan.io/tx/0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef'
		});

		render(TransactionDetail, { state });

		expect(screen.getByText(TransactionName.REMOVAL)).toBeInTheDocument();
		expect(screen.getByText(`❌ ${TransactionStatusMessage.ERROR}`)).toBeInTheDocument();
	});
});
