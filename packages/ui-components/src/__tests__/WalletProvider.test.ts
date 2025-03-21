import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/svelte';
import WalletProvider, {
	ACCOUNT_KEY,
	USE_ACCOUNT_KEY,
	type UseAccount
} from '../lib/providers/wallet/WalletProvider.svelte';
import { setContext } from 'svelte';
import { readable } from 'svelte/store';

vi.mock('svelte', () => ({
	getContext: vi.fn(),
	setContext: vi.fn()
}));

describe('WalletProvider', () => {
	it('should set account store in context', () => {
		const mockAccount = readable('0x123');

		render(WalletProvider, {
			props: {
				account: mockAccount
			}
		});

		expect(vi.mocked(setContext)).toHaveBeenCalledWith(ACCOUNT_KEY, mockAccount);
	});

	it('should set useAccount function in context', () => {
		const mockAccount = readable('0x123');

		render(WalletProvider, {
			props: {
				account: mockAccount
			}
		});

		expect(vi.mocked(setContext)).toHaveBeenCalledWith(USE_ACCOUNT_KEY, expect.any(Function));
	});

	it('should use default null account when no account prop provided', () => {
		render(WalletProvider);

		const setContextCalls = vi.mocked(setContext).mock.calls;
		const accountCall = setContextCalls.find((call) => call[0] === ACCOUNT_KEY);
		const defaultAccount = accountCall![1];

		expect(defaultAccount).toBeDefined();
	});
});
