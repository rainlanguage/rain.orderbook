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
import { handleDeployment } from '../lib/components/deployment/handleDeployment';
import { mockConfig } from '../lib/__mocks__/settings';
import type { DeploymentArgs } from '$lib/types/transaction';
import type { Account } from '$lib/types/account';

vi.mock('@rainlanguage/orderbook', () => ({
	DotrainOrderGui: vi.fn()
}));

const { mockConnectedStore } = await vi.hoisted(() => import('../lib/__mocks__/stores'));

vi.mock('$lib/hooks/useGui', () => ({
	useGui: vi.fn()
}));

vi.mock('../lib/components/deployment/handleDeployment', () => ({
	handleDeployment: vi.fn()
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
				chainId: 14,
				networkId: 14,
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
					chainId: 14,
					networkId: 14,
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
	settings: writable(mockConfig),
	registryUrl: 'https://registry.reown.xyz',
	account: readable('0x123')
} as DeploymentStepsProps;

describe('DeploymentSteps', () => {
	let guiInstance: DotrainOrderGui;
	let mockGui: DotrainOrderGui;

	beforeEach(() => {
		vi.clearAllMocks();

		// Create a mock instance with all the methods
		guiInstance = {
			areAllTokensSelected: vi.fn().mockReturnValue({ value: false }),
			getSelectTokens: vi.fn().mockReturnValue({ value: [] }),
			getNetworkKey: vi.fn().mockReturnValue({ value: 'flare' }),
			getCurrentDeployment: vi.fn().mockReturnValue(mockDeployment),
			getAllFieldDefinitions: vi.fn().mockReturnValue({ value: [] }),
			hasAnyDeposit: vi.fn().mockReturnValue({ value: false }),
			hasAnyVaultId: vi.fn().mockReturnValue(false),
			getAllTokenInfos: vi.fn().mockResolvedValue({ value: [] }),
			getCurrentDeploymentDetails: vi.fn().mockReturnValue({
				value: {
					name: 'Test Deployment',
					description: 'This is a test deployment description'
				}
			}),
			getTokenInfo: vi.fn(),
			isSelectTokenSet: vi.fn().mockReturnValue({ value: false }),
			saveSelectToken: vi.fn(),
			getDeploymentTransactionArgs: vi.fn()
		} as unknown as DotrainOrderGui;

		mockGui = guiInstance;
		vi.mocked(useGui).mockReturnValue(mockGui);
	});

	it('shows deployment details when provided', async () => {
		render(DeploymentSteps, { props: defaultProps });

		await waitFor(() => {
			expect(screen.getByText('SFLR<>WFLR on Flare')).toBeInTheDocument();
		});
	});

	it('correctly derives subgraphUrl from settings and networkKey', async () => {
		(mockGui.areAllTokensSelected as Mock).mockReturnValue({ value: true });
		(mockGui.hasAnyDeposit as Mock).mockReturnValue({ value: false });
		(mockGui.hasAnyVaultId as Mock).mockReturnValue({ value: false });
		(mockGui.getDeploymentTransactionArgs as Mock).mockReturnValue({
			value: {
				approvals: [],
				deploymentCalldata: '0x1',
				orderbookAddress: '0x1',
				chainId: 1
			}
		});
		(mockGui.getNetworkKey as Mock).mockReturnValue({ value: 'mainnet' });

		mockConnectedStore.mockSetSubscribeValue(true);

		const user = userEvent.setup();

		render(DeploymentSteps, defaultProps);

		await waitFor(() => {
			expect(screen.getByText('Deploy Strategy')).toBeInTheDocument();
		});

		const deployButton = screen.getByText('Deploy Strategy');
		await user.click(deployButton);

		await waitFor(() => {
			expect(handleDeployment).toHaveBeenCalledWith(
				mockGui,
				'0x123',
				mockConfig.orderbook.subgraphs.mainnet.url
			);
		});
	});

	it('shows select tokens section when tokens need to be selected', async () => {
		(mockGui.getSelectTokens as Mock).mockReturnValue({
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
		const mockSelectTokens = [
			{ key: 'token1', name: 'Token 1', description: undefined },
			{ key: 'token2', name: 'Token 2', description: undefined }
		];

		// Set up specific mocks for this test
		(mockGui.getSelectTokens as Mock).mockReturnValue({
			value: mockSelectTokens
		});
		(mockGui.getTokenInfo as Mock).mockImplementation(() => {});
		(mockGui.areAllTokensSelected as Mock).mockReturnValue({ value: true });
		(mockGui.isSelectTokenSet as Mock).mockReturnValue({ value: false });
		(mockGui.saveSelectToken as Mock).mockImplementation(() => {});
		(mockGui.getCurrentDeployment as Mock).mockReturnValue({
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

		(mockGui.getAllTokenInfos as Mock).mockResolvedValue({
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

		render(DeploymentSteps, { props: { ...defaultProps, account: readable(null) } });

		await waitFor(() => {
			expect(screen.getByText('Connect')).toBeInTheDocument();
		});
	});

	it('shows deploy button when all required fields are filled, and account is connected', async () => {
		const mockSelectTokens = [
			{ key: 'token1', name: 'Token 1', description: undefined },
			{ key: 'token2', name: 'Token 2', description: undefined }
		];

		// Set up specific mocks for this test
		(mockGui.getSelectTokens as Mock).mockReturnValue({
			value: mockSelectTokens
		});
		(mockGui.getTokenInfo as Mock).mockImplementation(() => {});
		(mockGui.areAllTokensSelected as Mock).mockReturnValue({ value: true });
		(mockGui.isSelectTokenSet as Mock).mockReturnValue({ value: false });
		(mockGui.saveSelectToken as Mock).mockImplementation(() => {});
		(mockGui.getCurrentDeployment as Mock).mockReturnValue({
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

		(mockGui.getAllTokenInfos as Mock).mockResolvedValue({
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
		(mockGui.getSelectTokens as Mock).mockReturnValue({
			value: mockSelectTokens
		});
		(mockGui.getTokenInfo as Mock).mockImplementation(() => {});
		(mockGui.areAllTokensSelected as Mock).mockReturnValue({ value: true });
		(mockGui.isSelectTokenSet as Mock).mockReturnValue({ value: false });
		(mockGui.saveSelectToken as Mock).mockImplementation(() => {});
		(mockGui.getCurrentDeployment as Mock).mockReturnValue({
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

		(mockGui.getAllTokenInfos as Mock).mockResolvedValue({
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
		(mockGui.getTokenInfo as Mock).mockResolvedValue({
			value: {
				address: '0x1',
				decimals: 18,
				name: 'Token 1',
				symbol: 'TKN1'
			}
		});
		await user.type(selectTokenInput, '0x1');

		const selectTokenOutput = screen.getAllByRole('textbox')[1];
		(mockGui.getTokenInfo as Mock).mockResolvedValue({
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
		(mockGui.getTokenInfo as Mock).mockResolvedValue({
			value: {
				address: '0x3',
				decimals: 18,
				name: 'Token 3',
				symbol: 'TKN3'
			}
		});
		await user.type(selectTokenInput, '0x3');

		(mockGui.getAllTokenInfos as Mock).mockResolvedValue({
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
		// Setup fake timers
		vi.useFakeTimers();

		// Mock with a delayed response (using setTimeout)
		vi.mocked(handleDeployment).mockImplementation(() => {
			return new Promise<DeploymentArgs>((resolve) => {
				setTimeout(() => resolve({} as DeploymentArgs), 1000);
			});
		});

		(mockGui.areAllTokensSelected as Mock).mockReturnValue({ value: true });

		const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
		render(DeploymentSteps, { props: defaultProps });

		const deployButton = screen.getByText('Deploy Strategy');
		await user.click(deployButton);

		// Check loading state
		expect(screen.getByText('Checking deployment...')).toBeInTheDocument();
		expect(screen.getByTestId('deploy-button')).toBeDisabled();

		// Fast-forward time to resolve the promise
		await vi.runAllTimersAsync();

		// Check final state
		expect(screen.queryByText('Checking deployment...')).not.toBeInTheDocument();
		expect(screen.getByText('Deploy Strategy')).toBeInTheDocument();
		expect(screen.getByTestId('deploy-button')).not.toBeDisabled();

		// Restore real timers
		vi.useRealTimers();
	});
	it('passes correct arguments to handleDeployment', async () => {
		(mockGui.areAllTokensSelected as Mock).mockReturnValue({ value: true });

		const propsWithMockHandlers = {
			...defaultProps,
			account: readable('0xTestAccount') as Account
		};

		const user = userEvent.setup();
		render(DeploymentSteps, { props: propsWithMockHandlers });

		const deployButton = screen.getByText('Deploy Strategy');
		await user.click(deployButton);

		await waitFor(() => {
			expect(handleDeployment).toHaveBeenCalledTimes(1);

			const [guiArg, accountArg, subgraphUrlArg] = vi.mocked(handleDeployment).mock.calls[0];

			expect(guiArg).toBe(mockGui);
			expect(accountArg).toBe('0xTestAccount');
			expect(subgraphUrlArg).toBe(mockConfig.orderbook.subgraphs.flare.url);
		});
	});
});
