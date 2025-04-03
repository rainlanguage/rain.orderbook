import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import DeploymentSteps from '../lib/components/deployment/DeploymentSteps.svelte';
import { DotrainOrderGui, type ScenarioCfg } from '@rainlanguage/orderbook/js_api';
import type { ComponentProps } from 'svelte';
import { readable, writable } from 'svelte/store';
import type { AppKit } from '@reown/appkit';
import type { GuiDeploymentCfg } from '@rainlanguage/orderbook/js_api';
import userEvent from '@testing-library/user-event';
import { useGui } from '$lib/hooks/useGui';
import { useAccount } from '$lib/providers/wallet/useAccount';
import * as getDeploymentTransactionArgsModule from '../lib/components/deployment/getDeploymentTransactionArgs';
import type { HandleAddOrderResult } from '../lib/components/deployment/getDeploymentTransactionArgs';
import { handleDeployment } from '../lib/utils/handleDeployment';
import type { Hex } from 'viem';
import type { DeploymentHandlers, handleDeployment } from '../lib/utils/handleDeployment';

const { mockConnectedStore } = await vi.hoisted(() => import('../lib/__mocks__/stores'));

vi.mock('$lib/hooks/useGui', () => ({
	useGui: vi.fn()
}));

vi.mock('$lib/providers/wallet/useAccount', () => ({
	useAccount: vi.fn()
}));

vi.mock('../lib/utils/handleDeployment', () => ({
	handleDeployment: vi.fn()
}));

export type DeploymentStepsProps = ComponentProps<DeploymentSteps>;

const mockHandleAddOrderResult: HandleAddOrderResult = {
	approvals: [],
	deploymentCalldata: '0x123',
	orderbookAddress: '0x456' as Hex,
	chainId: 1337,
	network: 'testnet'
};

const mockDeploymentHandlers: DeploymentHandlers = {
	handleDisclaimerModal: vi.fn(),
	handleDeployModal: vi.fn()
};

const dotrain = `raindex-version: 8898591f3bcaa21dc91dc3b8584330fc405eadfa`

const mockDeployment = {
	key: 'flare-sflr-wflr',
	name: 'SFLR<>WFLR on Flare',
	description: 'Rotate sFLR (Sceptre staked FLR) and WFLR on Flare.',
	deposits: [],
	fields: [],
	select_tokens: [],
	deployment: {
		key: 'flare-sflr-wflr',
		scenario: {
			key: 'flare',
			bindings: {}
		} as ScenarioCfg,
		order: {
			key: 'flare-sflr-wflr',
			network: {
				key: 'flare',
				'chain-id': 14,
				'network-id': 14,
				rpc: 'https://rpc.ankr.com/flare',
				label: 'Flare',
				currency: 'FLR'
			},
			deployer: {
				key: 'flare',
				bindings: {}
			} as ScenarioCfg,
			order: {
				key: 'flare-sflr-wflr',
				network: {
					key: 'flare',
					'chain-id': 14,
					'network-id': 14,
					rpc: 'https://rpc.ankr.com/flare',
					label: 'Flare',
					currency: 'FLR'
				},
				address: '0x0'
			},
			orderbook: {
				id: 'flare',
				address: '0x0'
			},
			inputs: [],
			outputs: []
		}
	}
} as unknown as GuiDeploymentCfg;

const defaultProps = {
	dotrain,
	strategyDetail: {
		name: 'SFLR<>WFLR on Flare',
		description: 'Rotate sFLR (Sceptre staked FLR) and WFLR on Flare.',
		short_description: 'Rotate sFLR (Sceptre staked FLR) and WFLR on Flare.'
	},
	deployment: mockDeployment,
	wagmiConnected: mockConnectedStore,
	appKitModal: writable({} as AppKit),
	deploymentHandlers: mockDeploymentHandlers,
	subgraphUrl: 'https://subgraph.com'
} as DeploymentStepsProps;

