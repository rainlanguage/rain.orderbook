import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import SelectToken from '../lib/components/deployment/SelectToken.svelte';
import type { ComponentProps } from 'svelte';
import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

export type SelectTokenComponentProps = ComponentProps<SelectToken>;
describe('SelectToken', () => {
	const mockGui: DotrainOrderGui = {
		saveSelectTokenAddress: vi.fn().mockResolvedValue(undefined),
		getSelectTokens: vi.fn().mockReturnValue(new Map([['TOKEN1', '0x123']]))
	} as unknown as DotrainOrderGui;

	const mockProps: SelectTokenComponentProps = {
		token: 'TOKEN1',
		gui: mockGui,
		selectTokens: new Map([['TOKEN1', '0x123']])
	};

	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('renders token label correctly', () => {
		const { getByText } = render(SelectToken, mockProps);
		expect(getByText('TOKEN1')).toBeInTheDocument();
	});

	it('renders input field', () => {
		const { getByRole } = render(SelectToken, mockProps);
		expect(getByRole('textbox')).toBeInTheDocument();
	});

	it('calls saveSelectTokenAddress when input changes', async () => {
		const { getByRole } = render(SelectToken, mockProps);
		const input = getByRole('textbox');

		await fireEvent.change(input, { target: { value: '0x456' } });

		expect(mockGui.saveSelectTokenAddress).toHaveBeenCalledWith('TOKEN1', '0x456');
		expect(mockGui.getSelectTokens).toHaveBeenCalled();
	});

	it('does nothing if gui is not defined', async () => {
		const { getByRole } = render(SelectToken, {
			...mockProps,
			gui: undefined
		} as unknown as SelectTokenComponentProps);
		const input = getByRole('textbox');

		await fireEvent.change(input, { target: { value: '0x456' } });

		expect(mockGui.saveSelectTokenAddress).not.toHaveBeenCalled();
		expect(mockGui.getSelectTokens).not.toHaveBeenCalled();
	});
});
