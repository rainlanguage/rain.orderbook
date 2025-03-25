import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import DepositInput from '../lib/components/deployment/DepositInput.svelte';
import type { GuiDepositCfg } from '@rainlanguage/orderbook/js_api';
import type { ComponentProps } from 'svelte';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

type DepositInputProps = ComponentProps<DepositInput>;

describe('DepositInput', () => {
	let mockStateUpdateCallback: Mock;
	let guiInstance: DotrainOrderGui;

	const mockDeposit: GuiDepositCfg = {
		token: { address: '0x123', key: 'TEST', symbol: 'TEST' },
		presets: ['100', '200', '300']
	} as unknown as GuiDepositCfg;

	beforeEach(() => {
		vi.clearAllMocks();
		guiInstance = new DotrainOrderGui();

		mockStateUpdateCallback = vi.fn();
		(DotrainOrderGui.prototype.getDeposits as Mock).mockReturnValue([
			{ token: 'output', amount: '10', address: '0x1234' }
		]);
		(DotrainOrderGui.prototype.saveDeposit as Mock).mockImplementation(() => {
			mockStateUpdateCallback();
		});
	});

	it('renders token name and presets', async () => {
		(DotrainOrderGui.prototype.getTokenInfo as Mock).mockResolvedValueOnce({
			value: {
				name: 'Test Token',
				symbol: 'TEST'
			}
		});

		const { getByText } = render(DepositInput, {
			props: {
				deposit: mockDeposit,
				gui: guiInstance
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
				gui: guiInstance
			} as unknown as DepositInputProps
		});

		await fireEvent.click(getByText('100'));
		expect(guiInstance.saveDeposit).toHaveBeenCalledWith('TEST', '100');
	});

	it('handles custom input changes and triggers state update', async () => {
		(DotrainOrderGui.prototype.isDepositPreset as Mock).mockReturnValue(false);

		const { getByPlaceholderText } = render(DepositInput, {
			props: {
				deposit: mockDeposit,
				gui: guiInstance,
				onStateUpdate: mockStateUpdateCallback
			} as unknown as DepositInputProps
		});

		const input = getByPlaceholderText('Enter deposit amount');
		await fireEvent.input(input, { target: { value: '150' } });

		expect(guiInstance.saveDeposit).toHaveBeenCalledWith('TEST', '150');
		expect(mockStateUpdateCallback).toHaveBeenCalled();
	});
});
