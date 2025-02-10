import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import ButtonVaultLink from '../lib/components/ButtonVaultLink.svelte';
import type { Vault } from '@rainlanguage/orderbook/js_api';

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
	} as unknown as Vault;

	it('should render vault information correctly', () => {
		render(ButtonVaultLink, {
			props: {
				tokenVault: mockVault,
				subgraphName: 'test'
			}
		});

		const vaultLink = screen.getByTestId('vault-link');
		expect(vaultLink).toBeTruthy();
		expect(vaultLink).toHaveTextContent('Test Token');
		expect(vaultLink).toHaveTextContent('TEST');
	});
});
