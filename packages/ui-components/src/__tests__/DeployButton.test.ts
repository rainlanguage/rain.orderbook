import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import DeployButton from '../lib/components/deployment/DeployButton.svelte';
import { DeploymentStepsError, DeploymentStepsErrorCode } from '../lib/errors';
import * as getDeploymentTransactionArgsModule from '../lib/components/deployment/getDeploymentTransactionArgs';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import { type HandleAddOrderResult } from '../lib/components/deployment/getDeploymentTransactionArgs';
import type { ComponentProps } from 'svelte';
import type { DeployModalProps, DisclaimerModalProps } from '../lib/types/modal';
import { mockWeb3Config } from '$lib/__mocks__/mockWeb3Config';

type DeployButtonProps = ComponentProps<DeployButton>;

const { mockWagmiConfigStore } = await vi.hoisted(() => import('../lib/__mocks__/stores'));

const mockHandleAddOrderResult: HandleAddOrderResult = {
	approvals: [],
	deploymentCalldata: '0x123',
	orderbookAddress: '0x456',
	chainId: 1337
};

const mockGui = {
	getNetworkKey: vi.fn().mockReturnValue('testnet'),
	generateDotrainText: vi.fn().mockReturnValue('mock dotrain text'),
	getCurrentDeployment: vi.fn().mockReturnValue({
		deployment: {
			order: {
				orderbook: {
					address: '0x456'
				}
			}
		}
	})
} as unknown as DotrainOrderGui;

const defaultProps: DeployButtonProps = {
	handleDeployModal: vi.fn() as (args: DeployModalProps) => void,
	handleDisclaimerModal: vi.fn() as (args: DisclaimerModalProps) => void,
	wagmiConfig: mockWagmiConfigStore,
	gui: mockGui,
	subgraphUrl: 'https://test.subgraph'
};

vi.mock('../lib/components/deployment/getDeploymentTransactionArgs', () => ({
	getDeploymentTransactionArgs: vi.fn()
}));

describe('DeployButton', () => {
	beforeEach(() => {
		vi.clearAllMocks();
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

	it('calls DeploymentStepsError.catch when deployment check fails', async () => {
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
			expect(catchSpy).toHaveBeenCalledWith(mockError, DeploymentStepsErrorCode.ADD_ORDER_FAILED);
			expect(screen.getByText('Deploy Strategy')).toBeInTheDocument();
		});
	});

	it('opens the deploy modal with correct args when disclaimer is accepted', async () => {
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
				subgraphUrl: props.subgraphUrl,
				network: 'testnet'
			}
		});
	});

	it('calls getDeploymentTransactionArgs with correct arguments', async () => {
		render(DeployButton, { props: defaultProps });

		fireEvent.click(screen.getByText('Deploy Strategy'));

		await waitFor(() => {
			expect(getDeploymentTransactionArgsModule.getDeploymentTransactionArgs).toHaveBeenCalledWith(
				defaultProps.gui,
				mockWeb3Config
			);
		});
	});

	it('does not open the deploy modal when disclaimer is rejected', async () => {
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

		// Get the onAccept callback but don't call it (simulating rejection)
		// Either by closing the modal or clicking a reject button

		// Verify the deploy modal was never opened
		expect(props.handleDeployModal).not.toHaveBeenCalled();
	});
});
