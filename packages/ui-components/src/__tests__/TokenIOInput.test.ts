import { render, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import TokenIOInput from '../lib/components/deployment/TokenIOInput.svelte';
import { DotrainOrderGui, type OrderIOCfg } from '@rainlanguage/orderbook/js_api';

import { useGui } from '$lib/hooks/useGui';

vi.mock('$lib/hooks/useGui', () => ({
	useGui: vi.fn()
}));

describe('TokenInput', () => {
	let mockGui: DotrainOrderGui;
	let mockStateUpdateCallback: Mock;

	const mockInput = {
		token: {
			address: '0x123',
			key: 'test'
		}
	};
	const mockTokenInfo = {
		symbol: 'MOCK',
		name: 'Mock Token',
		decimals: 18
	};

	beforeEach(() => {
		mockStateUpdateCallback = vi.fn();

		mockGui = {
			setVaultId: vi.fn().mockImplementation(() => {
				mockStateUpdateCallback();
			}),
			getTokenInfo: vi.fn().mockResolvedValue(mockTokenInfo),
			getCurrentDeployment: vi.fn().mockResolvedValue({
				deployment: {
					order: {
						inputs: [mockInput]
					}
				}
			}),
			getVaultIds: vi.fn().mockReturnValue(
				new Map([
					['input', ['vault1']],
					['output', ['vault2']]
				])
			)
		} as unknown as DotrainOrderGui;

		vi.mocked(useGui).mockReturnValue(mockGui);

		vi.clearAllMocks();
	});

	it('renders with correct label and no token symbol', () => {
		const { getByText } = render(TokenIOInput, {
			props: {
				i: 0,
				label: 'Input',
				vault: mockInput as OrderIOCfg
			}
		});
		expect(getByText('Input 1')).toBeInTheDocument();
	});

	it('renders input field with correct placeholder', () => {
		const { getByPlaceholderText } = render(TokenIOInput, {
			props: {
				i: 0,
				label: 'Input',
				vault: mockInput as OrderIOCfg
			}
		});
		const input = getByPlaceholderText('Enter vault ID');
		expect(input).toBeInTheDocument();
	});

	it('displays the correct vault ID value', async () => {
		const { getByText } = render(TokenIOInput, {
			props: {
				i: 0,
				label: 'Input',
				vault: mockInput as OrderIOCfg
			}
		});
		await waitFor(() => {
			expect(getByText('MOCK vault ID')).toBeInTheDocument();
		});
	});

	it('calls setVaultId when input changes', async () => {
		const { getByPlaceholderText } = render(TokenIOInput, {
			props: {
				i: 0,
				label: 'Input',
				vault: mockInput as OrderIOCfg
			}
		});

		const input = getByPlaceholderText('Enter vault ID');
		await fireEvent.input(input, { target: { value: 'vault1' } });

		expect(mockGui.setVaultId).toHaveBeenCalledWith(true, 0, 'vault1');
		expect(mockStateUpdateCallback).toHaveBeenCalledTimes(1);
	});

	it('calls setVaultId on output vault when input changes', async () => {
		const { getByPlaceholderText } = render(TokenIOInput, {
			props: {
				i: 0,
				label: 'Output',
				vault: mockInput as OrderIOCfg
			}
		});

		const input = getByPlaceholderText('Enter vault ID');
		await fireEvent.input(input, { target: { value: 'vault2' } });

		expect(mockGui.setVaultId).toHaveBeenCalledWith(false, 0, 'vault2');
		expect(mockStateUpdateCallback).toHaveBeenCalledTimes(1);
	});

	it('handles missing token info gracefully', () => {
		const { getByText } = render(TokenIOInput, {
			props: {
				i: 0,
				label: 'Input',
				vault: { token: { address: '0x789' } } as OrderIOCfg
			}
		});

		expect(getByText('Input 1')).toBeInTheDocument();
	});

	it('fetches and displays token symbol when token key is present', async () => {
		const { findByText } = render(TokenIOInput, {
			props: {
				i: 0,
				label: 'Input',
				vault: {
					token: {
						key: '0x456'
					}
				} as OrderIOCfg
			}
		});

		const labelWithSymbol = await findByText('Input 1 (MOCK)');
		expect(labelWithSymbol).toBeInTheDocument();
		expect(mockGui.getTokenInfo).toHaveBeenCalledWith('0x456');
	});
});
