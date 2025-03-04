import { render } from '@testing-library/svelte';
import SelectTokensSection from '../lib/components/deployment/SelectTokensSection.svelte';
import { describe, it, expect, vi } from 'vitest';
import { getViemChain } from '../lib/services/getViemChain';
import type { ComponentProps } from 'svelte';
import type { Chain } from 'viem/chains';

type SelectTokenSectionProps = ComponentProps<SelectTokensSection>;

vi.mock('../lib/services/getViemChain', () => ({
	getViemChain: vi.fn().mockReturnValue({ id: 1 } as Chain)
}));

describe('SelectTokensSection', () => {
	const mockGui = {};
	const mockSelectTokens = [{ id: 1 }, { id: 2 }];
	const mockOnSelectTokenSelect = vi.fn();
	const mockNetworkKey = 'mainnet';

	const defaultProps: SelectTokenSectionProps = {
		gui: mockGui,
		selectTokens: mockSelectTokens,
		onSelectTokenSelect: mockOnSelectTokenSelect,
		tokenList: [{ chainId: 1, address: '0x1', decimals: 18, name: 'test', symbol: 'TEST' }],
		networkKey: mockNetworkKey
	} as unknown as SelectTokenSectionProps;

	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should make a call to getViem get the chain for a given chainId if there are tokens in the list', () => {
		render(SelectTokensSection, {
			props: defaultProps
		});
		expect(getViemChain).toHaveBeenCalledWith(mockNetworkKey);
	});

	it('should not make a call to getViem get the chain for a given chainId if there are no tokens in the list', () => {
		render(SelectTokensSection, {
			props: { ...defaultProps, tokenList: [] }
		});
		expect(getViemChain).not.toHaveBeenCalled();
	});
});
