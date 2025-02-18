import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import DepositInput from '../lib/components/deployment/DepositInput.svelte';
import type { GuiDepositCfg } from '@rainlanguage/orderbook/js_api';
import type { ComponentProps } from 'svelte';

type DepositInputProps = ComponentProps<DepositInput>;

describe('DepositInput', () => {
	const mockGui = {
		getTokenInfo: vi.fn(),
		isDepositPreset: vi.fn(),
		saveDeposit: vi.fn(),
		getDeposits: vi.fn().mockReturnValue([{ token: 'output', amount: '10', address: '0x1234' }])
	};

	const mockDeposit: GuiDepositCfg = {
		token: { address: '0x123', key: 'TEST', symbol: 'TEST' },
		presets: ['100', '200', '300']
	} as unknown as GuiDepositCfg;

	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('renders token name and presets', async () => {
		const { getByText } = render(DepositInput, {
			props: {
				deposit: mockDeposit,
				gui: {
					...mockGui,
					getTokenInfo: vi.fn().mockReturnValue({ name: 'Test Token', symbol: 'TEST' })
				},
				handleUpdateGuiState: vi.fn()
			} as unknown as DepositInputProps
		});
		await waitFor(() => {
			expect(getByText(`Deposit amount (${mockDeposit.token?.symbol})`)).toBeTruthy();
			expect(getByText('100')).toBeTruthy();
			expect(getByText('200')).toBeTruthy();
			expect(getByText('300')).toBeTruthy();
		});
	});

	it('handles preset button clicks', async () => {
		const { getByText } = render(DepositInput, {
			props: {
				deposit: mockDeposit,
				gui: mockGui,
				handleUpdateGuiState: vi.fn()
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
				gui: mockGui,
				handleUpdateGuiState: vi.fn()
			} as unknown as DepositInputProps
		});

		const input = getByPlaceholderText('Enter deposit amount');
		await fireEvent.input(input, { target: { value: '150' } });

		expect(mockGui.saveDeposit).toHaveBeenCalledWith('TEST', '150');
	});
});
