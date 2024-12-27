import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import TokenInput from '../lib/components/deployment/TokenInput.svelte';
import type { ComponentProps } from 'svelte';

export type TokenInputComponentProps = ComponentProps<TokenInput>;

describe('TokenInput', () => {
    const mockTokenInfos = new Map([
        ['0x123', { symbol: 'ETH' }],
        ['0x456', { symbol: 'USDC' }]
    ]);

    const mockInput = {
        token: {
            address: '0x123'
        }
    };

    const mockGui = {
        setVaultId: vi.fn()
    };

    const mockProps: TokenInputComponentProps = {
        i: 0,
        input: mockInput,
        tokenInfos: mockTokenInfos,
        inputVaultIds: ['vault1'],
        gui: mockGui
    } as unknown as TokenInputComponentProps;

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders with correct label and token symbol', () => {
        const { getByText } = render(TokenInput, mockProps);
        expect(getByText('Input 1 (ETH)')).toBeInTheDocument();
    });

    it('renders input field with correct placeholder', () => {
        const { getByPlaceholderText } = render(TokenInput, mockProps);
        const input = getByPlaceholderText('Enter vault ID');
        expect(input).toBeInTheDocument();
    });

    it('displays the correct vault ID value', () => {
        const { getByDisplayValue } = render(TokenInput, mockProps);
        expect(getByDisplayValue('vault1')).toBeInTheDocument();
    });

    it('calls setVaultId when input changes', async () => {
        const { getByPlaceholderText } = render(TokenInput, mockProps);
        const input = getByPlaceholderText('Enter vault ID');

        await fireEvent.change(input, { target: { value: 'vault1' } });

        expect(mockGui.setVaultId).toHaveBeenCalledWith(true, 0, 'vault1');
    });

    it('does not call setVaultId when gui is undefined', async () => {
        const propsWithoutGui = { ...mockProps, gui: undefined } as unknown as TokenInputComponentProps;
        const { getByPlaceholderText } = render(TokenInput, propsWithoutGui);
        const input = getByPlaceholderText('Enter vault ID');

        await fireEvent.change(input, { target: { value: 'newVault' } });

        expect(mockGui.setVaultId).not.toHaveBeenCalled();
    });

    it('handles missing token info gracefully', () => {
        const propsWithUnknownToken = {
            ...mockProps,
            input: { token: { address: '0x789' } }
        };
        const { getByText } = render(TokenInput, propsWithUnknownToken as unknown as TokenInputComponentProps);
        expect(getByText('Input 1 ()')).toBeInTheDocument();
    });
});