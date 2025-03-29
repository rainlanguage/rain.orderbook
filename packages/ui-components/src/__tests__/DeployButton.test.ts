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
	getNetworkKey: vi.fn().mockReturnValue({
		value: 'testnet',
		error: null
	}),
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

	it('handles error from getNetworkKey correctly', async () => {
		const mockNetworkKeyError = new Error('Network key error');
		const mockGuiWithError = {
			getNetworkKey: vi.fn().mockReturnValue({ error: mockNetworkKeyError }),
			generateDotrainText: vi.fn(),
			getCurrentDeployment: vi.fn()
		};

		vi.mocked(useGui).mockReturnValue(mockGuiWithError as unknown as DotrainOrderGui);

		const catchSpy = vi.spyOn(DeploymentStepsError, 'catch');

		render(DeployButton, {
			props: {
				wagmiConfig: mockWagmiConfigStore,
				subgraphUrl: 'https://test.subgraph',
				testId: 'deploy-button'
			}
		});

		fireEvent.click(screen.getByText('Deploy Strategy'));

		await waitFor(() => {
			expect(catchSpy).toHaveBeenCalledWith(
				mockNetworkKeyError,
				DeploymentStepsErrorCode.NO_NETWORK_KEY
			);
			expect(screen.getByText('Deploy Strategy')).toBeInTheDocument();
		});
	});

	it('proceeds with deployment when getNetworkKey returns a value', async () => {
		const mockGuiWithValue = {
			getNetworkKey: vi.fn().mockReturnValue({
				value: 'custom-network-key',
				error: null
			}),
			generateDotrainText: vi.fn(),
			getCurrentDeployment: vi.fn().mockReturnValue({
				deployment: {
					order: {
						orderbook: {
							address: '0x456'
						}
					}
				}
			})
		};

		vi.mocked(useGui).mockReturnValue(mockGuiWithValue as unknown as DotrainOrderGui);

		vi.mocked(getDeploymentTransactionArgsModule.getDeploymentTransactionArgs).mockResolvedValue(
			mockHandleAddOrderResult
		);

		const { component } = render(DeployButton, {
			props: {
				wagmiConfig: mockWagmiConfigStore,
				subgraphUrl: 'https://test.subgraph',
				testId: 'deploy-button'
			}
		});

		const mockDispatch = vi.fn();
		component.$on('clickDeploy', mockDispatch);

		fireEvent.click(screen.getByText('Deploy Strategy'));

		await waitFor(() => {
			expect(mockDispatch).toHaveBeenCalledWith(
				expect.objectContaining({
					detail: expect.objectContaining({
						networkKey: 'custom-network-key'
					})
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
