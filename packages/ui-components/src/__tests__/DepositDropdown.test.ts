import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import DepositDropdown from '../lib/components/deployment/DepositDropdown.svelte';
import type { DotrainOrderGui, GuiDeposit } from '@rainlanguage/orderbook/js_api';

describe('DepositDropdown', () => {
    const mockTokenInfos = new Map();
    mockTokenInfos.set('0x123', { name: 'Test Token' });

    const mockDeposit: GuiDeposit = {
        token: { address: '0x123' },
        token_name: 'TEST',
        presets: ['100', '200', '300']
    } as unknown as GuiDeposit;

    const mockGui = {
        saveDeposit: vi.fn(),
        isDepositPreset: vi.fn()
    };

    it('renders token name correctly', () => {
        const { getByText } = render(DepositDropdown, {
            props: {
                deposit: mockDeposit,
                gui: mockGui as unknown as DotrainOrderGui,
                tokenInfos: mockTokenInfos
            }
        });

        expect(getByText('Test Token')).toBeTruthy();
    });

    it('shows "Choose deposit amount" by default', () => {
        const { getByText } = render(DepositDropdown, {
            props: {
                deposit: mockDeposit,
                gui: mockGui as unknown as DotrainOrderGui,
                tokenInfos: mockTokenInfos
            }
        });

        expect(getByText('Choose deposit amount')).toBeTruthy();
    });

    it('calls saveDeposit when preset is selected', async () => {
        mockGui.isDepositPreset.mockReturnValue(true);

        const { getByText } = render(DepositDropdown, {
            props: {
                deposit: mockDeposit,
                gui: mockGui as unknown as DotrainOrderGui,
                tokenInfos: mockTokenInfos
            }
        });

        await fireEvent.click(getByText('Choose deposit amount'));
        await fireEvent.click(getByText('100'));

        expect(mockGui.saveDeposit).toHaveBeenCalledWith('TEST', '100');
    });

    it('shows input field when custom value is selected', async () => {
        mockGui.isDepositPreset.mockReturnValue(false);

        const { getByText, getByPlaceholderText } = render(DepositDropdown, {
            props: {
                deposit: mockDeposit,
                gui: mockGui as unknown as DotrainOrderGui,
                tokenInfos: mockTokenInfos
            }
        });

        await fireEvent.click(getByText('Choose deposit amount'));
        await fireEvent.click(getByText('Custom value'));

        expect(getByPlaceholderText('Enter deposit amount')).toBeTruthy();
    });

    it('calls saveDeposit when custom value is entered', async () => {
        mockGui.isDepositPreset.mockReturnValue(false);

        const { getByPlaceholderText } = render(DepositDropdown, {
            props: {
                deposit: mockDeposit,
                gui: mockGui as unknown as DotrainOrderGui,
                tokenInfos: mockTokenInfos
            }
        });

        const input = getByPlaceholderText('Enter deposit amount');
        await fireEvent.change(input, { target: { value: '150' } });

        expect(mockGui.saveDeposit).toHaveBeenCalledWith('TEST', '150');
    });
});