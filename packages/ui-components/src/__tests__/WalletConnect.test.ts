import { render, screen } from '@testing-library/svelte';
import WalletConnect from '../lib/components/wallet/WalletConnect.svelte';
import { describe, it, vi, beforeEach, expect } from 'vitest';
import truncateEthAddress from 'truncate-eth-address';
import type { ComponentProps } from 'svelte';

type WalletConnectProps = ComponentProps<WalletConnect>;

const { mockSignerAddressStore, mockConnectedStore, mockAppKitModalStore } = await vi.hoisted(
	() => import('$lib/__mocks__/stores')
);

vi.mock('../lib/stores/wagmi', async () => {
	return {
		appKitModal: mockAppKitModalStore,
		connected: mockConnectedStore,
		signerAddress: mockSignerAddressStore
	};
});

const defaultProps: WalletConnectProps = {
	connected: mockConnectedStore,
	signerAddress: mockSignerAddressStore,
	classes: '',
	appKitModal: mockAppKitModalStore
};

describe('WalletConnect component', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetAllMocks();
	});

	it('displays "Connect" with red icon when wallet is not connected or wrong network', () => {
		mockSignerAddressStore.mockSetSubscribeValue('');
		mockConnectedStore.mockSetSubscribeValue(false);

		render(WalletConnect, { props: defaultProps });

		const connectButton = screen.getByTestId('wallet-connect');
		expect(connectButton).toBeInTheDocument();
	});

	it('displays truncated version of the connected address, when a wallet is connected', () => {
		mockSignerAddressStore.mockSetSubscribeValue('0x912ce59144191c1204e64559fe8253a0e49e6548');
		mockConnectedStore.mockSetSubscribeValue(true);

		render(WalletConnect, { props: defaultProps });

		expect(
			screen.getByText(truncateEthAddress('0x912ce59144191c1204e64559fe8253a0e49e6548'))
		).toBeInTheDocument();
	});
});
