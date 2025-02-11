import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import OrderVaultInfo from '../lib/components/OrderVaultInfo.svelte';
import type { Vault } from '@rainlanguage/orderbook/js_api';
// Mock the $app/navigation module
vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

describe('OrderVaultInfo', () => {
	const mockVault = {
		id: '123',
		vaultId: '1000',
		balance: '1000000000000000000',
		token: {
			name: 'Test Token',
			symbol: 'TEST',
			decimals: '18'
		}
	} as unknown as Vault;

	it('should display vault name and address', () => {
		render(OrderVaultInfo, {
			props: {
				tokenVault: mockVault,
				subgraphName: 'test'
			}
		});

		expect(screen.getByText('Test Token (TEST)')).toBeInTheDocument();
		expect(screen.getByText('1')).toBeInTheDocument();
	});
});
