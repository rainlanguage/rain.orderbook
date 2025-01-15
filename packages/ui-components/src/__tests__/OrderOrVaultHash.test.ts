import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import OrderOrVaultHash from '../lib/components/OrderOrVaultHash.svelte';
import { goto } from '$app/navigation';

vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

describe('OrderOrVaultHash', () => {
	const mockOrder = {
		id: '123',
		orderHash: '0x123abc',
		active: true
	};

	const mockInactiveOrder = {
		...mockOrder,
		active: false
	};

	const mockSubgraphName = 'test-subgraph';
	const mockUpdateFn = vi.fn();

	it('renders with active order', () => {
		const { getByTestId } = render(OrderOrVaultHash, {
			props: {
				type: 'orders',
				order: mockOrder,
				network: mockSubgraphName,
				updateActiveNetworkAndOrderbook: mockUpdateFn
			}
		});

		const button = getByTestId('vault-order-input');
		expect(button).toBeTruthy();
		expect(button.classList.toString()).toContain('text-white bg-green');
		expect(button.getAttribute('data-id')).toBe('123');
	});

	it('renders with inactive order', () => {
		const { getByTestId } = render(OrderOrVaultHash, {
			props: {
				type: 'orders',
				order: mockInactiveOrder,
				network: mockSubgraphName,
				updateActiveNetworkAndOrderbook: mockUpdateFn
			}
		});

		const button = getByTestId('vault-order-input');
		expect(button.classList.toString()).toContain('text-white bg-yellow');
	});

	it('handles click event correctly', async () => {
		const { getByTestId } = render(OrderOrVaultHash, {
			props: {
				type: 'orders',
				order: mockOrder,
				network: mockSubgraphName,
				updateActiveNetworkAndOrderbook: mockUpdateFn
			}
		});

		const button = getByTestId('vault-order-input');
		await fireEvent.click(button);

		expect(mockUpdateFn).toHaveBeenCalledWith(mockSubgraphName);
		expect(goto).toHaveBeenCalledWith(`/orders/${mockSubgraphName}-${mockOrder.id}`);
	});
});
