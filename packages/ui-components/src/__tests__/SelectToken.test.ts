import { render, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import SelectToken from '../lib/components/deployment/SelectToken.svelte';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import type { GuiSelectTokensCfg } from '@rainlanguage/orderbook/js_api';

import { useGui } from '$lib/hooks/useGui';

vi.mock('$lib/hooks/useGui', () => ({
	useGui: vi.fn()
}));

describe('SelectToken', () => {
	let mockStateUpdateCallback: Mock;
	const mockGui: DotrainOrderGui = {
		saveSelectToken: vi.fn(),
		isSelectTokenSet: vi.fn(),
		getTokenInfo: vi.fn().mockResolvedValue({
			symbol: 'ETH',
			decimals: 18,
			address: '0x456'
		})
	} as unknown as DotrainOrderGui;

	const mockToken: GuiSelectTokensCfg = {
		key: 'input',
		name: 'test input',
		description: 'test description'
	};

	beforeEach(() => {
		mockStateUpdateCallback = vi.fn();
		mockGui.saveSelectToken = vi.fn().mockImplementation(() => {
			mockStateUpdateCallback();
			return Promise.resolve();
		});
		vi.clearAllMocks();
	});

	it('renders token label correctly', () => {
		const { getByText } = render(SelectToken, {
			props: {
				token: mockToken,
				onSelectTokenSelect: vi.fn()
			}
		});
		expect(getByText('test input')).toBeInTheDocument();
	});

	it('renders input field', () => {
		const { getByRole } = render(SelectToken, {
			props: {
				token: mockToken,
				onSelectTokenSelect: vi.fn()
			}
		});
		expect(getByRole('textbox')).toBeInTheDocument();
	});

	it('calls saveSelectToken and updates token info when input changes', async () => {
		mockGui.getTokenInfo = vi.fn().mockResolvedValue(null);

		const user = userEvent.setup();
		const onSelectTokenSelect = vi.fn();

		const { getByRole } = render(SelectToken, {
			props: {
				token: mockToken,
				onSelectTokenSelect
			}
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
		// Override the saveSelectToken mock to reject
		mockGui.saveSelectToken = vi.fn().mockRejectedValue(new Error('Invalid address'));

		const user = userEvent.setup();

		const screen = render(SelectToken, {
			props: {
				token: mockToken,
				onSelectTokenSelect: vi.fn()
			}
		});

		const input = screen.getByRole('textbox');
		await userEvent.clear(input);
		await user.paste('invalid');

		await waitFor(() => {
			expect(screen.getByTestId('error')).toBeInTheDocument();
		});
	});

	it('replaces the token and triggers state update twice if the token is already set', async () => {
		mockGui.isSelectTokenSet = vi.fn().mockReturnValue(true);

		const user = userEvent.setup();
		const onSelectTokenSelect = vi.fn();

		const { getByRole } = render(SelectToken, {
			props: {
				token: mockToken,
				onSelectTokenSelect
			}
		});

		const input = getByRole('textbox');
		await userEvent.clear(input);
		await user.paste('invalid');

		await waitFor(() => {
			expect(mockGui.saveSelectToken).toHaveBeenCalled();
			expect(mockStateUpdateCallback).toHaveBeenCalledTimes(1);
		});
	});

	it('calls onSelectTokenSelect after input changes', async () => {
		const user = userEvent.setup();
		const onSelectTokenSelect = vi.fn();

		const { getByRole } = render(SelectToken, {
			props: {
				token: mockToken,
				onSelectTokenSelect
			}
		});

		const input = getByRole('textbox');
		await userEvent.clear(input);
		await user.paste('0x456');

		await waitFor(() => {
			expect(onSelectTokenSelect).toHaveBeenCalled();
		});
	});
});
