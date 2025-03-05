import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, it, vi, beforeEach, expect } from 'vitest';
import truncateEthAddress from 'truncate-eth-address';
import WalletConnect from '../lib/components/wallet/WalletConnect.svelte';

const { mockWagmiConfigStore, mockSignerAddressStore, mockConnectedStore, mockAppKitModalStore } =
	await vi.hoisted(() => import('../lib/__mocks__/stores'));

vi.mock('../lib/stores/wagmi', () => ({
	appKitModal: mockAppKitModalStore,
	wagmiConfig: mockWagmiConfigStore,
	useSignerAddress: vi.fn().mockReturnValue({
		signerAddress: mockSignerAddressStore,
		connected: mockConnectedStore
	})
}));

describe('WalletConnect component', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('displays "Connect" with red icon when wallet is not connected or wrong network', () => {
		mockSignerAddressStore.mockSetSubscribeValue('');
		mockConnectedStore.mockSetSubscribeValue(false);

		render(WalletConnect);

		const connectButton = screen.getByTestId('wallet-connect');
		expect(connectButton).toBeInTheDocument();
	});

	it('displays truncated version of the connected address, when a wallet is connected', async () => {
		mockSignerAddressStore.mockSetSubscribeValue('0x912ce59144191c1204e64559fe8253a0e49e6548');
		mockConnectedStore.mockSetSubscribeValue(true);

		render(WalletConnect);

		await waitFor(() => {
			expect(
				screen.getByText(truncateEthAddress('0x912ce59144191c1204e64559fe8253a0e49e6548'))
			).toBeInTheDocument();
		});
	});
});
