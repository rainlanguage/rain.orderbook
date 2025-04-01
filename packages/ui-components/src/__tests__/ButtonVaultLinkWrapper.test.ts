import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import ButtonVaultLinkWrapper from './ButtonVaultLinkWrapper.test.svelte';
import type { SgVault } from '@rainlanguage/orderbook/js_api';
import * as viem from 'viem';
import { readable } from 'svelte/store';
import type { CreateQueryResult } from '@tanstack/svelte-query';
import type { ComponentProps } from 'svelte';

type ButtonVaultLinkWrapperProps = ComponentProps<ButtonVaultLinkWrapper>;
// Mock viem functions
vi.mock('viem', async () => {
	const actual = await vi.importActual('viem');
	return {
		...actual,
		isAddress: vi.fn(),
		isAddressEqual: vi.fn()
	};
});

describe('ButtonVaultLinkWrapper', () => {
	const mockVault = {
		id: '123',
		vaultId: '1000',
		balance: '1000000000000000000',
		owner: '0x1234567890123456789012345678901234567890',
		token: {
			name: 'Test Token',
			symbol: 'TEST',
			decimals: '18'
		}
	} as unknown as SgVault;

	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should render vault information correctly without buttons', () => {
		// Mock conditions to not show buttons
		vi.mocked(viem.isAddress).mockReturnValue(false);
		vi.mocked(viem.isAddressEqual).mockReturnValue(false);

		render(ButtonVaultLinkWrapper, {
			props: {
				tokenVault: mockVault,
				subgraphName: 'test',
				signerAddress: '0x0000000000000000000000000000000000000000'
			} as unknown as ButtonVaultLinkWrapperProps
		});

		const vaultLink = screen.getByTestId('vault-link');
		expect(vaultLink).toBeTruthy();
		expect(vaultLink).toHaveTextContent('Test Token');
		expect(vaultLink).toHaveTextContent('TEST');

		// Verify buttons are not rendered
		const buttonSlotContent = screen.queryByTestId('buttons-rendered');
		expect(buttonSlotContent).toBeNull();
	});

	it('should render vault information with buttons when all conditions are met', () => {
		// Mock conditions to show buttons
		vi.mocked(viem.isAddress).mockReturnValue(true);
		vi.mocked(viem.isAddressEqual).mockReturnValue(true);

		// Mock DepositOrWithdrawButtons component
		vi.mock('../lib/components/DepositOrWithdrawButtons.svelte', () => ({
			default: {
				render: () => {
					const div = document.createElement('div');
					div.setAttribute('data-testid', 'deposit-withdraw-buttons');
					div.textContent = 'Mocked Buttons';
					return { component: div };
				}
			}
		}));

		render(ButtonVaultLinkWrapper, {
			props: {
				tokenVault: mockVault,
				subgraphName: 'test',
				signerAddress: '0x1234567890123456789012345678901234567890',
				handleDepositOrWithdrawModal: () => {},
				chainId: 1,
				rpcUrl: 'https://example.com',
				orderDetailQuery: readable({}) as unknown as CreateQueryResult,
				subgraphUrl: 'https://subgraph.example.com'
			}
		});

		const vaultLink = screen.getByTestId('vault-link');
		expect(vaultLink).toBeTruthy();

		// Verify viem.isAddress was called with the correct parameters
		expect(viem.isAddress).toHaveBeenCalledWith('0x1234567890123456789012345678901234567890');
		expect(viem.isAddress).toHaveBeenCalledWith(mockVault.owner);

		// Verify viem.isAddressEqual was called with the correct parameters
		expect(viem.isAddressEqual).toHaveBeenCalledWith(
			'0x1234567890123456789012345678901234567890',
			mockVault.owner
		);

		// Verify buttons container is rendered
		const buttonSlotContent = screen.getByTestId('buttons-rendered');
		expect(buttonSlotContent).toBeTruthy();
	});
});
