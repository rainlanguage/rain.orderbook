import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import TransactionDetail from '../lib/components/transactions/TransactionDetail.svelte';
import { writable } from 'svelte/store';
import type { TransactionState } from '../lib/models/Transaction';
import { TransactionStatusMessage } from '../lib/types/transaction';

describe('TransactionDetail', () => {
	it('should render the status emoji and message', () => {
		const state = writable<TransactionState>({
			status: TransactionStatusMessage.IDLE,
			message: 'Starting order removal',
			explorerLink:
				'https://etherscan.io/tx/0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef'
		});

		render(TransactionDetail, { state });

		expect(screen.getByText('⏳ Starting order removal')).toBeInTheDocument();
	});

	it('should render different status emojis for different states', () => {
		const states = [
			{ status: TransactionStatusMessage.IDLE, emoji: '⏳' },
			{ status: TransactionStatusMessage.PENDING_REMOVE_ORDER, emoji: '🔄' },
			{ status: TransactionStatusMessage.PENDING_SUBGRAPH, emoji: '📊' },
			{ status: TransactionStatusMessage.SUCCESS, emoji: '✅' },
			{ status: TransactionStatusMessage.ERROR, emoji: '❌' }
		] as const;

		states.forEach(({ status, emoji }) => {
			const state = writable<TransactionState>({
				status,
				message: 'Test message',
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
		const state = writable<TransactionState>({
			status: TransactionStatusMessage.IDLE,
			message: 'Test message',
			explorerLink
		});

		render(TransactionDetail, { state });

		const link = screen.getByRole('link', { name: 'View transaction on explorer' });
		expect(link).toBeInTheDocument();
		expect(link).toHaveAttribute('href', explorerLink);
		expect(link).toHaveAttribute('target', '_blank');
		expect(link).toHaveAttribute('rel', 'noopener noreferrer');
	});

	it('should update when the state changes', async () => {
		const state = writable<TransactionState>({
			status: TransactionStatusMessage.IDLE,
			message: 'Initial message',
			explorerLink:
				'https://etherscan.io/tx/0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef'
		});

		render(TransactionDetail, { state });

		expect(screen.getByText('⏳ Initial message')).toBeInTheDocument();

		state.update((current) => ({
			...current,
			status: TransactionStatusMessage.SUCCESS,
			message: 'Updated message'
		}));
		await waitFor(() => {
			expect(screen.getByText('✅ Updated message')).toBeInTheDocument();
		});
	});

	it('should handle unknown status with a question mark emoji', () => {
		const state = writable<TransactionState>({
			status: TransactionStatusMessage.ERROR,
			message: 'Unknown status message',
			explorerLink:
				'https://etherscan.io/tx/0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef'
		});

		render(TransactionDetail, { state });

		expect(screen.getByText('❌ Unknown status message')).toBeInTheDocument();
	});
});
