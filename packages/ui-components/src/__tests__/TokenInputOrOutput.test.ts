import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import TokenInputOrOutput from '../lib/components/deployment/TokenInputOrOutput.svelte';
import type { ComponentProps } from 'svelte';

export type TokenInputOrOutputComponentProps = ComponentProps<TokenInputOrOutput>;

describe('TokenInput', () => {
	const mockInput = {
		token: {
			address: '0x123',
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

	const mockProps: TokenInputOrOutputComponentProps = {
		i: 0,
		label: 'Input',
		vault: mockInput,
		vaultIds: ['vault1'],
		gui: mockGui
	} as unknown as TokenInputOrOutputComponentProps;

	beforeEach(() => {
		vi.clearAllMocks();
		mockGui.getTokenInfo = vi.fn().mockResolvedValue(mockTokenInfo);
	});

	it('renders with correct label and no token symbol', () => {
		const { getByText } = render(TokenInputOrOutput, mockProps);
		expect(getByText('Input 1')).toBeInTheDocument();
	});

	it('renders input field with correct placeholder', () => {
		const { getByPlaceholderText } = render(TokenInputOrOutput, mockProps);
		const input = getByPlaceholderText('Enter vault ID');
		expect(input).toBeInTheDocument();
	});

	it('displays the correct vault ID value', () => {
		const { getByDisplayValue } = render(TokenInputOrOutput, mockProps);
		expect(getByDisplayValue('vault1')).toBeInTheDocument();
	});

	it('calls setVaultId when input changes', async () => {
		const { getByPlaceholderText } = render(TokenInputOrOutput, mockProps);
		const input = getByPlaceholderText('Enter vault ID');

		await fireEvent.change(input, { target: { value: 'vault1' } });

		expect(mockGui.setVaultId).toHaveBeenCalledWith(true, 0, 'vault1');
	});

	it('does not call setVaultId when gui is undefined', async () => {
		const propsWithoutGui = {
			...mockProps,
			gui: undefined
		} as unknown as TokenInputOrOutputComponentProps;
		const { getByPlaceholderText } = render(TokenInputOrOutput, propsWithoutGui);
		const input = getByPlaceholderText('Enter vault ID');

		await fireEvent.change(input, { target: { value: 'newVault' } });

		expect(mockGui.setVaultId).not.toHaveBeenCalled();
	});

	it('handles missing token info gracefully', () => {
		const propsWithUnknownToken = {
			...mockProps,
			vault: { token: { address: '0x789' } }
		};
		const { getByText } = render(
			TokenInputOrOutput,
			propsWithUnknownToken as unknown as TokenInputOrOutputComponentProps
		);
		expect(getByText('Input 1')).toBeInTheDocument();
	});

	it('fetches and displays token symbol when token key is present', async () => {
		const propsWithTokenKey = {
			...mockProps,
			vault: {
				token: {
					key: '0x456',
				}
			}
		} as unknown as TokenInputOrOutputComponentProps;

		const { findByText } = render(TokenInputOrOutput, propsWithTokenKey);

		const labelWithSymbol = await findByText('Input 1 (MOCK)');
		expect(labelWithSymbol).toBeInTheDocument();
		expect(mockGui.getTokenInfo).toHaveBeenCalledWith('0x456');
	});
});
