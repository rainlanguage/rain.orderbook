import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import DepositInput from '../lib/components/deployment/DepositInput.svelte';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import type { GuiDepositCfg } from '@rainlanguage/orderbook/js_api';

import { useGui } from '$lib/hooks/useGui';

vi.mock('$lib/hooks/useGui', () => ({
	useGui: vi.fn()
}));

describe('DepositInput', () => {
	let mockGui: DotrainOrderGui;
	let mockStateUpdateCallback: Mock;

	const mockDeposit: GuiDepositCfg = {
		token: { address: '0x123', key: 'TEST', symbol: 'TEST' },
		presets: ['100', '200', '300']
	} as unknown as GuiDepositCfg;

	beforeEach(() => {
		mockStateUpdateCallback = vi.fn();

		mockGui = {
			getTokenInfo: vi.fn().mockReturnValue({ name: 'Test Token', symbol: 'TEST' }),
			isDepositPreset: vi.fn(),
			saveDeposit: vi.fn().mockImplementation(() => {
				mockStateUpdateCallback();
			}),
			getDeposits: vi.fn().mockReturnValue([{ token: 'output', amount: '10', address: '0x1234' }])
		} as unknown as DotrainOrderGui;

		vi.mocked(useGui).mockReturnValue(mockGui);

		vi.clearAllMocks();
	});

	it('renders token name and presets', async () => {
		const { getByText } = render(DepositInput, {
			props: {
				deposit: mockDeposit
			}
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
				deposit: mockDeposit
			}
		});

		await fireEvent.click(getByText('100'));
		expect(mockGui.saveDeposit).toHaveBeenCalledWith('TEST', '100');
	});

	it('handles custom input changes and triggers state update', async () => {
		(mockGui.isDepositPreset as Mock).mockReturnValue(false);

		const { getByPlaceholderText } = render(DepositInput, {
			props: {
				deposit: mockDeposit
			}
		});

		const input = getByPlaceholderText('Enter deposit amount');
		await fireEvent.input(input, { target: { value: '150' } });

		expect(mockGui.saveDeposit).toHaveBeenCalledWith('TEST', '150');
		expect(mockStateUpdateCallback).toHaveBeenCalled();
	});
});
