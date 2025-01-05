import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import DepositButtons from '../lib/components/deployment/DepositButtons.svelte';
import type { GuiDeposit } from '@rainlanguage/orderbook/js_api';
import type { ComponentProps } from 'svelte';

type DepositButtonsProps = ComponentProps<DepositButtons>;

describe('DepositButtons', () => {
	const mockGui = {
		isDepositPreset: vi.fn(),
		saveDeposit: vi.fn()
	};

	const mockTokenInfos = new Map([['0x123', { name: 'Test Token', symbol: 'TEST' }]]);

	const mockDeposit: GuiDeposit = {
		token: { address: '0x123', key: 'TEST' },
		presets: ['100', '200', '300']
	} as unknown as GuiDeposit;

	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('renders token name and presets', () => {
		const { getByText } = render(DepositButtons, {
			props: {
				deposit: mockDeposit,
				gui: mockGui,
				tokenInfos: mockTokenInfos
			} as unknown as DepositButtonsProps
		});

		expect(getByText('Test Token')).toBeTruthy();
		expect(getByText('100')).toBeTruthy();
		expect(getByText('200')).toBeTruthy();
		expect(getByText('300')).toBeTruthy();
	});

	it('handles preset button clicks', async () => {
		const { getByText } = render(DepositButtons, {
			props: {
				deposit: mockDeposit,
				gui: mockGui,
				tokenInfos: mockTokenInfos
			} as unknown as DepositButtonsProps
		});

		await fireEvent.click(getByText('100'));
		expect(mockGui.saveDeposit).toHaveBeenCalledWith('TEST', '100');
	});

	it('handles custom input changes', async () => {
		mockGui.isDepositPreset.mockReturnValue(false);

		const { getByPlaceholderText } = render(DepositButtons, {
			props: {
				deposit: mockDeposit,
				gui: mockGui,
				tokenInfos: mockTokenInfos
			} as unknown as DepositButtonsProps
		});

		const input = getByPlaceholderText('Enter deposit amount');
		await fireEvent.input(input, { target: { value: '150' } });

		expect(mockGui.saveDeposit).toHaveBeenCalledWith('TEST', '150');
	});
});
