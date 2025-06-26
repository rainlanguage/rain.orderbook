import type { RaindexOrder, RaindexOrderAsIO, RaindexVault } from '@rainlanguage/orderbook';
import fc from 'fast-check';
import { test } from '@fast-check/vitest';

type OrderOrVault = RaindexOrder | RaindexOrderAsIO | RaindexVault;

function isOrder(obj: OrderOrVault): obj is RaindexOrder | RaindexOrderAsIO {
	return obj && 'orderHash' in obj;
}
/**
 * Constructs a link path for an order or vault based on its type and network
 * @param orderOrVault - The order or vault object
 * @param type - The type of resource ('orders' or 'vaults')
 * @param network - The network name
 * @returns The constructed link path
 */
export function constructHashLink(
	orderOrVault: OrderOrVault,
	type: 'orders' | 'vaults',
	network: string
): string {
	if (!orderOrVault) {
		return `/${type}/${network}`;
	}

	const slug = isOrder(orderOrVault) ? orderOrVault.orderHash : (orderOrVault as RaindexVault)?.id;

	return `/${type}/${network}-${slug || ''}`;
}

/**
 * Determines if an order or vault is active
 * @param orderOrVault - The order or vault object
 * @returns True if the order is active, false otherwise
 */
export function isOrderOrVaultActive(orderOrVault: OrderOrVault): boolean {
	const _isOrder = isOrder(orderOrVault);
	return _isOrder ? (orderOrVault as RaindexOrderAsIO).active : false;
}

/**
 * Extracts the hash value from an order or vault
 * @param orderOrVault - The order or vault object
 * @returns The hash value
 */
export function extractHash(orderOrVault: OrderOrVault): string {
	const _isOrder = isOrder(orderOrVault);
	return _isOrder
		? (orderOrVault as RaindexOrder).orderHash
		: (orderOrVault as RaindexVault)?.id || '';
}

if (import.meta.vitest) {
	const { expect, it, describe } = import.meta.vitest;

	describe('constructHashLink', () => {
		test.prop([
			fc.record({
				orderHash: fc.string(),
				active: fc.boolean()
			}),
			fc.oneof(fc.constant('orders'), fc.constant('vaults')),
			fc.string()
		])('constructs correct link for orders', (order, type, network) => {
			const result = constructHashLink(order as RaindexOrder, type, network);
			expect(result).toBe(`/${type}/${network}-${order.orderHash}`);
		});

		test.prop([
			fc.record({
				id: fc.string(),
				owner: fc.string()
			}),
			fc.oneof(fc.constant('orders'), fc.constant('vaults')),
			fc.string()
		])('constructs correct link for vaults', (vault, type, network) => {
			const result = constructHashLink(vault as RaindexVault, type, network);
			expect(result).toBe(`/${type}/${network}-${vault.id}`);
		});
	});

	describe('isOrderOrVaultActive', () => {
		test.prop([
			fc.record({
				orderHash: fc.string(),
				active: fc.boolean()
			})
		])('returns correct active status for orders', (order) => {
			const result = isOrderOrVaultActive(order as RaindexOrderAsIO);
			expect(result).toBe(order.active);
		});

		test.prop([
			fc.record({
				id: fc.string(),
				owner: fc.string()
			})
		])('returns false for vaults', (vault) => {
			const result = isOrderOrVaultActive(vault as RaindexVault);
			expect(result).toBe(false);
		});
	});

	describe('extractHash', () => {
		test.prop([
			fc.record({
				orderHash: fc.string(),
				active: fc.boolean()
			})
		])('extracts hash from orders', (order) => {
			const result = extractHash(order as RaindexOrder);
			expect(result).toBe(order.orderHash);
		});

		test.prop([
			fc.record({
				id: fc.string(),
				owner: fc.string()
			})
		])('extracts hash from vaults', (vault) => {
			const result = extractHash(vault as RaindexVault);
			expect(result).toBe(vault.id);
		});

		it('handles undefined vault id', () => {
			// Create a partial vault object with undefined id
			const vault = { id: undefined } as unknown as RaindexVault;
			const result = extractHash(vault);
			expect(result).toBe('');
		});
	});
}
