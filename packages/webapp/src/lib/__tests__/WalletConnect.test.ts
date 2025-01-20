import { render, screen } from '@testing-library/svelte';
import WalletConnect from '../components/WalletConnect.svelte';
import { describe, it, vi, beforeEach, expect } from 'vitest';

const { mockSignerAddressStore, mockConnectedStore, mockAppKitModalStore } = await vi.hoisted(
	() => import('../__mocks__/stores')
);

vi.mock('$lib/stores/wagmi', async (importOriginal) => {
	const original = (await importOriginal()) as object;
	return {
		...original,
		appKitModal: mockAppKitModalStore,
		connected: mockConnectedStore
	};
});


describe('WalletConnect component', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetAllMocks();
	});

	it('displays "Connect" with red icon when wallet is not connected or wrong network', () => {
		mockSignerAddressStore.mockSetSubscribeValue('');
		mockConnectedStore.mockSetSubscribeValue(false);
		render(WalletConnect);

		const connectButton = screen.getByTestId('wallet-connect');
		expect(connectButton).toBeInTheDocument();
		expect(screen.getByTestId('not-connected')).toBeInTheDocument();
	});

	it('displays "Connected" with green icon when wallet is connected', () => {
		mockSignerAddressStore.mockSetSubscribeValue('0x123');
		mockConnectedStore.mockSetSubscribeValue(true);

		render(WalletConnect);

		const connectButton = screen.getByTestId('wallet-connect');
		expect(connectButton).toBeInTheDocument();
		expect(screen.getByTestId('connected')).toBeInTheDocument();
	});
});
