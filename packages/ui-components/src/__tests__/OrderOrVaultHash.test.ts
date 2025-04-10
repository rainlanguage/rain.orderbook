import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import OrderOrVaultHash from '../lib/components/OrderOrVaultHash.svelte';
import type { SgOrder, SgVault } from '@rainlanguage/orderbook';

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

	const mockVault = {
		id: '0xvault456'
	};

	const mockSubgraphName = 'test-subgraph';
	const mockUpdateFn = vi.fn();

	beforeEach(() => {
		mockUpdateFn.mockClear();
	});

	describe('Order rendering', () => {
		it('renders with active order', () => {
			const { getByTestId } = render(OrderOrVaultHash, {
				props: {
					type: 'orders',
					orderOrVault: mockOrder,
					network: mockSubgraphName,
					updateActiveNetworkAndOrderbook: mockUpdateFn
				}
			});

			const button = getByTestId('vault-order-input');
			const anchor = getByTestId('order-or-vault-hash');

			expect(button).toBeTruthy();
			expect(button.classList.toString()).toContain('text-white bg-green');
			expect(button.getAttribute('data-id')).toBe('0x123abc');

			expect(anchor).toBeTruthy();
			expect(anchor.getAttribute('href')).toBe('/orders/test-subgraph-0x123abc');

			expect(button.textContent).toBeDefined();
		});

		it('renders with inactive order', () => {
			const { getByTestId } = render(OrderOrVaultHash, {
				props: {
					type: 'orders',
					orderOrVault: mockInactiveOrder,
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
					orderOrVault: mockOrder,
					network: mockSubgraphName,
					updateActiveNetworkAndOrderbook: mockUpdateFn
				}
			});

			const button = getByTestId('vault-order-input');
			await fireEvent.click(button);

			expect(mockUpdateFn).toHaveBeenCalledWith(mockSubgraphName);
			expect(mockUpdateFn).toHaveBeenCalledTimes(1);
		});
	});

	describe('Vault rendering', () => {
		it('renders vault correctly', () => {
			const { getByTestId } = render(OrderOrVaultHash, {
				props: {
					type: 'vaults',
					orderOrVault: mockVault as unknown as SgVault,
					network: mockSubgraphName,
					updateActiveNetworkAndOrderbook: mockUpdateFn
				}
			});

			const button = getByTestId('vault-order-input');
			const anchor = getByTestId('order-or-vault-hash');

			expect(button).toBeTruthy();
			expect(button.getAttribute('data-id')).toBe('0xvault456');

			expect(anchor).toBeTruthy();
			expect(anchor.getAttribute('href')).toBe('/vaults/test-subgraph-0xvault456');
		});

		it('renders active order with appropriate styling', () => {
			const { getByTestId } = render(OrderOrVaultHash, {
				props: {
					type: 'orders',
					orderOrVault: mockOrder as unknown as SgOrder,
					network: mockSubgraphName,
					updateActiveNetworkAndOrderbook: mockUpdateFn
				}
			});

			const button = getByTestId('vault-order-input');
			expect(button.classList.toString()).toContain('text-white bg-green');
		});
		it('renders inactive order with appropriate styling', () => {
			const { getByTestId } = render(OrderOrVaultHash, {
				props: {
					type: 'orders',
					orderOrVault: mockInactiveOrder as unknown as SgOrder,
					network: mockSubgraphName,
					updateActiveNetworkAndOrderbook: mockUpdateFn
				}
			});

			const button = getByTestId('vault-order-input');
			expect(button.classList.toString()).toContain('bg-yellow-400');
		});
	});
});
