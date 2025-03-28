import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach, type Mock } from 'vitest';
import DeployButton from '../lib/components/deployment/DeployButton.svelte';
import { DeploymentStepsError, DeploymentStepsErrorCode } from '../lib/errors';
import * as getDeploymentTransactionArgsModule from '../lib/components/deployment/getDeploymentTransactionArgs';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import { type HandleAddOrderResult } from '../lib/components/deployment/getDeploymentTransactionArgs';
import type { ComponentProps } from 'svelte';
import { mockWeb3Config } from '$lib/__mocks__/mockWeb3Config';
import { useGui } from '../lib/hooks/useGui';

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
	wagmiConfig: mockWagmiConfigStore,
	subgraphUrl: 'https://test.subgraph',
	testId: 'deploy-button'
};

vi.mock('../lib/components/deployment/getDeploymentTransactionArgs', () => ({
	getDeploymentTransactionArgs: vi.fn()
}));

vi.mock('../lib/hooks/useGui', () => ({
	useGui: vi.fn()
}));

describe('DeployButton', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		DeploymentStepsError.clear();
		(useGui as Mock).mockReturnValue(mockGui);
	});

	it('renders the deploy button correctly', () => {
		render(DeployButton, {
			props: defaultProps
		});

		expect(screen.getByText('Deploy Strategy')).toBeInTheDocument();
		expect(screen.getByTestId('deploy-button')).toBeInTheDocument();
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

	it('dispatches event with correct data when deployment check succeeds', async () => {
		vi.mocked(getDeploymentTransactionArgsModule.getDeploymentTransactionArgs).mockResolvedValue(
			mockHandleAddOrderResult
		);

		const { component } = render(DeployButton, { props: defaultProps });

		const mockDispatch = vi.fn();
		component.$on('clickDeploy', mockDispatch);

		fireEvent.click(screen.getByText('Deploy Strategy'));

		await waitFor(() => {
			expect(mockDispatch).toHaveBeenCalledWith(
				expect.objectContaining({
					detail: {
						result: mockHandleAddOrderResult,
						networkKey: 'testnet',
						subgraphUrl: defaultProps.subgraphUrl
					}
				})
			);
		});
	});

	it('calls getDeploymentTransactionArgs with correct arguments', async () => {
		render(DeployButton, { props: defaultProps });

		fireEvent.click(screen.getByText('Deploy Strategy'));

		await waitFor(() => {
			expect(getDeploymentTransactionArgsModule.getDeploymentTransactionArgs).toHaveBeenCalledWith(
				mockGui,
				mockWeb3Config
			);
		});
	});

	it('applies custom testId when provided', () => {
		render(DeployButton, {
			props: {
				...defaultProps,
				testId: 'custom-test-id'
			}
		});

		expect(screen.getByTestId('custom-test-id')).toBeInTheDocument();
	});

	it('handles failed deployment transaction args correctly', async () => {
		const mockError = new Error('error getting args');
		vi.mocked(getDeploymentTransactionArgsModule.getDeploymentTransactionArgs).mockRejectedValue(
			mockError
		);

		const catchSpy = vi.spyOn(DeploymentStepsError, 'catch');

		render(DeployButton, { props: defaultProps });

		fireEvent.click(screen.getByText('Deploy Strategy'));

		await waitFor(() => {
			expect(catchSpy).toHaveBeenCalledWith(mockError, DeploymentStepsErrorCode.ADD_ORDER_FAILED);
			expect(screen.getByText('Deploy Strategy')).toBeInTheDocument();
		});
	});
});
