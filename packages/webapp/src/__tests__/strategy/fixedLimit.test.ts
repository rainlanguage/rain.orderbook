import { vi } from 'vitest';
import Page from '../../routes/deploy/[strategyName]/[deploymentKey]/+page.svelte';
import { render, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { useAccount, type DeploymentArgs } from '@rainlanguage/ui-components';
import { readable } from 'svelte/store';
import { DotrainOrderGui } from '@rainlanguage/orderbook';
import { handleDeploy } from '$lib/services/handleDeploy';
import { fetchStrategy, fetchRegistry, findLockRegion } from './helpers';

const { mockPageStore, mockSettingsStore, mockTransactionStore } = await vi.hoisted(
	() => import('@rainlanguage/ui-components')
);

const { mockConnectedStore, mockAppKitModalStore, mockWagmiConfigStore } = await vi.hoisted(
	() => import('$lib/__mocks__/stores')
);

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		useAccount: vi.fn(),
		transactionStore: mockTransactionStore
	};
});

vi.mock('$app/stores', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		page: mockPageStore
	};
});

vi.mock('$lib/stores/wagmi', () => ({
	connected: mockConnectedStore,
	appKitModal: mockAppKitModalStore,
	wagmiConfig: mockWagmiConfigStore
}));

vi.mock('$lib/services/handleDeploy', () => ({
	handleDeploy: vi.fn()
}));

let fixedLimitStrategy: string;

beforeAll(async () => {
	const registry = await fetchRegistry();
	fixedLimitStrategy = await fetchStrategy(registry['fixed-limit']);
});

beforeEach(() => {
	vi.clearAllMocks();
	vi.mocked(mockPageStore).mockSetSubscribeValue({});
	vi.mocked(useAccount).mockReturnValue({
		account: readable('0x999999cf1046e68e36E1aA2E0E07105eDDD1f08E'),
		matchesAccount: vi.fn()
	});
});

it('Fixed limit strategy full deployment', async () => {
	mockConnectedStore.mockSetSubscribeValue(true);
	mockPageStore.mockSetSubscribeValue({
		data: {
			stores: { settings: mockSettingsStore },
			dotrain: fixedLimitStrategy,
			deployment: {
				key: 'flare'
			},
			strategyDetail: {
				name: 'Fixed limit'
			}
		}
	});
	const screen = render(Page);

	// Wait for the gui provider to be in the document
	await waitFor(() => {
		expect(screen.getByTestId('gui-provider')).toBeInTheDocument();
	});

	// Get all the current input elements for select tokens
	const selectTokenInputs = screen.getAllByRole('textbox') as HTMLInputElement[];

	const buyTokenInput = selectTokenInputs[0];
	const sellTokenInput = selectTokenInputs[1];

	// Select the buy token
	await userEvent.clear(buyTokenInput);
	await userEvent.type(buyTokenInput, '0x1D80c49BbBCd1C0911346656B529DF9E5c2F783d');
	await waitFor(() => {
		expect(screen.getByTestId('select-token-success-token1')).toBeInTheDocument();
	});

	// Select the sell token
	await userEvent.clear(sellTokenInput);
	await userEvent.type(sellTokenInput, '0x12e605bc104e93B45e1aD99F9e555f659051c2BB');
	await waitFor(() => {
		expect(screen.getByTestId('select-token-success-token2')).toBeInTheDocument();
	});

	// Get the input component and write "10" into it
	const customValueInput = screen.getAllByPlaceholderText('Enter custom value')[0];
	await userEvent.clear(customValueInput);
	await userEvent.type(customValueInput, '10');

	const showAdvancedOptionsButton = screen.getByText('Show advanced options');
	await userEvent.click(showAdvancedOptionsButton);

	const vaultIdInputs = screen.getAllByTestId('vault-id-input') as HTMLInputElement[];

	// Set vault id for token2
	await userEvent.clear(vaultIdInputs[0]);
	await userEvent.type(vaultIdInputs[0], '0x123');

	// Set vault id for token1
	await userEvent.clear(vaultIdInputs[1]);
	await userEvent.type(vaultIdInputs[1], '0x234');

	// Click the "Deploy Strategy" button
	const deployButton = screen.getByText('Deploy Strategy');
	await userEvent.click(deployButton);

	const getDeploymentArgs = async () => {
		const gui = new DotrainOrderGui();
		await gui.chooseDeployment(fixedLimitStrategy, 'flare');
		await gui.saveSelectToken('token1', '0x1D80c49BbBCd1C0911346656B529DF9E5c2F783d');
		await gui.saveSelectToken('token2', '0x12e605bc104e93B45e1aD99F9e555f659051c2BB');
		gui.setVaultId(false, 0, '0x123');
		gui.setVaultId(true, 0, '0x234');
		gui.saveFieldValue('fixed-io', '10');
		const args = await gui.getDeploymentTransactionArgs(
			'0x999999cf1046e68e36E1aA2E0E07105eDDD1f08E'
		);
		return args.value;
	};
	const args = await getDeploymentArgs();

	// @ts-expect-error mock is not typed
	const callArgs = handleDeploy.mock.calls[0][0] as DeploymentArgs;

	const { prefixEnd, suffixEnd } = findLockRegion(
		callArgs.deploymentCalldata,
		args?.deploymentCalldata || ''
	);

	expect(callArgs.approvals).toEqual(args?.approvals);
	expect(callArgs.deploymentCalldata.length).toEqual(args?.deploymentCalldata.length);
	expect(callArgs.deploymentCalldata.slice(0, prefixEnd)).toEqual(
		args?.deploymentCalldata.slice(0, prefixEnd)
	);
	// The middle section of the calldata is different because of secret and nonce
	expect(callArgs.deploymentCalldata.slice(prefixEnd, suffixEnd)).not.toEqual(
		args?.deploymentCalldata.slice(prefixEnd, suffixEnd)
	);
	expect(callArgs.deploymentCalldata.slice(suffixEnd)).toEqual(
		args?.deploymentCalldata.slice(suffixEnd)
	);
	expect(callArgs.orderbookAddress).toEqual(args?.orderbookAddress);
	expect(callArgs.chainId).toEqual(args?.chainId);
	expect(callArgs.subgraphUrl).toEqual(undefined);
	expect(callArgs.network).toEqual('flare');
});
