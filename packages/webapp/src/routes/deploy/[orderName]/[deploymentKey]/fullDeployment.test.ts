import { vi, describe } from 'vitest';
import Page from './+page.svelte';
import { render, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import {
	useAccount,
	useToasts,
	useTransactions,
	type TransactionConfirmationProps
} from '@rainlanguage/ui-components';
import { readable, writable } from 'svelte/store';
import { DotrainRegistry, type NameAndDescriptionCfg } from '@rainlanguage/orderbook';
import { REGISTRY_URL } from '$lib/constants';
import { handleTransactionConfirmationModal } from '$lib/services/modal';

const { mockPageStore } = await vi.hoisted(() => import('@rainlanguage/ui-components'));

const { mockConnectedStore, mockAppKitModalStore, mockWagmiConfigStore } = await vi.hoisted(
	() => import('$lib/__mocks__/stores')
);

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		useTransactions: vi.fn(),
		useAccount: vi.fn(),
		useToasts: vi.fn()
	};
});

vi.mock('$lib/services/modal', async () => ({
	handleDisclaimerModal: vi.fn(async (props) => {
		const { DisclaimerModal } = await import('@rainlanguage/ui-components');
		new DisclaimerModal({
			target: document.body,
			props: { ...props, open: true }
		});
	}),
	handleTransactionConfirmationModal: vi.fn()
}));

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

