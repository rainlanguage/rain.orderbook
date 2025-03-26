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
		testId: 'deposit-button'
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

	it('emits deposit event with vault when clicked with deposit action', async () => {
		const user = userEvent.setup();
		const mockOnDeposit = vi.fn();

		const { component } = render(VaultActionButton, defaultProps);

		component.$on('deposit', mockOnDeposit);

		const button = screen.getByTestId('deposit-button');
		await user.click(button);

		expect(mockOnDeposit).toHaveBeenCalled();

		expect(mockOnDeposit.mock.calls[0][0].detail).toEqual(
			expect.objectContaining({
				vault: mockVault
			})
		);
	});

	it('emits withdraw event with vault when clicked with withdraw action', async () => {
		const user = userEvent.setup();
		const mockOnWithdraw = vi.fn();

		const withdrawProps = {
			...defaultProps,
			action: 'withdraw' as const,
			testId: 'withdraw-button'
		};

		const { component } = render(VaultActionButton, withdrawProps);

		component.$on('withdraw', mockOnWithdraw);

		const button = screen.getByTestId('withdraw-button');
		await user.click(button);

		expect(mockOnWithdraw).toHaveBeenCalled();

		expect(mockOnWithdraw.mock.calls[0][0].detail).toEqual(
			expect.objectContaining({
				vault: mockVault
			})
		);
	});

	it('includes onSuccess callback in event payload if provided', async () => {
		const user = userEvent.setup();
		const mockOnDeposit = vi.fn();
		const mockOnSuccess = vi.fn();

		const propsWithSuccess = {
			...defaultProps,
			onSuccess: mockOnSuccess
		};

		const { component } = render(VaultActionButton, propsWithSuccess);

		component.$on('deposit', mockOnDeposit);

		const button = screen.getByTestId('deposit-button');
		await user.click(button);

		expect(mockOnDeposit).toHaveBeenCalled();

		const eventDetail = mockOnDeposit.mock.calls[0][0].detail;
		expect(eventDetail.onSuccess).toBe(mockOnSuccess);

		eventDetail.onSuccess();

		expect(mockOnSuccess).toHaveBeenCalled();
	});

	it('is disabled when disabled prop is true', async () => {
		const user = userEvent.setup();
		const mockOnDeposit = vi.fn();

		const disabledProps = {
			...defaultProps,
			disabled: true
		};

		const { component } = render(VaultActionButton, disabledProps);
		component.$on('deposit', mockOnDeposit);

		const button = screen.getByTestId('deposit-button');
		expect(button).toBeDisabled();

		await user.click(button);

		expect(mockOnDeposit).not.toHaveBeenCalled();
	});
});
