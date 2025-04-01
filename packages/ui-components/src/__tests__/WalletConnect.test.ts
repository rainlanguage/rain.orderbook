import { render, screen } from '@testing-library/svelte';
import WalletConnect from '../lib/components/wallet/WalletConnect.svelte';
import { describe, it, vi, beforeEach, expect } from 'vitest';
import { readable, writable, type Writable } from 'svelte/store';
import type { AppKit } from '@reown/appkit';
import truncateEthAddress from 'truncate-eth-address';
import type { Hex } from 'viem';
const {mockConnectedStore } = await vi.hoisted(
	() => import('$lib/__mocks__/stores')
);

vi.mock('$lib/stores/wagmi', async (importOriginal) => {
	const original = (await importOriginal()) as object;
	return {
		...original,
		appKitModal: writable({} as AppKit),
		connected: mockConnectedStore,
	};
});

describe('WalletConnect component', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetAllMocks();
	});

	it('displays "Connect" with red icon when wallet is not connected or wrong network', () => {
		mockConnectedStore.mockSetSubscribeValue(false);

		render(WalletConnect);

		const connectButton = screen.getByTestId('wallet-connect');
		expect(connectButton).toBeInTheDocument();
	});

	it('displays truncated address when wallet is connected', () => {
		mockConnectedStore.mockSetSubscribeValue(true);

		render(WalletConnect, {
			props: {
				connected: mockConnectedStore as Writable<boolean>,
				appKitModal: writable({} as AppKit),
				signerAddress: readable('0x123' as Hex)
			}
		});

		expect(screen.getByText(truncateEthAddress('0x123'))).toBeInTheDocument();
	});
});
