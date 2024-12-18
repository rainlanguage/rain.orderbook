import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/test/matchers';
import { mockIPC } from '@tauri-apps/api/mocks';
import type { OrderPerformance } from '$lib/typeshare/subgraphTypes';
import { QueryClient } from '@tanstack/svelte-query';
import OrderApy from '../lib/components/tables/OrderAPY.svelte';
import { bigintStringToPercentage } from '../lib/utils/number';

vi.mock('$lib/stores/settings', async (importOriginal) => {
	const { writable } = await import('svelte/store');
	const { mockSettingsStore } = await import('@rainlanguage/ui-components');

	const _activeOrderbook = writable();

	return {
		...((await importOriginal()) as object),
		settings: mockSettingsStore,
		subgraphUrl: writable('https://example.com'),
		activeOrderbook: {
			..._activeOrderbook,
			load: vi.fn(() => _activeOrderbook.set(true))
		}
	};
});

vi.mock('$lib/services/modal', async () => {
	return {
		handleDepositGenericModal: vi.fn(),
		handleDepositModal: vi.fn(),
		handleWithdrawModal: vi.fn()
	};
});

const mockOrderApy: OrderPerformance[] = [
	{
		orderId: '1',
		orderHash: '1',
		orderbook: '1',
		denominatedPerformance: {
			apy: '1200000000000000000',
			apyIsNeg: true,
			token: {
				id: 'output_token',
				address: 'output_token',
				name: 'output_token',
				symbol: 'output_token',
				decimals: '0'
			},
			netVol: '0',
			netVolIsNeg: false,
			startingCapital: '0'
		},
		startTime: 1,
		endTime: 2,
		inputsVaults: [],
		outputsVaults: []
	}
];

test('renders table with correct data', async () => {
	const queryClient = new QueryClient();

	mockIPC((cmd) => {
		if (cmd === 'order_performance') {
			return mockOrderApy[0];
		}
	});

	render(OrderApy, {
		context: new Map([['$$_queryClient', queryClient]]),
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		props: { id: '1' } as any
	});

	await waitFor(async () => {
		// get apy row
		const rows = screen.getAllByTestId('apy-field');

		// checking
		for (let i = 0; i < mockOrderApy.length; i++) {
			const display =
				(mockOrderApy[i].denominatedPerformance!.apyIsNeg ? '-' : '') +
				bigintStringToPercentage(mockOrderApy[i].denominatedPerformance!.apy, 18, 5);
			expect(rows[i]).toHaveTextContent(display);
		}
	});
});
