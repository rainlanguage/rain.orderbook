import { render, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import TokenIOInput from '../lib/components/deployment/TokenIOInput.svelte';
import type { ComponentProps } from 'svelte';
import { DotrainOrderGui } from '@rainlanguage/orderbook';
import { useGui } from '$lib/hooks/useGui';

vi.mock('@rainlanguage/orderbook', () => ({
	DotrainOrderGui: vi.fn()
}));

vi.mock('$lib/hooks/useGui', () => ({
	useGui: vi.fn()
}));

type TokenIOInputComponentProps = ComponentProps<TokenIOInput>;

describe('TokenInput', () => {
	let guiInstance: DotrainOrderGui;
	let mockStateUpdateCallback: Mock;
	let mockProps: TokenIOInputComponentProps;
	let outputMockProps: TokenIOInputComponentProps;

	const mockInput = {
		token: {
			address: '0x123',
			key: 'test'
		}
	};
	const mockTokenInfo = {
		value: {
			symbol: 'MOCK',
			name: 'Mock Token',
			decimals: 18
		}
	};

	beforeEach(() => {
		vi.clearAllMocks();

		// Create a mock instance with all the methods
		guiInstance = {
			getTokenInfo: vi.fn().mockResolvedValue(mockTokenInfo),
			setVaultId: vi.fn().mockImplementation(() => {
				mockStateUpdateCallback();
			}),
			getCurrentDeployment: vi.fn().mockResolvedValue({
				deployment: {
					order: {
						inputs: [mockInput]
					}
				}
			}),
			getVaultIds: vi.fn().mockReturnValue({
				value: new Map([
					['input', ['vault1']],
					['output', ['vault2']]
				])
			})
		} as unknown as DotrainOrderGui;

		mockStateUpdateCallback = vi.fn();

		(useGui as Mock).mockReturnValue(guiInstance);

		mockProps = {
			i: 0,
			label: 'Input',
			vault: mockInput
		} as unknown as TokenIOInputComponentProps;
		outputMockProps = {
			i: 0,
			label: 'Output',
			vault: mockInput
		} as unknown as TokenIOInputComponentProps;
	});

	it('renders with correct label and no token symbol', () => {
		const { getByText } = render(TokenIOInput, mockProps);
		expect(getByText('Input 1')).toBeInTheDocument();
	});

	it('renders input field with correct placeholder', () => {
		const { getByPlaceholderText } = render(TokenIOInput, mockProps);
		const input = getByPlaceholderText('Enter vault ID');
		expect(input).toBeInTheDocument();
	});

	it('displays the correct vault ID value', async () => {
		const { getByText } = render(TokenIOInput, mockProps);
		await waitFor(() => {
			expect(getByText('MOCK vault ID')).toBeInTheDocument();
		});
	});

	it('calls setVaultId when input changes', async () => {
		const input = render(TokenIOInput, mockProps).getByPlaceholderText('Enter vault ID');
		await fireEvent.input(input, { target: { value: 'vault1' } });
		expect(guiInstance.setVaultId).toHaveBeenCalledWith(true, 0, 'vault1');
		expect(mockStateUpdateCallback).toHaveBeenCalledTimes(1);
	});

	it('calls setVaultId on output vault when input changes', async () => {
		const input = render(TokenIOInput, outputMockProps).getByPlaceholderText('Enter vault ID');
		await fireEvent.input(input, { target: { value: 'vault2' } });
		expect(guiInstance.setVaultId).toHaveBeenCalledWith(false, 0, 'vault2');
		expect(mockStateUpdateCallback).toHaveBeenCalledTimes(1);
	});

	it('handles missing token info gracefully', () => {
		const propsWithUnknownToken = {
			...mockProps,
			vault: { token: { address: '0x789' } }
		};
		const { getByText } = render(
			TokenIOInput,
			propsWithUnknownToken as unknown as TokenIOInputComponentProps
		);
		expect(getByText('Input 1')).toBeInTheDocument();
	});

	it('fetches and displays token symbol when token key is present', async () => {
		const propsWithTokenKey = {
			...mockProps,
			vault: {
				token: {
					key: '0x456'
				}
			}
		} as unknown as TokenIOInputComponentProps;

		const { findByText } = render(TokenIOInput, propsWithTokenKey);

		const labelWithSymbol = await findByText('Input 1 (MOCK)');
		expect(labelWithSymbol).toBeInTheDocument();
		expect(guiInstance.getTokenInfo).toHaveBeenCalledWith('0x456');
	});
});
