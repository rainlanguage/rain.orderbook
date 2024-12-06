/* eslint-disable @typescript-eslint/no-unused-vars */
import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, vi, type Mock } from 'vitest';
import { expect } from '../lib/test/matchers';
import OrderDetail from './OrderDetail.test.svelte';
import type { Order } from '@rainlanguage/orderbook/js_api';

const { mockWalletAddressMatchesOrBlankStore } = await vi.hoisted(
	() => import('../lib/__mocks__/stores')
);

vi.mock('svelte-codemirror-editor', async () => {
	const MockCodeMirrorRainlang = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { default: MockCodeMirrorRainlang };
});

vi.mock('codemirror-rainlang', () => ({
	RainlangLR: {
		fromText: vi.fn()
	}
}));

const mockOrder: Order = {
	id: 'mockId',
	owner: 'mockOwner',
	orderHash: 'mockOrderHash',
	active: true,
	meta: '0x',
	timestampAdded: '1234567890',
	orderbook: { id: '1' },
	inputs: [],
	outputs: []
} as unknown as Order;

vi.mock('@tanstack/svelte-query');

describe('OrderDetail Component', () => {
	it('shows the correct empty message when the query returns no data', async () => {
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		mockQuery.createQuery = vi.fn((__options, _queryClient) => ({
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: null,
					status: 'success',
					isFetching: false
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		render(OrderDetail, {
			props: {
				id: 'mockId',
				subgraphUrl: 'https://example.com'
			}
		});

		await waitFor(() => expect(screen.getByText('Order not found')).toBeInTheDocument());
	});

	it('shows remove button if owner wallet matches and order is active', async () => {
		const handleOrderRemoveModal = vi.fn();
		const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
		mockQuery.createQuery = vi.fn((__options, _queryClient) => ({
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			subscribe: (fn: (value: any) => void) => {
				fn({
					data: mockOrder,
					status: 'success',
					isFetching: false,
					refetch: () => {}
				});
				return { unsubscribe: () => {} };
			}
		})) as Mock;

		mockWalletAddressMatchesOrBlankStore.mockSetSubscribeValue(() => true);

		render(OrderDetail, {
			props: {
				id: mockOrder.id,
				subgraphUrl: 'https://example.com',
				walletAddressMatchesOrBlank: mockWalletAddressMatchesOrBlankStore,
				handleOrderRemoveModal
			}
		});

		await waitFor(() => {
			expect(screen.queryByText('Remove')).toBeInTheDocument();
			expect(handleOrderRemoveModal).not.toHaveBeenCalled();
		});
	});

	it('does not render the remove button if conditions are not met', async () => {
		mockWalletAddressMatchesOrBlankStore.mockSetSubscribeValue(() => false);

		render(OrderDetail, {
			props: {
				id: mockOrder.id,
				subgraphUrl: 'https://example.com',
				walletAddressMatchesOrBlank: mockWalletAddressMatchesOrBlankStore,
				handleOrderRemoveModal: vi.fn()
			}
		});

		await waitFor(() => {
			expect(screen.queryByText('Remove')).not.toBeInTheDocument();
		});
	});
});
