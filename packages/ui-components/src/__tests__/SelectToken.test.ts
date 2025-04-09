import { render, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import SelectToken from '../lib/components/deployment/SelectToken.svelte';
import type { ComponentProps } from 'svelte';
import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import { useGui } from '$lib/hooks/useGui';

type SelectTokenComponentProps = ComponentProps<SelectToken>;

const mockGui: DotrainOrderGui = {
	saveSelectToken: vi.fn(),
	isSelectTokenSet: vi.fn(),
	getTokenInfo: vi.fn().mockResolvedValue({
		symbol: 'ETH',
		decimals: 18,
		address: '0x456'
	})
} as unknown as DotrainOrderGui;

vi.mock('../lib/hooks/useGui', () => ({
	useGui: vi.fn()
}));

describe('SelectToken', () => {
	let mockStateUpdateCallback: Mock;

	const mockProps: SelectTokenComponentProps = {
		token: {
			key: 'input',
			name: 'test input',
			description: 'test description'
		},
		onSelectTokenSelect: vi.fn()
	};

	beforeEach(() => {
		vi.mocked(useGui).mockReturnValue(mockGui);
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
		const user = userEvent.setup();
		const mockGuiWithNoToken = {
			...mockGui,
			getTokenInfo: vi.fn().mockResolvedValue(null)
		} as unknown as DotrainOrderGui;

		(useGui as Mock).mockReturnValue(mockGuiWithNoToken);

		const { getByRole } = render(SelectToken, {
			...mockProps
		});
		const input = getByRole('textbox');

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
	});

	it('shows error message for invalid address, and removes the selectToken', async () => {
		mockGui.saveSelectToken = vi.fn().mockRejectedValue(new Error('Invalid address'));

		const user = userEvent.setup();

		(useGui as Mock).mockReturnValue(mockGuiWithError);

		const screen = render(SelectToken, {
			...mockProps
		});

		const input = screen.getByRole('textbox');
		await userEvent.clear(input);
		await user.paste('invalid');

		await waitFor(() => {
			expect(screen.getByTestId('error')).toBeInTheDocument();
		});
	});

	it('replaces the token and triggers state update if the token is already set', async () => {
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

		(useGui as Mock).mockReturnValue(mockGuiWithTokenSet);

		const user = userEvent.setup();

		const { getByRole } = render(SelectToken, {
			...mockProps
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
