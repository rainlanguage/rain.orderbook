import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import DeploymentSteps from '../lib/components/deployment/DeploymentSteps.svelte';
import { DotrainOrderGui, type ScenarioCfg } from '@rainlanguage/orderbook';
import type { ComponentProps } from 'svelte';
import { readable, writable } from 'svelte/store';
import type { AppKit } from '@reown/appkit';
import type { GuiDeploymentCfg } from '@rainlanguage/orderbook';
import userEvent from '@testing-library/user-event';
import { useGui } from '$lib/hooks/useGui';
import { useAccount } from '$lib/providers/wallet/useAccount';
import { mockConfigSource } from '../lib/__mocks__/settings';
import type { DeploymentArgs } from '$lib/types/transaction';

const { mockConnectedStore } = await vi.hoisted(() => import('../lib/__mocks__/stores'));

vi.mock('$lib/hooks/useGui', () => ({
	useGui: vi.fn()
}));

vi.mock('$lib/providers/wallet/useAccount', () => ({
	useAccount: vi.fn()
}));

type DeploymentStepsProps = ComponentProps<DeploymentSteps>;

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

const mockOnDeploy = vi.fn();

const defaultProps: DeploymentStepsProps = {
	strategyDetail: {
		name: 'SFLR<>WFLR on Flare',
		description: 'Rotate sFLR (Sceptre staked FLR) and WFLR on Flare.',
		short_description: 'Rotate sFLR (Sceptre staked FLR) and WFLR on Flare.'
	},
	deployment: mockDeployment,
	wagmiConnected: mockConnectedStore,
	appKitModal: writable({} as AppKit),
	onDeploy: mockOnDeploy,
	settings: writable(mockConfigSource),
	registryUrl: 'https://registry.reown.xyz'
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
			account: readable('0x123'),
			matchesAccount: vi.fn()
		});
	});

	it('shows deployment details when provided', async () => {
		render(DeploymentSteps, { props: defaultProps });

		await waitFor(() => {
			expect(screen.getByText('SFLR<>WFLR on Flare')).toBeInTheDocument();
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
			expect(screen.getByText('Connect')).toBeInTheDocument();
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
	it('passes correct arguments to onDeploy prop', async () => {
		vi.mocked(useAccount).mockReturnValue({
			account: readable('0xTestAccount'),
			matchesAccount: vi.fn()
		});

		(DotrainOrderGui.prototype.areAllTokensSelected as Mock).mockReturnValue({ value: true });

		const propsWithMockHandlers = {
			...defaultProps
		};

		const user = userEvent.setup();
		render(DeploymentSteps, { props: propsWithMockHandlers });

		const deployButton = screen.getByText('Deploy Strategy');
		await user.click(deployButton);

		await waitFor(() => {
			expect(mockOnDeploy).toHaveBeenCalledTimes(1);

			const [guiArg, subgraphUrlArg] = mockOnDeploy.mock.calls[0];

			expect(guiArg).toBe(mockGui);
			const expectedSubgraphUrl = mockConfigSource.subgraphs?.flare;
			expect(subgraphUrlArg).toBe(expectedSubgraphUrl);
		});
	});
});
