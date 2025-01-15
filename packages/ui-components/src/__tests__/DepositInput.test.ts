import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import DepositInput from '../lib/components/deployment/DepositInput.svelte';
import type { GuiDeposit } from '@rainlanguage/orderbook/js_api';
import type { ComponentProps } from 'svelte';

type DepositInputProps = ComponentProps<DepositInput>;

describe('DepositInput', () => {
	const mockGui = {
		isDepositPreset: vi.fn(),
		saveDeposit: vi.fn(),
		getDeposits: vi.fn().mockReturnValue([{ token: 'output', amount: '10', address: '0x1234' }])
	};

	const mockDeposit: GuiDeposit = {
		token: { address: '0x123', key: 'TEST', symbol: 'Test Token' },
		presets: ['100', '200', '300']
	} as unknown as GuiDeposit;

	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('renders token name and presets', () => {
		const { getByText } = render(DepositInput, {
			props: {
				deposit: mockDeposit,
				gui: mockGui
			} as unknown as DepositInputProps
		});

		expect(getByText(`Deposit amount (${mockDeposit.token?.symbol})`)).toBeTruthy();
		expect(getByText('100')).toBeTruthy();
		expect(getByText('200')).toBeTruthy();
		expect(getByText('300')).toBeTruthy();
	});

	it('handles preset button clicks', async () => {
		const { getByText } = render(DepositInput, {
			props: {
				deposit: mockDeposit,
				gui: mockGui
			} as unknown as DepositInputProps
		});

		await fireEvent.click(getByText('100'));
		expect(mockGui.saveDeposit).toHaveBeenCalledWith('TEST', '100');
	});

	it('handles custom input changes', async () => {
		mockGui.isDepositPreset.mockReturnValue(false);

		const { getByPlaceholderText } = render(DepositInput, {
			props: {
				deposit: mockDeposit,
				gui: mockGui
			} as unknown as DepositInputProps
		});

		const input = getByPlaceholderText('Enter deposit amount');
		await fireEvent.input(input, { target: { value: '150' } });

		expect(mockGui.saveDeposit).toHaveBeenCalledWith('TEST', '150');
	});
});