describe('DeploymentSteps', () => {
	let guiInstance: DotrainOrderGui;
	let mockGui: DotrainOrderGui;

	beforeEach(() => {
		vi.clearAllMocks();
		guiInstance = new DotrainOrderGui();

		(DotrainOrderGui.prototype.areAllTokensSelected as Mock).mockReturnValue({ value: false });
		(DotrainOrderGui.prototype.getSelectTokens as Mock).mockReturnValue({ value: [] });
		(DotrainOrderGui.prototype.getNetworkKey as Mock).mockReturnValue({ value: 'flare' });
		(DotrainOrderGui.prototype.getCurrentDeployment as Mock).mockReturnValue(mockDeployment);
		(DotrainOrderGui.prototype.getAllFieldDefinitions as Mock).mockReturnValue({ value: [] });
		(DotrainOrderGui.prototype.hasAnyDeposit as Mock).mockReturnValue({ value: false });
		(DotrainOrderGui.prototype.hasAnyVaultId as Mock).mockReturnValue(false);
		(DotrainOrderGui.prototype.getAllTokenInfos as Mock).mockResolvedValue({ value: [] });
		(DotrainOrderGui.prototype.getCurrentDeploymentDetails as Mock).mockReturnValue({
			value: {
				name: 'Test Deployment',
				description: 'This is a test deployment description'
			}
		});
		mockGui = guiInstance;
		vi.mocked(useGui).mockReturnValue(mockGui);
		vi.mocked(useAccount).mockReturnValue({
			account: writable('0x123')
		});
	});

	it('shows deployment details when provided', async () => {
		render(DeploymentSteps, { props: defaultProps });

		await waitFor(() => {
			expect(screen.getByText('SFLR<>WFLR on Flare')).toBeInTheDocument();
		});
	});

	it('correctly derives subgraphUrl from settings and networkKey', async () => {
		(DotrainOrderGui.prototype.areAllTokensSelected as Mock).mockReturnValue({ value: true });
		(DotrainOrderGui.prototype.hasAnyDeposit as Mock).mockReturnValue({ value: false });
		(DotrainOrderGui.prototype.hasAnyVaultId as Mock).mockReturnValue({ value: false });
		(DotrainOrderGui.prototype.getDeploymentTransactionArgs as Mock).mockReturnValue({
			value: {
				approvals: [],
				deploymentCalldata: '0x1',
				orderbookAddress: '0x1',
				chainId: 1
			}
		});
		(useAccount as Mock).mockReturnValue({ account: readable('0xuser') });
		mockConnectedStore.mockSetSubscribeValue(true);

		const user = userEvent.setup();

		render(DeploymentSteps, {
			props: {
				...defaultProps
			}
		});

		// Wait for UI updates after mocks are applied
		await waitFor(() => {
			expect(screen.getByText('Deploy Strategy')).toBeInTheDocument();
		});

		// Click the deploy button
		const deployButton = screen.getByText('Deploy Strategy');
		await user.click(deployButton);

		// Wait for the disclaimer modal to be called
		// Instead, check that the event deploy is emitted with a result.

		await waitFor(() => {
			expect(defaultProps.deploymentHandlers.handleDeployModal).toHaveBeenCalledWith(
				expect.objectContaining({
					args: expect.objectContaining({
						subgraphUrl: expect.any(String)
					})
				})
			);
			const calls = (defaultProps.deploymentHandlers.handleDeployModal as Mock).mock.calls;
			const passedSubgraphUrl = calls[0][0].args.subgraphUrl;
			expect(passedSubgraphUrl).toBe('https://subgraph.com/flare');
		});
	});

	it('shows select tokens section when tokens need to be selected', async () => {
		(DotrainOrderGui.prototype.getSelectTokens as Mock).mockReturnValue({
			value: ['token1', 'token2']
		});

		render(DeploymentSteps, {
			props: defaultProps
		});

		await waitFor(() => {
			expect(screen.getByText('Select Tokens')).toBeInTheDocument();
			expect(
				screen.getByText('Select the tokens that you want to use in your order.')
			).toBeInTheDocument();
		});
	});

	it('shows wallet connect button when all required fields are filled, but no account exists', async () => {
		(useAccount as Mock).mockReturnValue({
			account: writable(null)
		});

		const mockSelectTokens = [
			{ key: 'token1', name: 'Token 1', description: undefined },
			{ key: 'token2', name: 'Token 2', description: undefined }
		];

		// Set up specific mocks for this test
		(DotrainOrderGui.prototype.getSelectTokens as Mock).mockReturnValue({
			value: mockSelectTokens
		});
		(DotrainOrderGui.prototype.getTokenInfo as Mock).mockImplementation(() => {});
		(DotrainOrderGui.prototype.areAllTokensSelected as Mock).mockReturnValue({ value: true });
		(DotrainOrderGui.prototype.isSelectTokenSet as Mock).mockReturnValue({ value: false });
		(DotrainOrderGui.prototype.saveSelectToken as Mock).mockImplementation(() => {});
		(DotrainOrderGui.prototype.getCurrentDeployment as Mock).mockReturnValue({
			value: {
				deployment: {
					order: {
						inputs: [],
						outputs: []
					}
				},
				deposits: []
			}
		});

		(DotrainOrderGui.prototype.getAllTokenInfos as Mock).mockResolvedValue({
			value: [
				{
					address: '0x1',
					decimals: 18,
					name: 'Token 1',
					symbol: 'TKN1'
				},
				{
					address: '0x2',
					decimals: 18,
					name: 'Token 2',
					symbol: 'TKN2'
				}
			]
		});

		render(DeploymentSteps, { props: defaultProps });

		await waitFor(() => {
			expect(screen.getByText('Connect Wallet')).toBeInTheDocument();
		});
	});

	it('shows deploy button when all required fields are filled, and account is connected', async () => {
		(useAccount as Mock).mockReturnValue({
			account: writable('0x123')
		});

		const mockSelectTokens = [
			{ key: 'token1', name: 'Token 1', description: undefined },
			{ key: 'token2', name: 'Token 2', description: undefined }
		];

		// Set up specific mocks for this test
		(DotrainOrderGui.prototype.getSelectTokens as Mock).mockReturnValue({
			value: mockSelectTokens
		});
		(DotrainOrderGui.prototype.getTokenInfo as Mock).mockImplementation(() => {});
		(DotrainOrderGui.prototype.areAllTokensSelected as Mock).mockReturnValue({ value: true });
		(DotrainOrderGui.prototype.isSelectTokenSet as Mock).mockReturnValue({ value: false });
		(DotrainOrderGui.prototype.saveSelectToken as Mock).mockImplementation(() => {});
		(DotrainOrderGui.prototype.getCurrentDeployment as Mock).mockReturnValue({
			value: {
				deployment: {
					order: {
						inputs: [],
						outputs: []
					}
				},
				deposits: []
			}
		});

		(DotrainOrderGui.prototype.getAllTokenInfos as Mock).mockResolvedValue({
			value: [
				{
					address: '0x1',
					decimals: 18,
					name: 'Token 1',
					symbol: 'TKN1'
				},
				{
					address: '0x2',
					decimals: 18,
					name: 'Token 2',
					symbol: 'TKN2'
				}
			]
		});

		render(DeploymentSteps, { props: defaultProps });

		await waitFor(() => {
			expect(screen.getByText('Deploy Strategy')).toBeInTheDocument();
		});
	});
	it('refreshes field descriptions when tokens change', async () => {
		const mockSelectTokens = [
			{ key: 'token1', name: 'Token 1', description: undefined },
			{ key: 'token2', name: 'Token 2', description: undefined }
		];

		// Set up specific mocks for this test
		(DotrainOrderGui.prototype.getSelectTokens as Mock).mockReturnValue({
			value: mockSelectTokens
		});
		(DotrainOrderGui.prototype.getTokenInfo as Mock).mockImplementation(() => {});
		(DotrainOrderGui.prototype.areAllTokensSelected as Mock).mockReturnValue({ value: true });
		(DotrainOrderGui.prototype.isSelectTokenSet as Mock).mockReturnValue({ value: false });
		(DotrainOrderGui.prototype.saveSelectToken as Mock).mockImplementation(() => {});
		(DotrainOrderGui.prototype.getCurrentDeployment as Mock).mockReturnValue({
			value: {
				deployment: {
					order: {
						inputs: [],
						outputs: []
					}
				},
				deposits: []
			}
		});

		(DotrainOrderGui.prototype.getAllTokenInfos as Mock).mockResolvedValue({
			value: [
				{
					address: '0x1',
					decimals: 18,
					name: 'Token 1',
					symbol: 'TKN1'
				},
				{
					address: '0x2',
					decimals: 18,
					name: 'Token 2',
					symbol: 'TKN2'
				}
			]
		});

		const user = userEvent.setup();

		render(DeploymentSteps, {
			props: defaultProps
		});

		expect(mockGui.areAllTokensSelected).toHaveBeenCalled();

		await waitFor(() => {
			expect(screen.getByText('Select Tokens')).toBeInTheDocument();
			expect(screen.getByText('Token 1')).toBeInTheDocument();
			expect(screen.getByText('Token 2')).toBeInTheDocument();
		});

		let selectTokenInput = screen.getAllByRole('textbox')[0];
		(DotrainOrderGui.prototype.getTokenInfo as Mock).mockResolvedValue({
			value: {
				address: '0x1',
				decimals: 18,
				name: 'Token 1',
				symbol: 'TKN1'
			}
		});
		await user.type(selectTokenInput, '0x1');

		const selectTokenOutput = screen.getAllByRole('textbox')[1];
		(DotrainOrderGui.prototype.getTokenInfo as Mock).mockResolvedValue({
			value: {
				address: '0x2',
				decimals: 18,
				name: 'Token 2',
				symbol: 'TKN2'
			}
		});
		await user.type(selectTokenOutput, '0x2');

		await waitFor(() => {
			expect(mockGui.getAllTokenInfos).toHaveBeenCalled();
		});

		selectTokenInput = screen.getAllByRole('textbox')[0];
		(DotrainOrderGui.prototype.getTokenInfo as Mock).mockResolvedValue({
			value: {
				address: '0x3',
				decimals: 18,
				name: 'Token 3',
				symbol: 'TKN3'
			}
		});
		await user.type(selectTokenInput, '0x3');

		(DotrainOrderGui.prototype.getAllTokenInfos as Mock).mockResolvedValue({
			value: [
				{
					address: '0x3',
					decimals: 18,
					name: 'Token 3',
					symbol: 'TKN3'
				},
				{
					address: '0x2',
					decimals: 18,
					name: 'Token 2',
					symbol: 'TKN2'
				}
			]
		});

		await waitFor(() => {
			expect(mockGui.getAllTokenInfos).toHaveBeenCalled();
		});
	});
	it('shows loading state when checking deployment', async () => {
		vi.mocked(getDeploymentTransactionArgsModule.getDeploymentTransactionArgs).mockImplementation(
			() => new Promise((resolve) => setTimeout(() => resolve(mockHandleAddOrderResult), 100))
		);

		render(DeploymentSteps, {
			props: defaultProps
		});

		fireEvent.click(screen.getByText('Deploy Strategy'));

		await waitFor(() => {
			expect(screen.getByText('Checking deployment...')).toBeInTheDocument();
		});
	});

	it('handles deployment flow correctly', async () => {
		const user = userEvent.setup();
		const mockHandleAddOrderResult: HandleAddOrderResult = {
			approvals: [],
			deploymentCalldata: '0x123',
			orderbookAddress: '0x456' as Hex,
			chainId: 1337,
			network: 'testnet'
		};

		vi.mocked(getDeploymentTransactionArgsModule.getDeploymentTransactionArgs).mockResolvedValue(
			mockHandleAddOrderResult
		);

		render(DeploymentSteps, { props: defaultProps });

		const deployButton = screen.getByText('Deploy Strategy');
		await user.click(deployButton);

		await waitFor(() => {
			expect(mockDeploymentHandlers.handleDisclaimerModal).toHaveBeenCalledWith({
				open: true,
				onAccept: expect.any(Function)
			});
		});

		// Simulate accepting the disclaimer
		const onAccept = vi.mocked(mockDeploymentHandlers.handleDisclaimerModal).mock.calls[0][0]
			.onAccept;
		onAccept();

		await waitFor(() => {
			expect(mockDeploymentHandlers.handleDeployModal).toHaveBeenCalledWith({
				open: true,
				args: expect.objectContaining({
					...mockHandleAddOrderResult,
					subgraphUrl: undefined
				})
			});
		});
	});
});
