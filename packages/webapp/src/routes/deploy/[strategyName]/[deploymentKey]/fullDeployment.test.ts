import { vi, describe } from 'vitest';
import Page from './+page.svelte';
import { render, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { useAccount, type DeploymentArgs } from '@rainlanguage/ui-components';
import { readable } from 'svelte/store';
import { DotrainOrderGui } from '@rainlanguage/orderbook';
import { handleDeploy } from '$lib/services/handleDeploy';
import { REGISTRY_URL } from '$lib/constants';

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

describe('Full Deployment Tests', () => {
	let fixedLimitStrategy: string;
	let auctionStrategy: string;
	let dynamicSpreadStrategy: string;

	const fetchRegistry = async () => {
		const response = await fetch(REGISTRY_URL);
		const registry = await response.text();
		const linksMap = Object.fromEntries(
			registry
				.split('\n')
				.map((line) => line.trim().split(' '))
				.filter((parts) => parts.length === 2)
		);
		return linksMap;
	};
	const fetchStrategy = async (url: string) => {
		try {
			const response = await fetch(url);
			return await response.text();
		} catch (error) {
			assert.fail(error as string);
		}
	};
	function findLockRegion(a: string, b: string): { prefixEnd: number; suffixEnd: number } {
		expect(a.length).toEqual(b.length);
		const length = a.length;
		// Find prefix end
		let prefixEnd = 0;
		while (prefixEnd < length && a[prefixEnd] === b[prefixEnd]) {
			prefixEnd++;
		}
		// Find suffix start
		let suffixEnd = length;
		while (suffixEnd > prefixEnd && a[suffixEnd - 1] === b[suffixEnd - 1]) {
			suffixEnd--;
		}
		return { prefixEnd, suffixEnd };
	}

	beforeAll(async () => {
		const registry = await fetchRegistry();
		fixedLimitStrategy = await fetchStrategy(registry['fixed-limit']);
		assert(fixedLimitStrategy, 'Fixed limit strategy not found');
		auctionStrategy = await fetchStrategy(registry['auction-dca']);
		assert(auctionStrategy, 'Auction strategy not found');
		dynamicSpreadStrategy = await fetchStrategy(registry['dynamic-spread']);
		assert(dynamicSpreadStrategy, 'Dynamic spread strategy not found');
	});

	beforeEach(async () => {
		vi.clearAllMocks();
		vi.mocked(useAccount).mockReturnValue({
			account: readable('0x999999cf1046e68e36E1aA2E0E07105eDDD1f08E'),
			matchesAccount: vi.fn()
		});
		mockConnectedStore.mockSetSubscribeValue(true);
	});

	afterEach(async () => {
		await new Promise((resolve) => setTimeout(resolve, 5000));
	});

	it(
		'Fixed limit strategy',
		async () => {
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
				const gui = (await DotrainOrderGui.chooseDeployment(fixedLimitStrategy, 'flare'))
					.value as DotrainOrderGui;
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
			await new Promise((resolve) => setTimeout(resolve, 5000));
			const args = await getDeploymentArgs().catch((error) => {
				// eslint-disable-next-line no-console
				console.log('Fixed limit strategy error', error);
				return null;
			});

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
		},
		{ timeout: 30000 }
	);

	it(
		'Auction strategy',
		async () => {
			mockPageStore.mockSetSubscribeValue({
				data: {
					stores: { settings: mockSettingsStore },
					dotrain: auctionStrategy,
					deployment: {
						key: 'flare'
					},
					strategyDetail: {
						name: 'Auction'
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

			const sellTokenInput = selectTokenInputs[0];
			const buyTokenInput = selectTokenInputs[1];

			// Select the sell token
			await userEvent.clear(sellTokenInput);
			await userEvent.type(sellTokenInput, '0x12e605bc104e93B45e1aD99F9e555f659051c2BB');
			await waitFor(() => {
				expect(screen.getByTestId('select-token-success-output')).toBeInTheDocument();
			});

			// Select the buy token
			await userEvent.clear(buyTokenInput);
			await userEvent.type(buyTokenInput, '0x1D80c49BbBCd1C0911346656B529DF9E5c2F783d');
			await waitFor(() => {
				expect(screen.getByTestId('select-token-success-input')).toBeInTheDocument();
			});

			const timePerAmountEpochInput = screen.getByTestId(
				'binding-time-per-amount-epoch-input'
			) as HTMLInputElement;
			await userEvent.clear(timePerAmountEpochInput);
			await userEvent.type(timePerAmountEpochInput, '60');

			const amountPerEpochInput = screen.getByTestId(
				'binding-amount-per-epoch-input'
			) as HTMLInputElement;
			await userEvent.clear(amountPerEpochInput);
			await userEvent.type(amountPerEpochInput, '10');

			const maxTradeAmountInput = screen.getByTestId(
				'binding-max-trade-amount-input'
			) as HTMLInputElement;
			await userEvent.clear(maxTradeAmountInput);
			await userEvent.type(maxTradeAmountInput, '100');

			const minTradeAmountInput = screen.getByTestId(
				'binding-min-trade-amount-input'
			) as HTMLInputElement;
			await userEvent.clear(minTradeAmountInput);
			await userEvent.type(minTradeAmountInput, '1');

			const baselineInput = screen.getByTestId('binding-baseline-input') as HTMLInputElement;
			await userEvent.clear(baselineInput);
			await userEvent.type(baselineInput, '10');

			const initialIoInput = screen.getByTestId('binding-initial-io-input') as HTMLInputElement;
			await userEvent.clear(initialIoInput);
			await userEvent.type(initialIoInput, '10');

			const showAdvancedOptionsButton = screen.getByText('Show advanced options');
			await userEvent.click(showAdvancedOptionsButton);

			const vaultIdInputs = screen.getAllByTestId('vault-id-input') as HTMLInputElement[];

			// Set vault id for output
			await userEvent.clear(vaultIdInputs[0]);
			await userEvent.type(vaultIdInputs[0], '0x123');

			// Set vault id for input
			await userEvent.clear(vaultIdInputs[1]);
			await userEvent.type(vaultIdInputs[1], '0x234');

			// Click the "Deploy Strategy" button
			const deployButton = screen.getByText('Deploy Strategy');
			await userEvent.click(deployButton);

			const getDeploymentArgs = async () => {
				const gui = (await DotrainOrderGui.chooseDeployment(auctionStrategy, 'flare'))
					.value as DotrainOrderGui;
				await gui.saveSelectToken('input', '0x1D80c49BbBCd1C0911346656B529DF9E5c2F783d');
				await gui.saveSelectToken('output', '0x12e605bc104e93B45e1aD99F9e555f659051c2BB');
				gui.setVaultId(false, 0, '0x123');
				gui.setVaultId(true, 0, '0x234');
				gui.saveFieldValue('time-per-amount-epoch', '60');
				gui.saveFieldValue('amount-per-epoch', '10');
				gui.saveFieldValue('max-trade-amount', '100');
				gui.saveFieldValue('min-trade-amount', '1');
				gui.saveFieldValue('baseline', '10');
				gui.saveFieldValue('initial-io', '10');
				const args = await gui.getDeploymentTransactionArgs(
					'0x999999cf1046e68e36E1aA2E0E07105eDDD1f08E'
				);
				return args.value;
			};
			await new Promise((resolve) => setTimeout(resolve, 5000));
			const args = await getDeploymentArgs().catch((error) => {
				// eslint-disable-next-line no-console
				console.log('Auction strategy error', error);
				return null;
			});

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
		},
		{ timeout: 30000 }
	);

	it(
		'Dynamic spread strategy',
		async () => {
			mockPageStore.mockSetSubscribeValue({
				data: {
					stores: { settings: mockSettingsStore },
					dotrain: dynamicSpreadStrategy,
					deployment: {
						key: 'flare'
					},
					strategyDetail: {
						name: 'Dynamic spread'
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

			const firstTokenInput = selectTokenInputs[0];
			const secondTokenInput = selectTokenInputs[1];

			// Select the first token
			await userEvent.clear(firstTokenInput);
			await userEvent.type(firstTokenInput, '0x1D80c49BbBCd1C0911346656B529DF9E5c2F783d');
			await waitFor(() => {
				expect(screen.getByTestId('select-token-success-token1')).toBeInTheDocument();
			});

			// Select the second token
			await userEvent.clear(secondTokenInput);
			await userEvent.type(secondTokenInput, '0x12e605bc104e93B45e1aD99F9e555f659051c2BB');
			await waitFor(() => {
				expect(screen.getByTestId('select-token-success-token2')).toBeInTheDocument();
			});

			const amountIsFastExitButton = screen.getByTestId(
				'binding-amount-is-fast-exit-preset-Yes'
			) as HTMLElement;
			await userEvent.click(amountIsFastExitButton);

			const notAmountIsFastExitButton = screen.getByTestId(
				'binding-not-amount-is-fast-exit-preset-No'
			) as HTMLElement;
			await userEvent.click(notAmountIsFastExitButton);

			const initialIoInput = screen.getByTestId('binding-initial-io-input') as HTMLInputElement;
			await userEvent.clear(initialIoInput);
			await userEvent.type(initialIoInput, '100');

			const maxAmountInput = screen.getByTestId('binding-max-amount-input') as HTMLInputElement;
			await userEvent.clear(maxAmountInput);
			await userEvent.type(maxAmountInput, '1000');

			const minAmountInput = screen.getByTestId('binding-min-amount-input') as HTMLInputElement;
			await userEvent.clear(minAmountInput);
			await userEvent.type(minAmountInput, '10');

			const showAdvancedOptionsButton = screen.getByText('Show advanced options');
			await userEvent.click(showAdvancedOptionsButton);

			const vaultIdInputs = screen.getAllByTestId('vault-id-input') as HTMLInputElement[];

			// Set vault id for token1
			await userEvent.clear(vaultIdInputs[0]);
			await userEvent.type(vaultIdInputs[0], '0x234');

			// Set vault id for token2
			await userEvent.clear(vaultIdInputs[1]);
			await userEvent.type(vaultIdInputs[1], '0x123');

			// Click the "Deploy Strategy" button
			const deployButton = screen.getByText('Deploy Strategy');
			await userEvent.click(deployButton);

			const getDeploymentArgs = async () => {
				const gui = (await DotrainOrderGui.chooseDeployment(dynamicSpreadStrategy, 'flare'))
					.value as DotrainOrderGui;
				await gui.saveSelectToken('token1', '0x1D80c49BbBCd1C0911346656B529DF9E5c2F783d');
				await gui.saveSelectToken('token2', '0x12e605bc104e93B45e1aD99F9e555f659051c2BB');
				gui.setVaultId(false, 0, '0x123');
				gui.setVaultId(true, 0, '0x234');
				gui.saveFieldValue('amount-is-fast-exit', '1');
				gui.saveFieldValue('not-amount-is-fast-exit', '0');
				gui.saveFieldValue('initial-io', '100');
				gui.saveFieldValue('max-amount', '1000');
				gui.saveFieldValue('min-amount', '10');
				const args = await gui.getDeploymentTransactionArgs(
					'0x999999cf1046e68e36E1aA2E0E07105eDDD1f08E'
				);
				return args.value;
			};
			await new Promise((resolve) => setTimeout(resolve, 5000));
			const args = await getDeploymentArgs().catch((error) => {
				// eslint-disable-next-line no-console
				console.log('Dynamic spread strategy error', error);
				return null;
			});

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
		},
		{ timeout: 30000 }
	);
});
