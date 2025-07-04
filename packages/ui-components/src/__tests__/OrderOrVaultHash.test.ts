import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import OrderOrVaultHash from '../lib/components/OrderOrVaultHash.svelte';
import type { RaindexOrder, RaindexVault } from '@rainlanguage/orderbook';

vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

describe('OrderOrVaultHash', () => {
	const mockOrder = {
		id: '123',
		orderHash: '0x123abc',
		active: true
	} as unknown as RaindexOrder;

	const mockInactiveOrder = {
		...mockOrder,
		active: false
	} as unknown as RaindexOrder;

	const mockVault = {
		id: '0xvault456'
	} as unknown as RaindexVault;

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
					chainId: 1,
					orderbookAddress: '0x234567'
					// updateActiveNetworkAndOrderbook: mockUpdateFn
				}
			});

			const button = getByTestId('vault-order-input');
			const anchor = getByTestId('order-or-vault-hash');

			expect(button).toBeTruthy();
			expect(button.classList.toString()).toContain('text-white bg-green');
			expect(button.getAttribute('data-id')).toBe('0x123abc');

			expect(anchor).toBeTruthy();
			expect(anchor.getAttribute('href')).toBe('/orders/1-0x234567-0x123abc');

			expect(button.textContent).toBeDefined();
		});

		it('renders with inactive order', () => {
			const { getByTestId } = render(OrderOrVaultHash, {
				props: {
					type: 'orders',
					orderOrVault: mockInactiveOrder,
					chainId: 1,
					orderbookAddress: '0x234567'
					// updateActiveNetworkAndOrderbook: mockUpdateFn
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
					chainId: 1,
					orderbookAddress: '0x234567'
					// updateActiveNetworkAndOrderbook: mockUpdateFn
				}
			});

			const button = getByTestId('vault-order-input');
			await fireEvent.click(button);

			// expect(mockUpdateFn).toHaveBeenCalledWith(mockSubgraphName);
			// expect(mockUpdateFn).toHaveBeenCalledTimes(1);
		});
	});

	describe('Vault rendering', () => {
		it('renders vault correctly', () => {
			const { getByTestId } = render(OrderOrVaultHash, {
				props: {
					type: 'vaults',
					orderOrVault: mockVault,
					chainId: 1,
					orderbookAddress: '0x234567'
					// updateActiveNetworkAndOrderbook: mockUpdateFn
				}
			});

			const button = getByTestId('vault-order-input');
			const anchor = getByTestId('order-or-vault-hash');

			expect(button).toBeTruthy();
			expect(button.getAttribute('data-id')).toBe('0xvault456');

			expect(anchor).toBeTruthy();
			expect(anchor.getAttribute('href')).toBe('/vaults/1-0x234567-0xvault456');
		});

		it('renders active order with appropriate styling', () => {
			const { getByTestId } = render(OrderOrVaultHash, {
				props: {
					type: 'orders',
					orderOrVault: mockOrder as unknown as RaindexOrder,
					chainId: 1,
					orderbookAddress: '0x234567'
					// updateActiveNetworkAndOrderbook: mockUpdateFn
				}
			});

			const button = getByTestId('vault-order-input');
			expect(button.classList.toString()).toContain('text-white bg-green');
		});
		it('renders inactive order with appropriate styling', () => {
			const { getByTestId } = render(OrderOrVaultHash, {
				props: {
					type: 'orders',
					orderOrVault: mockInactiveOrder as unknown as RaindexOrder,
					chainId: 1,
					orderbookAddress: '0x234567'
					// updateActiveNetworkAndOrderbook: mockUpdateFn
				}
			});

			const button = getByTestId('vault-order-input');
			expect(button.classList.toString()).toContain('bg-yellow-400');
		});
	});
});
