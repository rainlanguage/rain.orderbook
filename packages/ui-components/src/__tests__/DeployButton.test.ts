import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import DeployButton from '../lib/components/deployment/DeployButton.svelte';
import { DeploymentStepsError } from '../lib/errors';
import * as getDeploymentTransactionArgsModule from '../lib/components/deployment/getDeploymentTransactionArgs';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import { useGui } from '../lib/hooks/useGui';
import { type HandleAddOrderResult } from '../lib/components/deployment/getDeploymentTransactionArgs';
import type { ComponentProps } from 'svelte';
import type { DeployModalProps, DisclaimerModalProps } from '../lib/types/modal';

type DeployButtonProps = ComponentProps<DeployButton>;

const { mockWagmiConfigStore } = await vi.hoisted(() => import('../lib/__mocks__/stores'));

const defaultProps: DeployButtonProps = {
	handleDeployModal: vi.fn() as (args: DeployModalProps) => void,
	handleDisclaimerModal: vi.fn() as (args: DisclaimerModalProps) => void,
	subgraphUrl: 'https://test.subgraph',
	network: 'testnet',
	wagmiConfig: mockWagmiConfigStore
};

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

vi.mock('../lib/components/deployment/getDeploymentTransactionArgs', () => ({
	getDeploymentTransactionArgs: vi.fn()
}));

describe('DeployButton', () => {
	let mockGui: DotrainOrderGui;

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

		DeploymentStepsError.clear();
	});

	it('renders the deploy button correctly', () => {
		render(DeployButton, {
			props: defaultProps
		});

		expect(screen.getByText('Deploy Strategy')).toBeInTheDocument();
	});

	it('shows loading state when checking deployment', async () => {
		vi.mocked(getDeploymentTransactionArgsModule.getDeploymentTransactionArgs).mockImplementation(
			() => new Promise((resolve) => setTimeout(() => resolve(mockHandleAddOrderResult), 100))
		);

		render(DeployButton, {
			props: defaultProps
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
			props: defaultProps
		});

		fireEvent.click(screen.getByText('Deploy Strategy'));

		await waitFor(() => {
			expect(screen.getByText('Checking deployment...')).toBeInTheDocument();
		});
	});

	it('handles deployment check errors correctly', async () => {
		const mockError = new Error('Deployment check failed');
		vi.mocked(getDeploymentTransactionArgsModule.getDeploymentTransactionArgs).mockRejectedValue(
			mockError
		);

		const catchSpy = vi.spyOn(DeploymentStepsError, 'catch');

		render(DeployButton, {
			props: defaultProps
		});

		fireEvent.click(screen.getByText('Deploy Strategy'));

		await waitFor(() => {
			expect(catchSpy).toHaveBeenCalledWith(mockError, expect.any(String));
			expect(screen.getByText('Deploy Strategy')).toBeInTheDocument();
		});
	});

	it('opens the deploy modal when disclaimer is accepted', async () => {
		const props = { ...defaultProps };
		vi.mocked(getDeploymentTransactionArgsModule.getDeploymentTransactionArgs).mockResolvedValue(
			mockHandleAddOrderResult
		);

		render(DeployButton, { props });

		fireEvent.click(screen.getByText('Deploy Strategy'));

		await waitFor(() => {
			expect(props.handleDisclaimerModal).toHaveBeenCalledWith({
				open: true,
				onAccept: expect.any(Function)
			});
		});

		// Get the onAccept callback from the disclaimer modal call
		const onAccept = vi.mocked(props.handleDisclaimerModal).mock.calls[0][0].onAccept;
		onAccept();

		expect(props.handleDeployModal).toHaveBeenCalledWith({
			open: true,
			args: {
				...mockHandleAddOrderResult,
				subgraphUrl: 'https://test.subgraph',
				chainId: 1337,
				network: 'testnet'
			}
		});
	});

	it('gets the orderbook network from the GUI', () => {
		render(DeployButton, {
			props: defaultProps
		});

		expect(mockGui.getOrderbookNetwork).toHaveBeenCalled();
	});
});
