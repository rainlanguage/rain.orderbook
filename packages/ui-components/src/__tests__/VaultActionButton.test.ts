import { render, screen } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import userEvent from '@testing-library/user-event';
import VaultActionButton from '../lib/components/actions/VaultActionButton.svelte';
import type { SgVault } from '@rainlanguage/orderbook/js_api';

describe('VaultActionButton', () => {
	const mockVault: SgVault = {
		id: '1',
		vaultId: '0xabc',
		owner: '0x123',
		token: {
			id: '0x456',
			address: '0x456',
			name: 'USDC coin',
			symbol: 'USDC',
			decimals: '6'
		},
		balance: '100000000000',
		ordersAsInput: [],
		ordersAsOutput: [],
		balanceChanges: [],
		orderbook: {
			id: '0x00'
		}
	};

	const defaultProps = {
		action: 'deposit' as const,
		vault: mockVault,
		testId: 'deposit-button',
		onDepositOrWithdraw: vi.fn()
	};

	beforeEach(() => {
		vi.resetAllMocks();
	});

	it('renders with deposit action and displays correct icon', () => {
		render(VaultActionButton, defaultProps);

		const button = screen.getByTestId('deposit-button');
		expect(button).toBeInTheDocument();
	});

	it('renders with withdraw action and displays correct icon', () => {
		const withdrawProps = {
			...defaultProps,
			action: 'withdraw' as const,
			testId: 'withdraw-button'
		};

		render(VaultActionButton, withdrawProps);

		const button = screen.getByTestId('withdraw-button');
		expect(button).toBeInTheDocument();
	});

	it('displays label when provided', () => {
		const propsWithLabel = {
			...defaultProps,
			label: 'Test Label'
		};

		render(VaultActionButton, propsWithLabel);
		expect(screen.getByText('Test Label')).toBeInTheDocument();
	});

	it('calls onDepositOrWithdraw callback with vault when clicked with deposit action', async () => {
		const user = userEvent.setup();
		const mockOnDepositOrWithdraw = vi.fn();

		const props = {
			...defaultProps,
			onDepositOrWithdraw: mockOnDepositOrWithdraw
		};

		render(VaultActionButton, props);

		const button = screen.getByTestId('deposit-button');
		await user.click(button);

		expect(mockOnDepositOrWithdraw).toHaveBeenCalledWith(mockVault);
	});

	it('calls onDepositOrWithdraw callback with vault when clicked with withdraw action', async () => {
		const user = userEvent.setup();
		const mockOnDepositOrWithdraw = vi.fn();

		const withdrawProps = {
			...defaultProps,
			action: 'withdraw' as const,
			testId: 'withdraw-button',
			onDepositOrWithdraw: mockOnDepositOrWithdraw
		};

		render(VaultActionButton, withdrawProps);

		const button = screen.getByTestId('withdraw-button');
		await user.click(button);

		expect(mockOnDepositOrWithdraw).toHaveBeenCalledWith(mockVault);
	});

	it('is disabled when disabled prop is true', async () => {
		const user = userEvent.setup();
		const mockOnDepositOrWithdraw = vi.fn();

		const disabledProps = {
			...defaultProps,
			disabled: true,
			onDepositOrWithdraw: mockOnDepositOrWithdraw
		};

		render(VaultActionButton, disabledProps);

		const button = screen.getByTestId('deposit-button');
		expect(button).toBeDisabled();

		await user.click(button);

		expect(mockOnDepositOrWithdraw).not.toHaveBeenCalled();
	});
});
