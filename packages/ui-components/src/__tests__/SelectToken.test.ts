import { render, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import SelectToken from '../lib/components/deployment/SelectToken.svelte';
import type { ComponentProps } from 'svelte';
import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

export type SelectTokenComponentProps = ComponentProps<SelectToken>;
describe('SelectToken', () => {
	let mockStateUpdateCallback: Mock;
	const mockGui: DotrainOrderGui = {
		saveSelectToken: vi.fn(),
		replaceSelectToken: vi.fn(),
		isSelectTokenSet: vi.fn(),
		getTokenInfo: vi.fn().mockResolvedValue({
			symbol: 'ETH',
			decimals: 18,
			address: '0x456'
		})
	} as unknown as DotrainOrderGui;

	const mockProps: SelectTokenComponentProps = {
		gui: mockGui,
		token: {
			key: 'input',
			name: 'test input',
			description: 'test description'
		},
		onSelectTokenSelect: vi.fn()
	};

	beforeEach(() => {
		mockStateUpdateCallback = vi.fn();
		mockGui.saveSelectToken = vi.fn().mockImplementation(() => {
			mockStateUpdateCallback();
			return Promise.resolve();
		});
		mockGui.replaceSelectToken = vi.fn().mockImplementation(() => {
			mockStateUpdateCallback();
			mockStateUpdateCallback();
			return Promise.resolve();
		});
		vi.clearAllMocks();
	});

	it('renders token label correctly', () => {
		const { getByText } = render(SelectToken, mockProps);
		expect(getByText('test input')).toBeInTheDocument();
	});

	it('renders input field', () => {
		const { getByRole } = render(SelectToken, mockProps);
		expect(getByRole('textbox')).toBeInTheDocument();
	});

	it('calls saveSelectToken and updates token info when input changes', async () => {
		const user = userEvent.setup();
		const mockGuiWithNoToken = {
			...mockGui,
			getTokenInfo: vi.fn().mockResolvedValue(null)
		} as unknown as DotrainOrderGui;
		const { getByRole } = render(SelectToken, {
			...mockProps,
			gui: mockGuiWithNoToken
		});
		const input = getByRole('textbox');

		await userEvent.clear(input);
		await user.paste('0x456');

		await waitFor(() => {
			expect(mockGui.saveSelectToken).toHaveBeenCalledWith('input', '0x456');
		});
		expect(mockStateUpdateCallback).toHaveBeenCalledTimes(1);
	});

	it('shows error message for invalid address, and removes the selectToken', async () => {
		const user = userEvent.setup();
		const mockGuiWithError = {
			...mockGui,
			saveSelectToken: vi.fn().mockRejectedValue(new Error('Invalid address'))
		} as unknown as DotrainOrderGui;

		const screen = render(SelectToken, {
			...mockProps,
			gui: mockGuiWithError
		});

		const input = screen.getByRole('textbox');
		await userEvent.clear(input);
		await user.paste('invalid');
		await waitFor(() => {
			expect(screen.getByTestId('error')).toBeInTheDocument();
		});
	});

	it('does nothing if gui is not defined', async () => {
		const user = userEvent.setup();
		const { getByRole } = render(SelectToken, {
			...mockProps,
			gui: undefined
		} as unknown as SelectTokenComponentProps);
		const input = getByRole('textbox');

		await userEvent.clear(input);
		await user.paste('0x456');

		await waitFor(() => {
			expect(mockGui.saveSelectToken).not.toHaveBeenCalled();
		});
	});

	it('replaces the token and triggers state update twice if the token is already set', async () => {
		const mockGuiWithTokenSet = {
			...mockGui,
			isSelectTokenSet: vi.fn().mockResolvedValue(true)
		} as unknown as DotrainOrderGui;

		const user = userEvent.setup();

		const { getByRole } = render(SelectToken, {
			...mockProps,
			gui: mockGuiWithTokenSet
		});

		const input = getByRole('textbox');
		await userEvent.clear(input);
		await user.paste('invalid');
		await waitFor(() => {
			expect(mockGui.replaceSelectToken).toHaveBeenCalled();
			expect(mockStateUpdateCallback).toHaveBeenCalledTimes(2);
		});
	});

	it('calls onSelectTokenSelect after input changes', async () => {
		const user = userEvent.setup();
		const { getByRole } = render(SelectToken, mockProps);
		const input = getByRole('textbox');

		await userEvent.clear(input);
		await user.paste('0x456');

		await waitFor(() => {
			expect(mockProps.onSelectTokenSelect).toHaveBeenCalled();
		});
	});
});
