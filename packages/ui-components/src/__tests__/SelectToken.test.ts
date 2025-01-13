import { render, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import SelectToken from '../lib/components/deployment/SelectToken.svelte';
import type { ComponentProps } from 'svelte';
import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

export type SelectTokenComponentProps = ComponentProps<SelectToken>;
describe('SelectToken', () => {
	const mockGui: DotrainOrderGui = {
		saveSelectTokenAddress: vi.fn().mockResolvedValue(undefined),
		getSelectTokens: vi.fn().mockReturnValue(new Map([['TOKEN1', '0x123']])),
		getTokenInfos: vi
			.fn()
			.mockResolvedValue(new Map([['0x123', { name: 'Test Token', symbol: 'TEST', decimals: 18 }]]))
	} as unknown as DotrainOrderGui;

	const mockProps: SelectTokenComponentProps = {
		token: ['TOKEN1', '0x123'],
		gui: mockGui,
		selectTokens: new Map([['TOKEN1', '0x123']]),
		tokenInfos: new Map([['0x123', { name: 'Test Token', symbol: 'TEST', decimals: 18 }]])
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

	it('calls saveSelectTokenAddress and updates token info when input changes', async () => {
		const user = userEvent.setup();
		const { getByRole } = render(SelectToken, mockProps);
		const input = getByRole('textbox');

		await user.clear(input);
		await user.type(input, '0x456');

		await waitFor(() => {
			expect(mockGui.saveSelectTokenAddress).toHaveBeenCalledWith('TOKEN1', '0x456');
			expect(mockGui.getSelectTokens).toHaveBeenCalled();
			expect(mockGui.getTokenInfos).toHaveBeenCalled();
		});
	});

	it('shows error message for invalid address', async () => {
		const user = userEvent.setup();
		const mockGuiWithError = {
			...mockGui,
			saveSelectTokenAddress: vi.fn().mockRejectedValue(new Error('Invalid address'))
		} as unknown as DotrainOrderGui;

		const { getByRole, findByText } = render(SelectToken, {
			...mockProps,
			gui: mockGuiWithError
		});

		const input = getByRole('textbox');
		await user.type(input, 'invalid');

		await waitFor(() => {
			expect(findByText('Invalid address')).resolves.toBeInTheDocument();
		});
	});

	it('does nothing if gui is not defined', async () => {
		const user = userEvent.setup();
		const { getByRole } = render(SelectToken, {
			...mockProps,
			gui: undefined
		} as unknown as SelectTokenComponentProps);
		const input = getByRole('textbox');

		await user.type(input, '0x456');

		await waitFor(() => {
			expect(mockGui.saveSelectTokenAddress).not.toHaveBeenCalled();
			expect(mockGui.getSelectTokens).not.toHaveBeenCalled();
		});
	});
});
