import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import TokenIOInput from '../lib/components/deployment/TokenIOInput.svelte';
import type { ComponentProps } from 'svelte';

export type TokenIOInputComponentProps = ComponentProps<TokenIOInput>;

describe('TokenInput', () => {
	const mockInput = {
		token: {
			address: '0x123'
		}
	};

	const mockGui = {
		setVaultId: vi.fn(),
		getTokenInfo: vi.fn()
	};

	const mockTokenInfo = {
		symbol: 'MOCK',
		name: 'Mock Token',
		decimals: 18
	};

	const inputMockProps: TokenIOInputComponentProps = {
		i: 0,
		isInput: true,
		label: 'Input',
		vault: mockInput,
		vaultIds: ['vault1'],
		gui: mockGui
	} as unknown as TokenIOInputComponentProps;
	const outputMockProps: TokenIOInputComponentProps = {
		i: 0,
		isInput: false,
		label: 'Output',
		vault: mockInput,
		vaultIds: ['vault2'],
		gui: mockGui
	} as unknown as TokenIOInputComponentProps;

	beforeEach(() => {
		vi.clearAllMocks();
		mockGui.getTokenInfo = vi.fn().mockResolvedValue(mockTokenInfo);
	});

	it('renders with correct label and no token symbol', () => {
		const { getByText } = render(TokenIOInput, inputMockProps);
		expect(getByText('Input 1')).toBeInTheDocument();
	});

	it('renders input field with correct placeholder', () => {
		const { getByPlaceholderText } = render(TokenIOInput, inputMockProps);
		const input = getByPlaceholderText('Enter vault ID');
		expect(input).toBeInTheDocument();
	});

	it('displays the correct vault ID value', () => {
		const { getByDisplayValue } = render(TokenIOInput, inputMockProps);
		expect(getByDisplayValue('vault1')).toBeInTheDocument();
	});

	it('calls setVaultId on input vault when input changes', async () => {
		const input = render(TokenIOInput, inputMockProps).getByPlaceholderText('Enter vault ID');
		await fireEvent.change(input, { target: { value: 'vault1' } });
		expect(mockGui.setVaultId).toHaveBeenCalledWith(true, 0, 'vault1');
	});

	it('calls setVaultId on output vault when input changes', async () => {
		const input = render(TokenIOInput, outputMockProps).getByPlaceholderText('Enter vault ID');
		await fireEvent.change(input, { target: { value: 'vault2' } });
		expect(mockGui.setVaultId).toHaveBeenCalledWith(false, 0, 'vault2');
	});

	it('does not call setVaultId when gui is undefined', async () => {
		const propsWithoutGui = {
			...inputMockProps,
			gui: undefined
		} as unknown as TokenIOInputComponentProps;
		const { getByPlaceholderText } = render(TokenIOInput, propsWithoutGui);
		const input = getByPlaceholderText('Enter vault ID');

		await fireEvent.change(input, { target: { value: 'newVault' } });

		expect(mockGui.setVaultId).not.toHaveBeenCalled();
	});

	it('handles missing token info gracefully', () => {
		const propsWithUnknownToken = {
			...inputMockProps,
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
			...inputMockProps,
			vault: {
				token: {
					key: '0x456'
				}
			}
		} as unknown as TokenIOInputComponentProps;

		const { findByText } = render(TokenIOInput, propsWithTokenKey);

		const labelWithSymbol = await findByText('Input 1 (MOCK)');
		expect(labelWithSymbol).toBeInTheDocument();
		expect(mockGui.getTokenInfo).toHaveBeenCalledWith('0x456');
	});
});
