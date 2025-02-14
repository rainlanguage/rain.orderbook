import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import DisclaimerModal from '../lib/components/deployment/DisclaimerModal.svelte';
import { getDeploymentTransactionArgs } from '../lib/components/deployment/getDeploymentTransactionArgs';
import { writable } from 'svelte/store';
import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import type { OrderIO } from '@rainlanguage/orderbook/js_api';

vi.mock('./getDeploymentTransactionArgs', () => ({
	getDeploymentTransactionArgs: vi.fn()
}));

describe('DisclaimerModal', () => {
	const mockGui = {} as DotrainOrderGui;
	const mockOutputs = [] as OrderIO[];
	const mockWagmiConfig = writable(undefined);
	const mockHandleDeployModal = vi.fn();

	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('calls getDeploymentTransactionArgs when opened', async () => {
		const mockResult = {
			approvals: {},
			deploymentCalldata: {},
			orderbookAddress: '0x123',
			chainId: 1
		};
		(getDeploymentTransactionArgs as Mock).mockResolvedValue(mockResult);

		render(DisclaimerModal, {
			props: {
				open: true,
				gui: mockGui,
				allTokenOutputs: mockOutputs,
				wagmiConfig: mockWagmiConfig,
				handleDeployModal: mockHandleDeployModal
			}
		});

		expect(getDeploymentTransactionArgs).toHaveBeenCalledWith(mockGui, undefined, mockOutputs);
	});

	it('shows error message when getDeploymentTransactionArgs fails', async () => {
		const errorMessage = 'Test error message';
		(getDeploymentTransactionArgs as Mock).mockRejectedValue(new Error(errorMessage));

		render(DisclaimerModal, {
			props: {
				open: true,
				gui: mockGui,
				allTokenOutputs: mockOutputs,
				wagmiConfig: mockWagmiConfig,
				handleDeployModal: mockHandleDeployModal
			}
		});

		const errorText = await screen.findByText('Error getting deployment transaction data:');
		const errorDetails = await screen.findByText(errorMessage);
		expect(errorText).toBeInTheDocument();
		expect(errorDetails).toBeInTheDocument();
	});

	it('calls handleDeployModal with result when accepting disclaimer', async () => {
		const mockResult = {
			approvals: {},
			deploymentCalldata: {},
			orderbookAddress: '0x123',
			chainId: 1
		};
		(getDeploymentTransactionArgs as Mock).mockResolvedValue(mockResult);

		render(DisclaimerModal, {
			props: {
				open: true,
				gui: mockGui,
				allTokenOutputs: mockOutputs,
				wagmiConfig: mockWagmiConfig,
				handleDeployModal: mockHandleDeployModal
			}
		});

		const deployButton = await screen.findByText('Deploy');
		await fireEvent.click(deployButton);

		expect(mockHandleDeployModal).toHaveBeenCalledWith(mockResult);
	});
});
