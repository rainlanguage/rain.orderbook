/* eslint-disable @typescript-eslint/no-unused-vars */
import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi, type Mock } from 'vitest';
import { expect } from '../lib/test/matchers';
import { QueryClient } from '@tanstack/svelte-query';
import OrderDetail from '../lib/components/detail/OrderDetail.svelte';
import { getOrder, type Order } from '@rainlanguage/orderbook/js_api';
import { readable, writable } from 'svelte/store';

const { mockWalletAddressMatchesOrBlankStore } = await vi.hoisted(
	() => import('../lib/__mocks__/stores')
);

const mockOrder: Order = {
	id: '0xabc...bcdef',
	owner: '0x1111111111111111111111111111111111111111',
	meta: '0x',
	timestampAdded: '1234567890',
	orderHash: '0xabcdef1234567890',
	expression: '0x',
	interpreter: '0x',
	dispatch: '0x',
	active: true,
	orderbook: { id: '1' },
	inputs: [
		{
			token: {
				id: '1',
				address: '0x1234567890abcdef',
				name: 'Token A',
				symbol: 'TKA',
				decimals: '18'
			},
			balance: '1000000000000000000',
			vaultId: '1'
		}
	],
	outputs: [
		{
			token: {
				id: '2',
				address: '0xfedcba0987654321',
				name: 'Token B',
				symbol: 'TKB',
				decimals: '18'
			},
			balance: '2000000000000000000',
			vaultId: '2'
		}
	]
};

vi.mock('@tanstack/svelte-query');

const MockComponent = await vi.hoisted(() => import('../lib/__mocks__/MockComponent.svelte'));

vi.mock('../lib/components/CodeMirrorRainlang.svelte', async (importOriginal) => {
	return {
		default: MockComponent.default
	};
});

vi.mock('../lib/components/charts/TanstackLightweightChartLine.svelte', async (importOriginal) => {
	return {
		default: MockComponent.default
	};
});

vi.mock('../lib/components/detail/TanstackOrderQuote.svelte', async (importOriginal) => {
	return {
		default: MockComponent.default
	};
});

vi.mock('../lib/components/charts/OrderTradesChart.svelte', async (importOriginal) => {
	return {
		default: MockComponent.default
	};
});

vi.mock('../lib/components/tables/OrderTradesListTable.svelte', async (importOriginal) => {
	return {
		default: MockComponent.default
	};
});

vi.mock('lightweight-charts', async (importOriginal) => ({
	...((await importOriginal()) as object),
	createChart: vi.fn(() => ({
		addLineSeries: vi.fn(),
		remove(): void {},
		applyOptions: vi.fn()
	}))
}));

describe('OrderDetail Component', () => {
	it('shows the correct empty message when the query returns no data', async () => {
		const codeMirrorTheme = writable({});

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
				id: 'order1',
				rpcUrl: 'https://example.com',
				subgraphUrl: 'https://example.com',
				colorTheme: writable('light'),
				codeMirrorTheme,
				lightweightChartsTheme: {}
			}
		});

		await waitFor(() => expect(screen.getByText('Order not found')).toBeInTheDocument());
	});

	it('shows remove button if owner wallet matches and order is active, opens correct modal', async () => {
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
				rpcUrl: 'https://example.com',
				subgraphUrl: 'https://example.com',
				colorTheme: writable('light'),
				codeMirrorTheme: writable({}),
				lightweightChartsTheme: {},
				walletAddressMatchesOrBlank: mockWalletAddressMatchesOrBlankStore,
				handleOrderRemoveModal: vi.fn()
			}
		});

		mockWalletAddressMatchesOrBlankStore.set(() => true);

		await waitFor(() => {
			expect(screen.queryByText('Remove')).toBeInTheDocument();
		});

		screen.getByText('Remove').click();

		await waitFor(() => {
			expect(handleOrderRemoveModal).toHaveBeenCalledWith(mockOrder, expect.any(Function));
		});
	});
});
