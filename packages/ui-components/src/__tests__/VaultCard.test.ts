/* eslint-disable @typescript-eslint/no-explicit-any */
import { render, screen } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import VaultCard from '../lib/components/VaultCard.svelte';
import type { RaindexVault } from '@rainlanguage/orderbook';
import userEvent from '@testing-library/user-event';

// Mock the navigation
vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

const mockVault: RaindexVault = {
	id: '0x1234567890abcdef1234567890abcdef12345678',
	token: {
		symbol: 'ETH',
		name: 'Ethereum',
		address: '0x0000000000000000000000000000000000000000',
		decimals: 18
	},
	formattedBalance: '1.5'
} as unknown as RaindexVault;

describe('VaultCard', () => {
	it('renders vault information correctly', () => {
		render(VaultCard, {
			vault: mockVault,
			chainId: 1
		});

		expect(screen.getByTestId('vault-card')).toBeInTheDocument();
		expect(screen.getByText('ETH')).toBeInTheDocument();
		expect(screen.getByText('1.5')).toBeInTheDocument();
	});

	it('navigates to vault details when clicked', async () => {
		const { goto } = await import('$app/navigation');

		render(VaultCard, {
			vault: mockVault,
			chainId: 1
		});

		const vaultCard = screen.getByTestId('vault-card');
		await userEvent.click(vaultCard);

		expect(goto).toHaveBeenCalledWith('/vaults/1-0x1234567890abcdef1234567890abcdef12345678');
	});

	it('displays different token symbols correctly', () => {
		const daiVault = {
			...mockVault,
			token: {
				...mockVault.token,
				symbol: 'DAI'
			},
			formattedBalance: '2500.0'
		} as unknown as RaindexVault;

		render(VaultCard, {
			vault: daiVault,
			chainId: 137
		});

		expect(screen.getByText('DAI')).toBeInTheDocument();
		expect(screen.getByText('2500.0')).toBeInTheDocument();
	});

	it('navigates with correct chain ID', async () => {
		const { goto } = await import('$app/navigation');

		render(VaultCard, {
			vault: mockVault,
			chainId: 137 // Polygon
		});

		const vaultCard = screen.getByTestId('vault-card');
		await userEvent.click(vaultCard);

		expect(goto).toHaveBeenCalledWith('/vaults/137-0x1234567890abcdef1234567890abcdef12345678');
	});

	it('has proper accessibility attributes', () => {
		render(VaultCard, {
			vault: mockVault,
			chainId: 1
		});

		const button = screen.getByTestId('vault-card');
		expect(button).toHaveAttribute('type', 'button');
		expect(button.tagName).toBe('BUTTON');
	});

	it('displays formatted balance correctly', () => {
		const vaultWithLongBalance = {
			...mockVault,
			formattedBalance: '1,234,567.89'
		} as unknown as RaindexVault;

		render(VaultCard, {
			vault: vaultWithLongBalance,
			chainId: 1
		});

		expect(screen.getByText('1,234,567.89')).toBeInTheDocument();
	});
});