describe('Full Deployment Tests', () => {
	let registry: DotrainRegistry | null = null;

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
		const registryResult = await DotrainRegistry.new(REGISTRY_URL);
		if (registryResult.error) {
			throw new Error('Failed to create registry');
		}
		registry = registryResult.value;
	});

	beforeEach(async () => {
		vi.clearAllMocks();
		vi.mocked(useAccount).mockReturnValue({
			account: readable('0x999999cf1046e68e36E1aA2E0E07105eDDD1f08E'),
			matchesAccount: vi.fn()
		});
		vi.mocked(useToasts).mockReturnValue({
			removeToast: vi.fn(),
			toasts: writable([]),
			addToast: vi.fn(),
			errToast: vi.fn()
		});
		vi.mocked(useTransactions).mockReturnValue({
			// @ts-expect-error simple object
			manager: writable({}),
			transactions: readable()
		});
		mockConnectedStore.mockSetSubscribeValue(true);
		vi.mocked(handleTransactionConfirmationModal).mockResolvedValue({
			success: true,
			hash: '0xtesthash'
		});
	});

	afterEach(async () => {
		await new Promise((resolve) => setTimeout(resolve, 10000));
	});

	it(
		'Fixed limit order',
		async () => {
			if (!registry) {
				throw new Error('No registry available');
			}

			const fixedLimitDeploymentDetails = registry.getDeploymentDetails('fixed-limit');
			if (fixedLimitDeploymentDetails.error) {
				throw new Error('Failed to get deployment details');
			}
			const deployment = fixedLimitDeploymentDetails.value.get('base') as NameAndDescriptionCfg;
			const fixedLimitOrderDetail = registry
				.getAllOrderDetails()
				.value?.valid.get('fixed-limit') as NameAndDescriptionCfg;

			mockPageStore.mockSetSubscribeValue({
				data: {
					orderName: 'fixed-limit',
					deployment: { key: 'base', ...deployment },
					registry,
					orderDetail: fixedLimitOrderDetail
				}
			});
			const screen = render(Page);

			// Wait for the gui provider to be in the document
			await waitFor(
				() => {
					expect(screen.getByTestId('gui-provider')).toBeInTheDocument();
				},
				{ timeout: 30000 }
			);

			await waitFor(
				() => {
					expect(screen.getAllByRole('button', { name: /chevron down solid/i }).length).toBe(2);
				},
				{ timeout: 30000 }
			);
			const tokenSelectionButtons = screen.getAllByRole('button', { name: /chevron down solid/i });

			await userEvent.click(tokenSelectionButtons[0]);
			await userEvent.click(screen.getByText('Cortex'));
			await waitFor(
				() => {
					expect(screen.getByTestId('select-token-success-token1')).toBeInTheDocument();
				},
				{ timeout: 30000 }
			);
			await new Promise((resolve) => setTimeout(resolve, 2000));

			await userEvent.click(tokenSelectionButtons[1]);
			await userEvent.click(screen.getByText('NANI'));
			await waitFor(
				() => {
					expect(screen.getByTestId('select-token-success-token2')).toBeInTheDocument();
				},
				{ timeout: 30000 }
			);
			await new Promise((resolve) => setTimeout(resolve, 2000));

			// Get the input component and write "10" into it
			const customValueInput = screen.getAllByPlaceholderText('Enter custom value')[0];
			await userEvent.clear(customValueInput);
			await userEvent.type(customValueInput, '10');

			const showAdvancedOptionsButton = screen.getByText('Show advanced options');
			await userEvent.click(showAdvancedOptionsButton);

			const vaultIdInputs = screen.getAllByTestId('vault-id-input') as HTMLInputElement[];

			// Set vault id for token2
			await userEvent.clear(vaultIdInputs[0]);
			await userEvent.type(vaultIdInputs[0], '234');

			// Set vault id for token1
			await userEvent.clear(vaultIdInputs[1]);
			await userEvent.type(vaultIdInputs[1], '123');

			// Click the "Deploy Order" button
			const deployButton = screen.getByText('Deploy Order');
			await userEvent.click(deployButton);

			await waitFor(
				async () => {
					const disclaimerButton = screen.getByText('Deploy');
					await userEvent.click(disclaimerButton);
				},
				{ timeout: 30000 }
			);

			const getDeploymentArgs = async () => {
				if (!registry) {
					throw new Error('Registry not initialized');
				}
				const guiResult = await registry.getGui('fixed-limit', 'base');
				if (guiResult.error) {
					throw new Error(guiResult.error.readableMsg ?? guiResult.error.msg);
				}
				const gui = guiResult.value;
				await gui.setSelectToken('token1', '0x000000000000012def132e61759048be5b5c6033');
				await gui.setSelectToken('token2', '0x00000000000007c8612ba63df8ddefd9e6077c97');
				gui.setVaultId('output', 'token2', '234');
				gui.setVaultId('input', 'token1', '123');
				gui.setFieldValue('fixed-io', '10');
				const args = await gui.getDeploymentTransactionArgs(
					'0x999999cf1046e68e36E1aA2E0E07105eDDD1f08E'
				);
				return args.value;
			};
			await new Promise((resolve) => setTimeout(resolve, 10000));
			const args = await getDeploymentArgs().catch((error) => {
				// eslint-disable-next-line no-console
				console.log('Fixed limit order error', error);
				return null;
			});

			// @ts-expect-error mock is not typed
			const callArgs = handleTransactionConfirmationModal.mock.calls.at(-1)?.[0] as
				| TransactionConfirmationProps
				| undefined;

			expect(callArgs).toBeDefined();
			if (!callArgs) {
				return;
			}
			expect(callArgs.modalTitle).toEqual('Deploying your order');

			const { prefixEnd, suffixEnd } = findLockRegion(
				callArgs.args.calldata,
				args?.deploymentCalldata || ''
			);

			expect(callArgs.args.calldata.length).toEqual(args?.deploymentCalldata.length);
			expect(callArgs.args.calldata.slice(0, prefixEnd)).toEqual(
				args?.deploymentCalldata.slice(0, prefixEnd)
			);
			// The middle section of the calldata is different because of secret and nonce
			expect(callArgs.args.calldata.slice(prefixEnd, suffixEnd)).not.toEqual(
				args?.deploymentCalldata.slice(prefixEnd, suffixEnd)
			);
			expect(callArgs.args.calldata.slice(suffixEnd)).toEqual(
				args?.deploymentCalldata.slice(suffixEnd)
			);
			expect(callArgs.args.toAddress).toEqual(args?.orderbookAddress);
			expect(callArgs.args.chainId).toEqual(args?.chainId);
		},
		{ timeout: 30000 }
	);

	// TODO: Issue #2037
	// it(
	// 	'Auction order',
	// 	async () => {
	// 		mockPageStore.mockSetSubscribeValue({
	// 			data: {
	// 				dotrain: auctionOrder,
	// 				deployment: {
	// 					key: 'base'
	// 				},
	// 				orderDetail: {
	// 					name: 'Auction'
	// 				}
	// 			}
	// 		});

	// 		const screen = render(Page);

	// 		// Wait for the gui provider to be in the document
	// 		await waitFor(
	// 			() => {
	// 				expect(screen.getByTestId('gui-provider')).toBeInTheDocument();
	// 			},
	// 			{ timeout: 300000 }
	// 		);

	// 		// Check that the token dropdowns are present
	// 		await waitFor(
	// 			() => {
	// 				expect(screen.getAllByRole('button', { name: /chevron down solid/i }).length).toBe(2);
	// 			},
	// 			{ timeout: 300000 }
	// 		);
	// 		const tokenSelectionButtons = screen.getAllByRole('button', { name: /chevron down solid/i });

	// 		await userEvent.click(tokenSelectionButtons[0]);
	// 		await userEvent.click(screen.getByText('Cortex'));
	// 		await waitFor(
	// 			() => {
	// 				expect(screen.getByTestId('select-token-success-output')).toBeInTheDocument();
	// 			},
	// 			{ timeout: 300000 }
	// 		);
	// 		await new Promise((resolve) => setTimeout(resolve, 2000));

	// 		await userEvent.click(tokenSelectionButtons[1]);
	// 		await userEvent.click(screen.getByText('NANI'));
	// 		await waitFor(
	// 			() => {
	// 				expect(screen.getByTestId('select-token-success-input')).toBeInTheDocument();
	// 			},
	// 			{ timeout: 300000 }
	// 		);
	// 		await new Promise((resolve) => setTimeout(resolve, 2000));

	// 		const timePerAmountEpochInput = screen.getByTestId(
	// 			'binding-time-per-amount-epoch-input'
	// 		) as HTMLInputElement;
	// 		await userEvent.clear(timePerAmountEpochInput);
	// 		await userEvent.type(timePerAmountEpochInput, '60');

	// 		const amountPerEpochInput = screen.getByTestId(
	// 			'binding-amount-per-epoch-input'
	// 		) as HTMLInputElement;
	// 		await userEvent.clear(amountPerEpochInput);
	// 		await userEvent.type(amountPerEpochInput, '10');

	// 		const maxTradeAmountInput = screen.getByTestId(
	// 			'binding-max-trade-amount-input'
	// 		) as HTMLInputElement;
	// 		await userEvent.clear(maxTradeAmountInput);
	// 		await userEvent.type(maxTradeAmountInput, '100');

	// 		const minTradeAmountInput = screen.getByTestId(
	// 			'binding-min-trade-amount-input'
	// 		) as HTMLInputElement;
	// 		await userEvent.clear(minTradeAmountInput);
	// 		await userEvent.type(minTradeAmountInput, '1');

	// 		const baselineInput = screen.getByTestId('binding-baseline-input') as HTMLInputElement;
	// 		await userEvent.clear(baselineInput);
	// 		await userEvent.type(baselineInput, '10');

	// 		const initialIoInput = screen.getByTestId('binding-initial-io-input') as HTMLInputElement;
	// 		await userEvent.clear(initialIoInput);
	// 		await userEvent.type(initialIoInput, '10');

	// 		const showAdvancedOptionsButton = screen.getByText('Show advanced options');
	// 		await userEvent.click(showAdvancedOptionsButton);

	// 		const vaultIdInputs = screen.getAllByTestId('vault-id-input') as HTMLInputElement[];

	// 		// Set vault id for output
	// 		await userEvent.clear(vaultIdInputs[0]);
	// 		await userEvent.type(vaultIdInputs[0], '0x123');

	// 		// Set vault id for input
	// 		await userEvent.clear(vaultIdInputs[1]);
	// 		await userEvent.type(vaultIdInputs[1], '0x234');

	// 		// Click the "Deploy Order" button
	// 		const deployButton = screen.getByText('Deploy Order');
	// 		await userEvent.click(deployButton);

	// 		await waitFor(
	// 			async () => {
	// 				const disclaimerButton = screen.getByText('Deploy');
	// 				await userEvent.click(disclaimerButton);
	// 			},
	// 			{ timeout: 300000 }
	// 		);

	// 		const getDeploymentArgs = async () => {
	// 			const gui = (await DotrainOrderGui.newWithDeployment(auctionOrder, 'base'))
	// 				.value as DotrainOrderGui;
	// 			await gui.setSelectToken('input', '0x000000000000012def132e61759048be5b5c6033');
	// 			await gui.setSelectToken('output', '0x00000000000007c8612ba63df8ddefd9e6077c97');
	// 			gui.setVaultId('output', 'output', '0x123');
	// 			gui.setVaultId('input', 'input', '0x234');
	// 			gui.setFieldValue('time-per-amount-epoch', '60');
	// 			gui.setFieldValue('amount-per-epoch', '10');
	// 			gui.setFieldValue('max-trade-amount', '100');
	// 			gui.setFieldValue('min-trade-amount', '1');
	// 			gui.setFieldValue('baseline', '10');
	// 			gui.setFieldValue('initial-io', '10');
	// 			const args = await gui.getDeploymentTransactionArgs(
	// 				'0x999999cf1046e68e36E1aA2E0E07105eDDD1f08E'
	// 			);
	// 			return args.value;
	// 		};
	// 		await new Promise((resolve) => setTimeout(resolve, 10000));
	// 		const args = await getDeploymentArgs().catch((error) => {
	// 			// eslint-disable-next-line no-console
	// 			console.log('Auction order error', error);
	// 			return null;
	// 		});

	// 		// @ts-expect-error mock is not typed
	// 		const callArgs = handleTransactionConfirmationModal.mock
	// 			.calls[0][0] as TransactionConfirmationProps;

	// 		const { prefixEnd, suffixEnd } = findLockRegion(
	// 			callArgs.args.calldata,
	// 			args?.deploymentCalldata || ''
	// 		);

	// 		expect(callArgs.args.calldata.length).toEqual(args?.deploymentCalldata.length);
	// 		expect(callArgs.args.calldata.slice(0, prefixEnd)).toEqual(
	// 			args?.deploymentCalldata.slice(0, prefixEnd)
	// 		);
	// 		// The middle section of the calldata is different because of secret and nonce
	// 		expect(callArgs.args.calldata.slice(prefixEnd, suffixEnd)).not.toEqual(
	// 			args?.deploymentCalldata.slice(prefixEnd, suffixEnd)
	// 		);
	// 		expect(callArgs.args.calldata.slice(suffixEnd)).toEqual(
	// 			args?.deploymentCalldata.slice(suffixEnd)
	// 		);
	// 		expect(callArgs.args.toAddress).toEqual(args?.orderbookAddress);
	// 		expect(callArgs.args.chainId).toEqual(args?.chainId);
	// 	},
	// 	{ timeout: 300000 }
	// );

	// it(
	// 	'Dynamic spread order',
	// 	async () => {
	// 		mockPageStore.mockSetSubscribeValue({
	// 			data: {
	// 				dotrain: dynamicSpreadOrder,
	// 				deployment: {
	// 					key: 'base'
	// 				},
	// 				orderDetail: {
	// 					name: 'Dynamic spread'
	// 				}
	// 			}
	// 		});

	// 		const screen = render(Page);

	// 		// Wait for the gui provider to be in the document
	// 		await waitFor(
	// 			() => {
	// 				expect(screen.getByTestId('gui-provider')).toBeInTheDocument();
	// 			},
	// 			{ timeout: 300000 }
	// 		);

	// 		await waitFor(
	// 			() => {
	// 				expect(screen.getAllByRole('button', { name: /chevron down solid/i }).length).toBe(2);
	// 			},
	// 			{ timeout: 300000 }
	// 		);
	// 		const tokenSelectionButtons = screen.getAllByRole('button', { name: /chevron down solid/i });

	// 		await userEvent.click(tokenSelectionButtons[0]);
	// 		await userEvent.click(screen.getByText('Cortex'));
	// 		await waitFor(() => {
	// 			expect(screen.getByTestId('select-token-success-token1')).toBeInTheDocument();
	// 		});
	// 		await new Promise((resolve) => setTimeout(resolve, 2000));

	// 		await userEvent.click(tokenSelectionButtons[1]);
	// 		await userEvent.click(screen.getByText('NANI'));
	// 		await waitFor(
	// 			() => {
	// 				expect(screen.getByTestId('select-token-success-token2')).toBeInTheDocument();
	// 			},
	// 			{ timeout: 300000 }
	// 		);
	// 		await new Promise((resolve) => setTimeout(resolve, 2000));

	// 		const amountIsFastExitButton = screen.getByTestId(
	// 			'binding-amount-is-fast-exit-preset-Yes'
	// 		) as HTMLElement;
	// 		await userEvent.click(amountIsFastExitButton);

	// 		const notAmountIsFastExitButton = screen.getByTestId(
	// 			'binding-not-amount-is-fast-exit-preset-No'
	// 		) as HTMLElement;
	// 		await userEvent.click(notAmountIsFastExitButton);

	// 		const initialIoInput = screen.getByTestId('binding-initial-io-input') as HTMLInputElement;
	// 		await userEvent.clear(initialIoInput);
	// 		await userEvent.type(initialIoInput, '100');

	// 		const maxAmountInput = screen.getByTestId('binding-max-amount-input') as HTMLInputElement;
	// 		await userEvent.clear(maxAmountInput);
	// 		await userEvent.type(maxAmountInput, '1000');

	// 		const minAmountInput = screen.getByTestId('binding-min-amount-input') as HTMLInputElement;
	// 		await userEvent.clear(minAmountInput);
	// 		await userEvent.type(minAmountInput, '10');

	// 		const showAdvancedOptionsButton = screen.getByText('Show advanced options');
	// 		await userEvent.click(showAdvancedOptionsButton);

	// 		const vaultIdInputs = screen.getAllByTestId('vault-id-input') as HTMLInputElement[];

	// 		// Set vault id for token1
	// 		await userEvent.clear(vaultIdInputs[0]);
	// 		await userEvent.type(vaultIdInputs[0], '0x234');

	// 		// Set vault id for token2
	// 		await userEvent.clear(vaultIdInputs[1]);
	// 		await userEvent.type(vaultIdInputs[1], '0x123');

	// 		// Click the "Deploy Order" button
	// 		const deployButton = screen.getByText('Deploy Order');
	// 		await userEvent.click(deployButton);

	// 		await waitFor(
	// 			async () => {
	// 				const disclaimerButton = screen.getByText('Deploy');
	// 				await userEvent.click(disclaimerButton);
	// 			},
	// 			{ timeout: 300000 }
	// 		);

	// 		const getDeploymentArgs = async () => {
	// 			const gui = (await DotrainOrderGui.newWithDeployment(dynamicSpreadOrder, 'base'))
	// 				.value as DotrainOrderGui;
	// 			await gui.setSelectToken('token1', '0x000000000000012def132e61759048be5b5c6033');
	// 			await gui.setSelectToken('token2', '0x00000000000007c8612ba63df8ddefd9e6077c97');
	// 			gui.setVaultId('output', 'token2', '0x123');
	// 			gui.setVaultId('input', 'token1', '0x234');
	// 			gui.setFieldValue('amount-is-fast-exit', '1');
	// 			gui.setFieldValue('not-amount-is-fast-exit', '0');
	// 			gui.setFieldValue('initial-io', '100');
	// 			gui.setFieldValue('max-amount', '1000');
	// 			gui.setFieldValue('min-amount', '10');
	// 			const args = await gui.getDeploymentTransactionArgs(
	// 				'0x999999cf1046e68e36E1aA2E0E07105eDDD1f08E'
	// 			);
	// 			return args.value;
	// 		};
	// 		await new Promise((resolve) => setTimeout(resolve, 10000));
	// 		const args = await getDeploymentArgs().catch((error) => {
	// 			// eslint-disable-next-line no-console
	// 			console.log('Dynamic spread order error', error);
	// 			return null;
	// 		});

	// 		// @ts-expect-error mock is not typed
	// 		const callArgs = handleTransactionConfirmationModal.mock
	// 			.calls[0][0] as TransactionConfirmationProps;

	// 		const { prefixEnd, suffixEnd } = findLockRegion(
	// 			callArgs.args.calldata,
	// 			args?.deploymentCalldata || ''
	// 		);

	// 		expect(callArgs.args.calldata.length).toEqual(args?.deploymentCalldata.length);
	// 		expect(callArgs.args.calldata.slice(0, prefixEnd)).toEqual(
	// 			args?.deploymentCalldata.slice(0, prefixEnd)
	// 		);
	// 		// The middle section of the calldata is different because of secret and nonce
	// 		expect(callArgs.args.calldata.slice(prefixEnd, suffixEnd)).not.toEqual(
	// 			args?.deploymentCalldata.slice(prefixEnd, suffixEnd)
	// 		);
	// 		expect(callArgs.args.calldata.slice(suffixEnd)).toEqual(
	// 			args?.deploymentCalldata.slice(suffixEnd)
	// 		);
	// 		expect(callArgs.args.toAddress).toEqual(args?.orderbookAddress);
	// 		expect(callArgs.args.chainId).toEqual(args?.chainId);
	// 	},
	// 	{ timeout: 300000 }
	// );
});
