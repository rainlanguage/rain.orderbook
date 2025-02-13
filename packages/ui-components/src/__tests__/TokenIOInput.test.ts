import { render, fireEvent, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import TokenIOInput from '../lib/components/deployment/TokenIOInput.svelte';
import type { ComponentProps } from 'svelte';

export type TokenIOInputComponentProps = ComponentProps<TokenIOInput>;

describe('TokenInput', () => {
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

	const mockGui = {
		setVaultId: vi.fn(),
		getTokenInfo: vi.fn(),
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
	};

	const mockProps: TokenIOInputComponentProps = {
		i: 0,
		label: 'Input',
		vault: mockInput,
		gui: mockGui,
		handleUpdateGuiState: vi.fn()
	} as unknown as TokenIOInputComponentProps;

	const outputMockProps: TokenIOInputComponentProps = {
		i: 0,
		label: 'Output',
		vault: mockInput,
		gui: mockGui,
		handleUpdateGuiState: vi.fn()
	} as unknown as TokenIOInputComponentProps;

	beforeEach(() => {
		vi.clearAllMocks();
		mockGui.getTokenInfo = vi.fn().mockResolvedValue(mockTokenInfo);
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
		expect(mockGui.setVaultId).toHaveBeenCalledWith(true, 0, 'vault1');
	});

	it('calls setVaultId on output vault when input changes', async () => {
		const input = render(TokenIOInput, outputMockProps).getByPlaceholderText('Enter vault ID');
		await fireEvent.input(input, { target: { value: 'vault2' } });
		expect(mockGui.setVaultId).toHaveBeenCalledWith(false, 0, 'vault2');
	});

	it('does not call setVaultId when gui is undefined', async () => {
		const propsWithoutGui = {
			...mockProps,
			gui: undefined
		} as unknown as TokenIOInputComponentProps;
		const { getByPlaceholderText } = render(TokenIOInput, propsWithoutGui);
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
		expect(mockGui.getTokenInfo).toHaveBeenCalledWith('0x456');
	});
});
