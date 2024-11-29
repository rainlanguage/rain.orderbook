import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import ButtonVaultLink from '../lib/components/ButtonVaultLink.svelte';
import * as navigation from '$app/navigation';
import { userEvent } from '@testing-library/user-event';
import type { Vault } from '../../dist/typeshare/subgraphTypes';

// Mock the $app/navigation module
vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

describe('ButtonVaultLink', () => {
	const mockVault = {
		id: '123',
		vaultId: '1000',
		balance: '1000000000000000000',
		token: {
			name: 'Test Token',
			symbol: 'TEST',
			decimals: '18'
		}
	};

	it('should navigate to vault details page when clicked', async () => {
		render(ButtonVaultLink, {
			props: {
				tokenVault: mockVault as unknown as Vault
			}
		});

		const vaultLink = screen.getByTestId('vault-link');
		expect(vaultLink).toBeTruthy();
		await userEvent.click(vaultLink);
		expect(navigation.goto).toHaveBeenCalledWith(`/vaults/${mockVault.id}`);
	});
});
