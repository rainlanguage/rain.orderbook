import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import OrderVaultInfo from '../lib/components/OrderVaultInfo.svelte';
import * as navigation from '$app/navigation';
import { userEvent } from '@testing-library/user-event';
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

	it('should navigate to vault details page when clicked', async () => {
		render(OrderVaultInfo, {
			props: {
				tokenVault: mockVault,
				subgraphName: 'test'
			}
		});

		const vaultLink = screen.getByTestId('vault-link');
		expect(vaultLink).toBeTruthy();
		await userEvent.click(vaultLink);
		expect(navigation.goto).toHaveBeenCalledWith(`/vaults/test-${mockVault.id}`);
	});
});
