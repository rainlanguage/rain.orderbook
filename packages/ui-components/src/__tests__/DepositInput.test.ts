import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import DepositInput from '../lib/components/deployment/DepositInput.svelte';
import type { GuiDepositCfg } from '@rainlanguage/orderbook';
import type { ComponentProps } from 'svelte';

type DepositInputProps = ComponentProps<DepositInput>;

describe('DepositInput', () => {
	let mockStateUpdateCallback: Mock;

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
		mockStateUpdateCallback = vi.fn();
		mockGui.saveDeposit.mockImplementation(() => {
			mockStateUpdateCallback();
		});
		vi.clearAllMocks();
	});

	it('renders token name and presets', async () => {
		const { getByText } = render(DepositInput, {
			props: {
				deposit: mockDeposit,
				gui: {
					...mockGui,
					getTokenInfo: vi.fn().mockReturnValue({ name: 'Test Token', symbol: 'TEST' })
				}
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
				gui: mockGui
			} as unknown as DepositInputProps
		});

		await fireEvent.click(getByText('100'));
		expect(mockGui.saveDeposit).toHaveBeenCalledWith('TEST', '100');
	});

	it('handles custom input changes and triggers state update', async () => {
		mockGui.isDepositPreset.mockReturnValue(false);

		const { getByPlaceholderText } = render(DepositInput, {
			props: {
				deposit: mockDeposit,
				gui: mockGui,
				onStateUpdate: mockStateUpdateCallback
			} as unknown as DepositInputProps
		});

		const input = getByPlaceholderText('Enter deposit amount');
		await fireEvent.input(input, { target: { value: '150' } });

		expect(mockGui.saveDeposit).toHaveBeenCalledWith('TEST', '150');
		expect(mockStateUpdateCallback).toHaveBeenCalled();
	});
});
