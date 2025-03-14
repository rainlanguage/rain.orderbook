import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import DeployButton from '../lib/components/deployment/DeployButton.svelte';
import { DeploymentStepsError } from '../lib/errors';
import * as getDeploymentTransactionArgsModule from '../lib/components/deployment/getDeploymentTransactionArgs';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import { useGui } from '../lib/hooks/useGui';
import { type HandleAddOrderResult } from '../lib/components/deployment/getDeploymentTransactionArgs';
import type { ComponentProps } from 'svelte';
import type { Config } from '@wagmi/core';
import { writable, type Writable } from 'svelte/store';
import type { DeployModalProps, DisclaimerModalProps } from '../lib/types/modal';

// Define the component props type
type DeployButtonProps = ComponentProps<DeployButton>;

// Create a mock wagmi config store
const mockWagmiConfigStore = writable<Config>({ mockWagmiConfig: true } as unknown as Config);

// Define default props object
const defaultProps: DeployButtonProps = {
	handleDeployModal: vi.fn() as (args: DeployModalProps) => void,
	handleDisclaimerModal: vi.fn() as (args: DisclaimerModalProps) => void,
	subgraphUrl: 'https://test.subgraph',
	network: 'testnet',
	wagmiConfig: mockWagmiConfigStore
};

// Mock result for deployment transaction args
const mockHandleAddOrderResult: HandleAddOrderResult = {
	approvals: [],
	deploymentCalldata: '0x123',
	orderbookAddress: '0x456',
	chainId: 1337
};

// Mocks
vi.mock('../lib/hooks/useGui', () => ({
	useGui: vi.fn()
}));

vi.mock('../lib/stores/wagmi', () => ({
	wagmiConfig: {
		subscribe: vi.fn((callback) => {
			callback({ mockWagmiConfig: true });
			return () => {};
		})
	}
}));

vi.mock('../lib/components/deployment/getDeploymentTransactionArgs', () => ({
	getDeploymentTransactionArgs: vi.fn()
}));

describe('DeployButton', () => {
	let mockGui: DotrainOrderGui;
	let mockHandleDeployModal: ReturnType<typeof vi.fn>;
	let mockHandleDisclaimerModal: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		vi.clearAllMocks();

		// Create a fresh GUI mock for each test
		mockGui = {
			getOrderbookNetwork: vi.fn().mockReturnValue({
				key: 'testnet',
				chainId: 1337,
				networkId: 1337,
				rpc: 'https://test.rpc',
				label: 'Test Network',
				currency: 'TEST'
			})
		} as unknown as DotrainOrderGui;

		vi.mocked(useGui).mockReturnValue(mockGui);

		mockHandleDeployModal = vi.fn();
		mockHandleDisclaimerModal = vi.fn();

		DeploymentStepsError.clear();
	});

	it('renders the deploy button correctly', () => {
		render(DeployButton, {
			props: {
				handleDeployModal: mockHandleDeployModal,
				handleDisclaimerModal: mockHandleDisclaimerModal,
				subgraphUrl: 'https://test.subgraph',
				network: 'testnet',
				wagmiConfig: mockWagmiConfigStore
			}
		});

		expect(screen.getByText('Deploy Strategy')).toBeInTheDocument();
	});

	it('shows loading state when checking deployment', async () => {
		vi.mocked(getDeploymentTransactionArgsModule.getDeploymentTransactionArgs).mockImplementation(
			() => new Promise((resolve) => setTimeout(() => resolve(mockHandleAddOrderResult), 100))
		);

		render(DeployButton, {
			props: {
				handleDeployModal: mockHandleDeployModal,
				handleDisclaimerModal: mockHandleDisclaimerModal,
				subgraphUrl: 'https://test.subgraph'
			}
		});

		fireEvent.click(screen.getByText('Deploy Strategy'));

		await waitFor(() => {
			expect(screen.getByText('Checking deployment...')).toBeInTheDocument();
		});
	});

	it('opens the disclaimer modal when deployment check succeeds', async () => {
		const mockResult = { mockResult: true };
		vi.mocked(getDeploymentTransactionArgsModule.getDeploymentTransactionArgs).mockResolvedValue(
			mockResult as unknown as HandleAddOrderResult
		);

		render(DeployButton, {
			props: {
				handleDeployModal: mockHandleDeployModal,
				handleDisclaimerModal: mockHandleDisclaimerModal,
				subgraphUrl: 'https://test.subgraph',
				network: 'testnet',
				wagmiConfig: mockWagmiConfigStore
			}
		});

		fireEvent.click(screen.getByText('Deploy Strategy'));

		await waitFor(() => {
			expect(mockHandleDisclaimerModal).toHaveBeenCalledWith({
				open: true,
				onAccept: expect.any(Function)
			});
		});
	});

	it('handles deployment check errors correctly', async () => {
		const mockError = new Error('Deployment check failed');
		vi.mocked(getDeploymentTransactionArgsModule.getDeploymentTransactionArgs).mockRejectedValue(
			mockError
		);

		const catchSpy = vi.spyOn(DeploymentStepsError, 'catch');

		render(DeployButton, {
			props: {
				handleDeployModal: mockHandleDeployModal,
				handleDisclaimerModal: mockHandleDisclaimerModal,
				subgraphUrl: 'https://test.subgraph',
				network: 'testnet',
				wagmiConfig: mockWagmiConfigStore
			}
		});

		fireEvent.click(screen.getByText('Deploy Strategy'));

		await waitFor(() => {
			expect(catchSpy).toHaveBeenCalledWith(mockError, expect.any(String));
			expect(screen.getByText('Deploy Strategy')).toBeInTheDocument();
		});
	});

	it('opens the deploy modal when disclaimer is accepted', async () => {
		const mockResult = { mockResult: true };
		vi.mocked(getDeploymentTransactionArgsModule.getDeploymentTransactionArgs).mockResolvedValue(
			mockResult as unknown as HandleAddOrderResult
		);

		render(DeployButton, {
			props: {
				handleDeployModal: mockHandleDeployModal,
				handleDisclaimerModal: mockHandleDisclaimerModal,
				subgraphUrl: 'https://test.subgraph',
				network: 'testnet',
				wagmiConfig: mockWagmiConfigStore
			}
		});

		fireEvent.click(screen.getByText('Deploy Strategy'));

		await waitFor(() => {
			expect(mockHandleDisclaimerModal).toHaveBeenCalled();
		});

		const onAccept = mockHandleDisclaimerModal.mock.calls[0][0].onAccept;

		onAccept();

		expect(mockHandleDeployModal).toHaveBeenCalledWith({
			open: true,
			args: {
				...mockResult,
				subgraphUrl: 'https://test.subgraph',
				chainId: 1337,
				network: 'testnet'
			}
		});
	});

	it('gets the orderbook network from the GUI', () => {
		render(DeployButton, {
			props: {
				handleDeployModal: mockHandleDeployModal,
				handleDisclaimerModal: mockHandleDisclaimerModal,
				subgraphUrl: 'https://test.subgraph',
				network: 'testnet',
				wagmiConfig: mockWagmiConfigStore
			}
		});

		expect(mockGui.getOrderbookNetwork).toHaveBeenCalled();
	});
});
