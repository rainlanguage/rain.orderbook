import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import DeploymentSteps from '../lib/components/deployment/DeploymentSteps.svelte';
import { DotrainOrderGui, type ScenarioCfg } from '@rainlanguage/orderbook/js_api';
import type { ComponentProps } from 'svelte';
import { writable } from 'svelte/store';
import type { ConfigSource, GuiDeploymentCfg } from '@rainlanguage/orderbook/js_api';
import type { DeployModalProps, DisclaimerModalProps } from '../lib/types/modal';
import userEvent from '@testing-library/user-event';

import { useGui } from '$lib/hooks/useGui';
import type { AppKit } from '@reown/appkit';

vi.mock('$lib/hooks/useGui', () => ({
	useGui: vi.fn()
}));

const { mockWagmiConfigStore, mockConnectedStore, mockSignerAddressStore } = await vi.hoisted(
	() => import('../lib/__mocks__/stores')
);

vi.mock('../lib/stores/wagmi', async (importOriginal) => {
	const original = (await importOriginal()) as object;
	return {
		...original,
		connected: mockConnectedStore,
		signerAddress: mockSignerAddressStore,
		wagmiConfig: mockWagmiConfigStore,
		appKitModal: writable({} as AppKit)
	};
});

vi.mock('../lib/components/deployment/DeployButton.svelte', async () => {
	const MockDeployButton = await import('../lib/__mocks__/MockComponent.svelte');
	return { default: MockDeployButton };
});

export type DeploymentStepsProps = ComponentProps<DeploymentSteps>;

vi.mock('@rainlanguage/orderbook/js_api', () => ({
	DotrainOrderGui: {
		chooseDeployment: vi.fn(),
		getStrategyDetails: vi.fn()
	}
}));

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

const defaultProps: DeploymentStepsProps = {
	handleDeployModal: vi.fn() as unknown as (args: DeployModalProps) => void,
	handleDisclaimerModal: vi.fn() as unknown as (args: DisclaimerModalProps) => void,
	settings: writable({} as ConfigSource),
	wagmiConfig: mockWagmiConfigStore,
	connected: mockConnectedStore,
	appKitModal: writable({} as AppKit),
	signerAddress: mockSignerAddressStore
};

describe('DeploymentSteps', () => {
	let mockGui: DotrainOrderGui;

	beforeEach(() => {
		vi.clearAllMocks();

		// Create a mock GUI instance
		mockGui = {
			areAllTokensSelected: vi.fn().mockReturnValue(false),
			getSelectTokens: vi.fn().mockReturnValue([]),
			getNetworkKey: vi.fn().mockReturnValue('flare'),
			getCurrentDeployment: vi.fn().mockReturnValue(mockDeployment),
			getAllFieldDefinitions: vi.fn().mockReturnValue([]),
			hasAnyDeposit: vi.fn().mockReturnValue(false),
			hasAnyVaultId: vi.fn().mockReturnValue(false),
			getAllTokenInfos: vi.fn().mockResolvedValue([]),
			getCurrentDeploymentDetails: vi.fn().mockReturnValue({
				name: 'Test Deployment',
				description: 'This is a test deployment description'
			})
		} as unknown as DotrainOrderGui;

		// Make useGui return our mock instance
		vi.mocked(useGui).mockReturnValue(mockGui);
	});

	it('shows select tokens section when tokens need to be selected', async () => {
		// Override the getSelectTokens mock for this test
		mockGui.getSelectTokens = vi.fn().mockReturnValue(['token1', 'token2']);

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

	it('shows connect wallet button when not connected', async () => {
		mockConnectedStore.mockSetSubscribeValue(false);

		render(DeploymentSteps, {
			props: defaultProps
		});

		await waitFor(() => {
			expect(screen.getByText('Connect Wallet')).toBeInTheDocument();
		});
	});

	it('refreshes field descriptions when tokens change', async () => {
		const mockSelectTokens = [
			{ key: 'token1', name: 'Token 1', description: undefined },
			{ key: 'token2', name: 'Token 2', description: undefined }
		];

		// Set up specific mocks for this test
		mockGui.getSelectTokens = vi.fn().mockReturnValue(mockSelectTokens);
		mockGui.getTokenInfo = vi.fn();
		mockGui.areAllTokensSelected = vi.fn().mockReturnValue(true);
		mockGui.isSelectTokenSet = vi.fn().mockReturnValue(false);
		mockGui.saveSelectToken = vi.fn();
		mockGui.getCurrentDeployment = vi.fn().mockReturnValue({
			deployment: {
				order: {
					inputs: [],
					outputs: []
				}
			},
			deposits: []
		});

		mockGui.getAllTokenInfos = vi.fn().mockResolvedValue([
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
		]);

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
			address: '0x1',
			decimals: 18,
			name: 'Token 1',
			symbol: 'TKN1'
		});
		await user.type(selectTokenInput, '0x1');

		const selectTokenOutput = screen.getAllByRole('textbox')[1];
		(mockGui.getTokenInfo as Mock).mockResolvedValue({
			address: '0x2',
			decimals: 18,
			name: 'Token 2',
			symbol: 'TKN2'
		});
		await user.type(selectTokenOutput, '0x2');

		await waitFor(() => {
			expect(mockGui.getAllTokenInfos).toHaveBeenCalled();
			expect(mockGui.getAllFieldDefinitions).toHaveBeenCalled();
		});

		selectTokenInput = screen.getAllByRole('textbox')[0];
		(mockGui.getTokenInfo as Mock).mockResolvedValue({
			address: '0x3',
			decimals: 18,
			name: 'Token 3',
			symbol: 'TKN3'
		});
		await user.type(selectTokenInput, '0x3');

		(mockGui.getAllTokenInfos as Mock).mockResolvedValue([
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
		]);

		await waitFor(() => {
			expect(mockGui.getAllTokenInfos).toHaveBeenCalled();
			expect(mockGui.getAllFieldDefinitions).toHaveBeenCalled();
		});
	});

	it('displays the correct deployment details', async () => {
		render(DeploymentSteps, {
			props: defaultProps
		});

		await waitFor(() => {
			expect(screen.getByText('Test Deployment')).toBeInTheDocument();
			expect(screen.getByText('This is a test deployment description')).toBeInTheDocument();
			expect(mockGui.getCurrentDeploymentDetails).toHaveBeenCalled();
		});
	});
});
