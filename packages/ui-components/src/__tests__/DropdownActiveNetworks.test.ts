import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { expect, test, describe, type Mock, vi } from 'vitest';
import DropdownActiveNetworks from '../lib/components/dropdown/DropdownActiveNetworks.svelte';
import { useRaindexClient } from '$lib/hooks/useRaindexClient';
import type { NetworkCfg } from '@rainlanguage/orderbook';

vi.mock('$lib/hooks/useRaindexClient', () => ({
	useRaindexClient: vi.fn()
}));

describe('DropdownActiveNetworks', () => {
	const mockUseRaindexClient = () => {
		(useRaindexClient as Mock).mockReturnValue({
			getUniqueChainIds: () => ({
				error: undefined,
				value: [1, 2, 14]
			}),
			getAllNetworks: () => ({
				error: undefined,
				value: new Map([
					['mainnet', { key: 'Ethereum', chainId: 1 } as NetworkCfg],
					['testnet', { key: 'Expanse Network', chainId: 2 } as NetworkCfg],
					['local', { key: 'local', chainId: 14 } as NetworkCfg]
				])
			})
		});
	};

	test('renders correctly', () => {
		mockUseRaindexClient();
		const onChange = vi.fn();

		render(DropdownActiveNetworks, {
			props: {
				selectedChainIds: [],
				onChange
			}
		});
		expect(screen.getByText('Networks')).toBeInTheDocument();
	});

	test('displays the correct number of options', async () => {
		mockUseRaindexClient();
		const onChange = vi.fn();

		render(DropdownActiveNetworks, {
			props: {
				selectedChainIds: [],
				onChange
			}
		});

		await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));

		await waitFor(() => {
			const options = screen.getAllByTestId('dropdown-checkbox-option');
			expect(options).toHaveLength(3);
		});
	});

	test('calls onChange when options are selected', async () => {
		mockUseRaindexClient();
		const onChange = vi.fn();

		render(DropdownActiveNetworks, {
			props: {
				selectedChainIds: [],
				onChange
			}
		});

		await fireEvent.click(screen.getByTestId('dropdown-checkbox-button'));
		await fireEvent.click(screen.getByText('Ethereum'));

		await waitFor(() => {
			expect(onChange).toHaveBeenCalledWith([1]);
		});
	});
});
