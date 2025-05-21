import { vi, describe } from 'vitest';
import Page from './+page.svelte';
import { render, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { useAccount } from '@rainlanguage/ui-components';
import { readable } from 'svelte/store';
import { DotrainOrderGui } from '@rainlanguage/orderbook';
import { handleDeploy } from '$lib/services/handleDeploy';

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
	beforeEach(() => {
		vi.clearAllMocks();
		vi.mocked(useAccount).mockReturnValue({
			account: readable('0x999999cf1046e68e36E1aA2E0E07105eDDD1f08E'),
			matchesAccount: vi.fn()
		});
	});

	it('Fixed limit strategy', async () => {
		// const handleDeploymentTransactionSpy = vi.spyOn(
		// 	mockTransactionStore,
		// 	'handleDeploymentTransaction'
		// );
		mockConnectedStore.mockSetSubscribeValue(true);
		// Mock the page store with the fixed limit strategy
		mockPageStore.mockSetSubscribeValue({
			data: {
				stores: { settings: mockSettingsStore },
				dotrain: fixedLimitStrategy,
				deployment: {
					key: 'flare',
					name: 'Fixed limit',
					description: 'Fixed limit strategy'
				},
				strategyDetail: {
					name: 'Fixed limit'
				}
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
			} as any
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

		await waitFor(async () => {
			// expect(handleDeploymentTransactionSpy).toHaveBeenCalled();

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

			expect(handleDeploy).toHaveBeenCalledWith({
				approvals: args?.approvals,
				// TODO: for some reason the calldata is different in ui and here
				deploymentCalldata: args?.deploymentCalldata,
				orderbookAddress: args?.orderbookAddress,
				chainId: args?.chainId,
				subgraphUrl: undefined,
				network: 'flare'
			});
		});

		screen.debug();
	});
});

const fixedLimitStrategy = `
raindex-version: 8898591f3bcaa21dc91dc3b8584330fc405eadfa

networks:
  flare:
    rpc: https://flare.rpc.thirdweb.com
    chain-id: 14
    network-id: 14
    currency: FLR
  base:
    rpc: https://base-rpc.publicnode.com
    chain-id: 8453
    network-id: 8453
    currency: ETH
  arbitrum:
    rpc: https://1rpc.io/arb
    chain-id: 42161
    network-id: 42161
    currency: ETH
  polygon:
    rpc: https://1rpc.io/matic
    chain-id: 137
    network-id: 137
    currency: POL
  bsc:
    rpc: https://bsc-dataseed.bnbchain.org
    chain-id: 56
    network-id: 56
    currency: BNB
  ethereum:
    rpc: https://1rpc.io/eth
    chain-id: 1
    network-id: 1
    currency: ETH
  linea:
    rpc: https://rpc.linea.build
    chain-id: 59144
    network-id: 59144
    currency: ETH

metaboards:
  flare: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-flare-0x893BBFB7/0.1/gn
  base: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-base-0x59401C93/0.1/gn
  arbitrum: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-arbitrum/0.1/gn
  polygon: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-polygon/0.1/gn
  bsc: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-bsc/0.1/gn
  ethereum: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/metadata-mainnet/2024-10-25-2857/gn
  linea: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-linea-0xed7d6156/1.0.0/gn

subgraphs:
  flare: https://example.com/subgraph
  base: https://example.com/subgraph
  arbitrum: https://example.com/subgraph
  polygon: https://example.com/subgraph
  bsc: https://example.com/subgraph
  ethereum: https://example.com/subgraph
  linea: https://example.com/subgraph

orderbooks:
  flare:
    address: 0xCEe8Cd002F151A536394E564b84076c41bBBcD4d
  base:
    address: 0xd2938e7c9fe3597f78832ce780feb61945c377d7
  arbitrum:
    address: 0x550878091b2B1506069F61ae59e3A5484Bca9166
  polygon:
    address: 0x7D2f700b1f6FD75734824EA4578960747bdF269A
  bsc:
    address: 0xd2938E7c9fe3597F78832CE780Feb61945c377d7
  ethereum:
    address: 0x0eA6d458488d1cf51695e1D6e4744e6FB715d37C
  linea:
    address: 0x22410e2a46261a1B1e3899a072f303022801C764

deployers:
  flare:
    address: 0xE3989Ea7486c0F418C764e6c511e86f6E8830FAb
  base:
    address: 0xC1A14cE2fd58A3A2f99deCb8eDd866204eE07f8D
  arbitrum:
    address: 0x9B0D254bd858208074De3d2DaF5af11b3D2F377F
  polygon:
    address: 0xE7116BC05C8afe25e5B54b813A74F916B5D42aB1
  ethereum:
    address: 0xd19581a021f4704ad4eBfF68258e7A0a9DB1CD77
  linea:
    address: 0xA2f56F8F74B7d04d61f281BE6576b6155581dcBA
  bsc:
    address: 0xA2f56F8F74B7d04d61f281BE6576b6155581dcBA

orders:
  base:
    orderbook: base
    inputs:
      - token: token1
    outputs:
      - token: token2
  flare:
    orderbook: flare
    inputs:
      - token: token1
    outputs:
      - token: token2
  arbitrum:
    orderbook: arbitrum
    inputs:
      - token: token1
    outputs:
      - token: token2
  polygon:
    orderbook: polygon
    inputs:
      - token: token1
    outputs:
      - token: token2
  bsc:
    orderbook: bsc
    inputs:
      - token: token1
    outputs:
      - token: token2
  ethereum:
    orderbook: ethereum
    inputs:
      - token: token1
    outputs:
      - token: token2
  linea:
    orderbook: linea
    inputs:
      - token: token1
    outputs:
      - token: token2

scenarios:
  arbitrum:
    orderbook: arbitrum
    runs: 1
    bindings:
      raindex-subparser: 0xb06202aA3Fe7d85171fB7aA5f17011d17E63f382
      fixed-io-output-token: \${order.outputs.0.token.address}
  polygon:
    orderbook: polygon
    runs: 1
    bindings:
      raindex-subparser: 0xF9323B7d23c655122Fb0272D989b83E105cBcf9d
      fixed-io-output-token: \${order.outputs.0.token.address}
  base:
    orderbook: base
    runs: 1
    bindings:
      raindex-subparser: 0x662dFd6d5B6DF94E07A60954901D3001c24F856a
      fixed-io-output-token: \${order.outputs.0.token.address}
  ethereum:
    orderbook: ethereum
    runs: 1
    bindings:
      raindex-subparser: 0x22410e2a46261a1B1e3899a072f303022801C764
      fixed-io-output-token: \${order.outputs.0.token.address}
  flare:
    orderbook: flare
    runs: 1
    bindings:
      raindex-subparser: 0xFe2411CDa193D9E4e83A5c234C7Fd320101883aC
      fixed-io-output-token: \${order.outputs.0.token.address}
  linea:
    orderbook: linea
    runs: 1
    bindings:
      raindex-subparser: 0xF77b3c3f61af5a3cE7f7CE3cfFc117491104432E
      fixed-io-output-token: \${order.outputs.0.token.address}
  bsc:
    orderbook: bsc
    runs: 1
    bindings:
      raindex-subparser: 0x662dFd6d5B6DF94E07A60954901D3001c24F856a
      fixed-io-output-token: \${order.outputs.0.token.address}

deployments:
  base:
    order: base
    scenario: base
  flare:
    order: flare
    scenario: flare
  arbitrum:
    order: arbitrum
    scenario: arbitrum
  polygon:
    order: polygon
    scenario: polygon
  ethereum:
    order: ethereum
    scenario: ethereum
  linea:
    order: linea
    scenario: linea
  bsc:
    order: bsc
    scenario: bsc

gui:
  name: Fixed limit
  description: A very simple strategy that places a limit order at a fixed price.
  short-description: A very simple strategy that places a limit order at a fixed price.
  deployments:
    base:
      name: Base
      description: Deploy a limit order on Base.
      deposits:
        - token: token2
      fields:
        - binding: fixed-io
          name: \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol}
          description: Fixed exchange rate (\${order.inputs.0.token.symbol} received per 1 \${order.outputs.0.token.symbol} sold)
      select-tokens:
        - key: token1
          name: Token to Buy
          description: Select the token you want to purchase
        - key: token2
          name: Token to Sell
          description: Select the token you want to sell
    flare:
      name: Flare
      description: Deploy a limit order on Flare.
      deposits:
        - token: token2
      fields:
        - binding: fixed-io
          name: \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol}
          description: Fixed exchange rate (\${order.inputs.0.token.symbol} received per 1 \${order.outputs.0.token.symbol} sold)
      select-tokens:
        - key: token1
          name: Token to Buy
          description: Select the token you want to purchase
        - key: token2
          name: Token to Sell
          description: Select the token you want to sell
    linea:
      name: Linea
      description: Deploy a limit order on Linea.
      deposits:
        - token: token2
      fields:
        - binding: fixed-io
          name: \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol}
          description: Fixed exchange rate (\${order.inputs.0.token.symbol} received per 1 \${order.outputs.0.token.symbol} sold)
      select-tokens:
        - key: token1
          name: Token to Buy
          description: Select the token you want to purchase
        - key: token2
          name: Token to Sell
          description: Select the token you want to sell
    bsc:
      name: BSC
      description: Deploy a limit order on BSC.
      deposits:
        - token: token2
      fields:
        - binding: fixed-io
          name: \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol}
          description: Fixed exchange rate (\${order.inputs.0.token.symbol} received per 1 \${order.outputs.0.token.symbol} sold)
      select-tokens:
        - key: token1
          name: Token to Buy
          description: Select the token you want to purchase
        - key: token2
          name: Token to Sell
          description: Select the token you want to sell
    polygon:
      name: Polygon
      description: Deploy a limit order on Polygon.
      deposits:
        - token: token2
      fields:
        - binding: fixed-io
          name: \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol}
          description: Fixed exchange rate (\${order.inputs.0.token.symbol} received per 1 \${order.outputs.0.token.symbol} sold)
      select-tokens:
        - key: token1
          name: Token to Buy
          description: Select the token you want to purchase
        - key: token2
          name: Token to Sell
          description: Select the token you want to sell
    arbitrum:
      name: Arbitrum
      description: Deploy a limit order on Arbitrum.
      deposits:
        - token: token2
      fields:
        - binding: fixed-io
          name: \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol}
          description: Fixed exchange rate (\${order.inputs.0.token.symbol} received per 1 \${order.outputs.0.token.symbol} sold)
      select-tokens:
        - key: token1
          name: Token to Buy
          description: Select the token you want to purchase
        - key: token2
          name: Token to Sell
          description: Select the token you want to sell
    ethereum:
      name: Ethereum
      description: Deploy a limit order on Ethereum.
      deposits:
        - token: token2
      fields:
        - binding: fixed-io
          name: \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol}
          description: Fixed exchange rate (\${order.inputs.0.token.symbol} received per 1 \${order.outputs.0.token.symbol} sold)
      select-tokens:
        - key: token1
          name: Token to Buy
          description: Select the token you want to purchase
        - key: token2
          name: Token to Sell
          description: Select the token you want to sell

---
#raindex-subparser !The subparser to use.

#fixed-io !The io ratio for the limit order.
#fixed-io-output-token !The output token that the fixed io is for. If this doesn't match the runtime output then the fixed-io will be inverted.

#calculate-io
using-words-from raindex-subparser
max-output: max-value(),
io: if(
  equal-to(
    output-token()
    fixed-io-output-token
  )
  fixed-io
  inv(fixed-io)
);

#handle-io
:;

#handle-add-order
:;
`;

const auctionStrategy = `
raindex-version: 8898591f3bcaa21dc91dc3b8584330fc405eadfa

networks:
  arbitrum:
    rpc: https://1rpc.io/arb
    chain-id: 42161
    network-id: 42161
    currency: ETH
  bsc:
    rpc: https://bsc-dataseed.bnbchain.org
    chain-id: 56
    network-id: 56
    currency: BNB
  base:
    rpc: https://base-rpc.publicnode.com
    chain-id: 8453
    network-id: 8453
    currency: ETH
  ethereum:
    rpc: https://1rpc.io/eth
    chain-id: 1
    network-id: 1
    currency: ETH
  flare:
    rpc: https://flare.rpc.thirdweb.com
    chain-id: 14
    network-id: 14
    currency: FLR
  polygon:
    rpc: https://1rpc.io/matic
    chain-id: 137
    network-id: 137
    currency: POL

subgraphs:
  arbitrum: https://example.com/subgraph
  bsc: https://example.com/subgraph
  base: https://example.com/subgraph
  ethereum: https://example.com/subgraph
  flare: https://example.com/subgraph
  polygon: https://example.com/subgraph

metaboards:
  arbitrum: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-arbitrum/0.1/gn
  bsc: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-bsc/0.1/gn
  base: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-base/0.1/gn
  ethereum: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/metadata-mainnet/2024-10-25-2857/gn
  flare: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-flare-0x893BBFB7/0.1/gn
  polygon: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-polygon/0.1/gn

orderbooks:
  arbitrum:
    address: 0x550878091b2B1506069F61ae59e3A5484Bca9166
  bsc:
    address: 0xd2938E7c9fe3597F78832CE780Feb61945c377d7
  base:
    address: 0xd2938e7c9fe3597f78832ce780feb61945c377d7
  ethereum:
    address: 0x0eA6d458488d1cf51695e1D6e4744e6FB715d37C
  flare:
    address: 0xCEe8Cd002F151A536394E564b84076c41bBBcD4d
  polygon:
    address: 0x7D2f700b1f6FD75734824EA4578960747bdF269A

deployers:
  arbitrum:
    address: 0x9B0D254bd858208074De3d2DaF5af11b3D2F377F
  bsc:
    address: 0xA2f56F8F74B7d04d61f281BE6576b6155581dcBA
  base:
    address: 0xC1A14cE2fd58A3A2f99deCb8eDd866204eE07f8D
  ethereum:
    address: 0xd19581a021f4704ad4eBfF68258e7A0a9DB1CD77
  flare:
    address: 0xE3989Ea7486c0F418C764e6c511e86f6E8830FAb
  polygon:
    address: 0xE7116BC05C8afe25e5B54b813A74F916B5D42aB1

tokens:
  flare-wflr:
    network: flare
    address: 0x1D80c49BbBCd1C0911346656B529DF9E5c2F783d
    decimals: 18
  flare-sflr:
    network: flare
    address: 0x12e605bc104e93B45e1aD99F9e555f659051c2BB
    decimals: 18

orders:
  arbitrum:
    orderbook: arbitrum
    inputs:
      - token: input
    outputs:
      - token: output
  bsc:
    orderbook: bsc
    inputs:
      - token: input
    outputs:
      - token: output
  base:
    orderbook: base
    inputs:
      - token: input
    outputs:
      - token: output
  ethereum:
    orderbook: ethereum
    inputs:
      - token: input
    outputs:
      - token: output
  flare:
    orderbook: flare
    inputs:
      - token: input
    outputs:
      - token: output
  polygon:
    orderbook: polygon
    inputs:
      - token: input
    outputs:
      - token: output
  flare-sflr-wflr:
    orderbook: flare
    inputs:
      - token: flare-sflr
    outputs:
      - token: flare-wflr
  flare-wflr-sflr:
    orderbook: flare
    inputs:
      - token: flare-wflr
    outputs:
      - token: flare-sflr

scenarios:
  arbitrum:
    orderbook: arbitrum
    runs: 1
    bindings:
      raindex-subparser: 0xb06202aA3Fe7d85171fB7aA5f17011d17E63f382
      subparser-0: 0xb06202aA3Fe7d85171fB7aA5f17011d17E63f382
      baseline-fn: '''constant-baseline'
      initial-io-fn: '''constant-initial-io'
      shy-epoch: 0.05
  bsc:
    orderbook: bsc
    runs: 1
    bindings:
      raindex-subparser: 0x662dFd6d5B6DF94E07A60954901D3001c24F856a
      subparser-0: 0x662dFd6d5B6DF94E07A60954901D3001c24F856a
      baseline-fn: '''constant-baseline'
      initial-io-fn: '''constant-initial-io'
      shy-epoch: 0.05
  base:
    orderbook: base
    runs: 1
    bindings:
      raindex-subparser: 0x662dFd6d5B6DF94E07A60954901D3001c24F856a
      subparser-0: 0x662dFd6d5B6DF94E07A60954901D3001c24F856a
      baseline-fn: '''constant-baseline'
      initial-io-fn: '''constant-initial-io'
      shy-epoch: 0.05
  polygon:
    orderbook: polygon
    runs: 1
    bindings:
      raindex-subparser: 0xF9323B7d23c655122Fb0272D989b83E105cBcf9d
      subparser-0: 0xF9323B7d23c655122Fb0272D989b83E105cBcf9d
      baseline-fn: '''constant-baseline'
      initial-io-fn: '''constant-initial-io'
      shy-epoch: 0.05
  ethereum:
    orderbook: ethereum
    runs: 1
    bindings:
      raindex-subparser: 0x22410e2a46261a1B1e3899a072f303022801C764
      subparser-0: 0x22410e2a46261a1B1e3899a072f303022801C764
      baseline-fn: '''constant-baseline'
      initial-io-fn: '''constant-initial-io'
      shy-epoch: 0.05
  flare:
    orderbook: flare
    runs: 1
    bindings:
      raindex-subparser: 0xFe2411CDa193D9E4e83A5c234C7Fd320101883aC
      subparser-0: 0x915E36ef882941816356bC3718Df868054F868aD
      baseline-fn: '''constant-baseline'
      initial-io-fn: '''constant-initial-io'
      shy-epoch: 0.05
  flare-sflr-baseline:
    orderbook: flare
    deployer: flare
    runs: 1
    bindings:
      raindex-subparser: 0xFe2411CDa193D9E4e83A5c234C7Fd320101883aC
      subparser-0: 0x915E36ef882941816356bC3718Df868054F868aD
      baseline-fn: '''sflr-baseline'
      initial-io-fn: '''sflr-baseline'
      next-trade-baseline-multiplier: 0
      shy-epoch: 0.05
  flare-sflr-baseline-inv:
    orderbook: flare
    deployer: flare
    runs: 1
    bindings:
      raindex-subparser: 0xFe2411CDa193D9E4e83A5c234C7Fd320101883aC
      subparser-0: 0x915E36ef882941816356bC3718Df868054F868aD
      baseline-fn: '''sflr-baseline-inv'
      initial-io-fn: '''sflr-baseline-inv'
      next-trade-baseline-multiplier: 0
      shy-epoch: 0.05

deployments:
  arbitrum:
    order: arbitrum
    scenario: arbitrum
  polygon:
    order: polygon
    scenario: polygon
  bsc:
    order: bsc
    scenario: bsc
  base:
    order: base
    scenario: base
  ethereum:
    order: ethereum
    scenario: ethereum
  flare:
    order: flare
    scenario: flare
  flare-sflr-wflr:
    order: flare-sflr-wflr
    scenario: flare-sflr-baseline
  flare-wflr-sflr:
    order: flare-wflr-sflr
    scenario: flare-sflr-baseline-inv

gui:
  name: Auction based cost averaging
  description: https://raw.githubusercontent.com/rainlanguage/rain.strategies/e25bc4876b5ffb8bb28097b0ca66de291c75ff56/src/auction-dca.md
  short-description:  >
    A strategy that aims to fill a time-based budget via looping auctions - useful for breaking up large trades into smaller amounts or smoothing out market volatility for regular investments.
  deployments:
    arbitrum:
      name: Arbitrum
      description: Deploy an auction-based cost averaging strategy on Arbitrum.
      deposits:
        - token: output
      fields:
        - binding: time-per-amount-epoch
          name: Budget period (in seconds)
          description: |
            The budget is spent over this time period.

            For example, if the budget is daily then this is 86400 seconds (24 * 60 * 60).
          show-custom-field: true
          presets:
            - name: Per minute (60)
              value: 60
            - name: Per hour (3600)
              value: 3600
            - name: Per day (86400)
              value: 86400
            - name: Per week (604800)
              value: 604800
            - name: Per 30 days (2592000)
              value: 2592000
            - name: Per 365 days (31536000)
              value: 31536000
        - binding: amount-per-epoch
          name: Budget (\${order.outputs.0.token.symbol} per period)
          description: |
            The amount of \${order.outputs.0.token.symbol} to spend each budget period.

            For example, if the budget is daily and this is 10 then 10 \${order.outputs.0.token.symbol} will be sold for \${order.inputs.0.token.symbol} each day.
        - binding: max-trade-amount
          name: Maximum trade size (\${order.outputs.0.token.symbol})
          description: |
            The maximum amount of \${order.outputs.0.token.symbol} to sell in a single auction.
        - binding: min-trade-amount
          name: Minimum trade size (\${order.outputs.0.token.symbol})
          description: |
            The minimum amount of \${order.outputs.0.token.symbol} to sell in a single auction.
        - binding: time-per-trade-epoch
          name: Auction period (in seconds)
          description: |
            The auction period is the time between each auction price halvening.
          show-custom-field: true
          default: 3600
          presets:
            - name: Every 20 minutes (1200)
              value: 1200
            - name: Every 30 minutes (1800)
              value: 1800
            - name: Every hour (3600)
              value: 3600
            - name: Every 2 hours (7200)
              value: 7200
            - name: Every 3 hours (10800)
              value: 10800
            - name: Every 6 hours (21600)
              value: 21600
            - name: Every 12 hours (43200)
              value: 43200
            - name: Every 24 hours (86400)
              value: 86400
        - binding: baseline
          name: Baseline \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol}
          description: |
            The absolute minimum amount of \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol} that the auction will trade at.

            I.e. for each 1 \${order.outputs.0.token.symbol} sold you will receive at least this many \${order.inputs.0.token.symbol}.

            I.e. this is calculated as the \${order.outputs.0.token.symbol} $ price divided by the \${order.inputs.0.token.symbol} $ price.
            You can find $ prices for most tokens on dex tools, dex screener and gecko terminal.

            For more information about IO ratios and how to calculate them see [Understanding IO Ratios](https://youtu.be/NdPOi1ZDnDk).
        - binding: initial-io
          name: Kickoff \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol}
          description: |
            The initial \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol} to kickoff the first auction.

            This ratio is calculated in the same way as the baseline ratio.

            It must be greater than the baseline ratio, regardless of what you are selling or buying.
            This is because getting more input per output is always better for you.
            The initial auction will start high (at the value you set here) and then drop to the baseline ratio over time.
            Subsequent auctions will start at some % above the last auction price and drop to the baseline ratio over time.
        - binding: next-trade-multiplier
          name: Auction start multiplier
          description: |
            The multiplier to apply to the last trade to kick off the next auction.
          show-custom-field: true
          default: 1.01
          presets:
            - name: 1.01x
              value: 1.01
            - name: 1.02x
              value: 1.02
            - name: 1.05x
              value: 1.05
            - name: 1.1x
              value: 1.1
        - binding: next-trade-baseline-multiplier
          name: Auction end multiplier
          description: |
            The multiplier to apply to the last trade to set the baseline for the next auction.
          show-custom-field: true
          default: 0
          presets:
            - name: Disabled (0)
              value: 0
            - name: 0.7x
              value: 0.7
            - name: 0.8x
              value: 0.8
            - name: 0.9x
              value: 0.9
            - name: 0.95x
              value: 0.95
            - name: 0.99x
              value: 0.99
      select-tokens:
        - key: output
          name: Token to Sell
          description: Select the token you want to sell
        - key: input
          name: Token to Buy
          description: Select the token you want to purchase
        

    polygon:
      name: Polygon
      description: Deploy an auction-based cost averaging strategy on Polygon.
      deposits:
        - token: output
      fields:
        - binding: time-per-amount-epoch
          name: Budget period (in seconds)
          description: |
            The budget is spent over this time period.

            For example, if the budget is daily then this is 86400 seconds (24 * 60 * 60).
          show-custom-field: true
          presets:
            - name: Per minute (60)
              value: 60
            - name: Per hour (3600)
              value: 3600
            - name: Per day (86400)
              value: 86400
            - name: Per week (604800)
              value: 604800
            - name: Per 30 days (2592000)
              value: 2592000
            - name: Per 365 days (31536000)
              value: 31536000
        - binding: amount-per-epoch
          name: Budget (\${order.outputs.0.token.symbol} per period)
          description: |
            The amount of \${order.outputs.0.token.symbol} to spend each budget period.

            For example, if the budget is daily and this is 10 then 10 \${order.outputs.0.token.symbol} will be sold for \${order.inputs.0.token.symbol} each day.
        - binding: max-trade-amount
          name: Maximum trade size (\${order.outputs.0.token.symbol})
          description: |
            The maximum amount of \${order.outputs.0.token.symbol} to sell in a single auction.
        - binding: min-trade-amount
          name: Minimum trade size (\${order.outputs.0.token.symbol})
          description: |
            The minimum amount of \${order.outputs.0.token.symbol} to sell in a single auction.
        - binding: time-per-trade-epoch
          name: Auction period (in seconds)
          description: |
            The auction period is the time between each auction price halvening.
          show-custom-field: true
          default: 3600
          presets:
            - name: Every 20 minutes (1200)
              value: 1200
            - name: Every 30 minutes (1800)
              value: 1800
            - name: Every hour (3600)
              value: 3600
            - name: Every 2 hours (7200)
              value: 7200
            - name: Every 3 hours (10800)
              value: 10800
            - name: Every 6 hours (21600)
              value: 21600
            - name: Every 12 hours (43200)
              value: 43200
            - name: Every 24 hours (86400)
              value: 86400
        - binding: baseline
          name: Baseline \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol}
          description: |
            The absolute minimum amount of \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol} that the auction will trade at.

            I.e. for each 1 \${order.outputs.0.token.symbol} sold you will receive at least this many \${order.inputs.0.token.symbol}.

            I.e. this is calculated as the \${order.outputs.0.token.symbol} $ price divided by the \${order.inputs.0.token.symbol} $ price.
            You can find $ prices for most tokens on dex tools, dex screener and gecko terminal.

            For more information about IO ratios and how to calculate them see [Understanding IO Ratios](https://youtu.be/NdPOi1ZDnDk).
        - binding: initial-io
          name: Kickoff \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol}
          description: |
            The initial \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol} to kickoff the first auction.

            This ratio is calculated in the same way as the baseline ratio.

            It must be greater than the baseline ratio, regardless of what you are selling or buying.
            This is because getting more input per output is always better for you.
            The initial auction will start high (at the value you set here) and then drop to the baseline ratio over time.
            Subsequent auctions will start at some % above the last auction price and drop to the baseline ratio over time.
        - binding: next-trade-multiplier
          name: Auction start multiplier
          description: |
            The multiplier to apply to the last trade to kick off the next auction.
          show-custom-field: true
          default: 1.01
          presets:
            - name: 1.01x
              value: 1.01
            - name: 1.02x
              value: 1.02
            - name: 1.05x
              value: 1.05
            - name: 1.1x
              value: 1.1
        - binding: next-trade-baseline-multiplier
          name: Auction end multiplier
          description: |
            The multiplier to apply to the last trade to set the baseline for the next auction.
          show-custom-field: true
          default: 0
          presets:
            - name: Disabled (0)
              value: 0
            - name: 0.7x
              value: 0.7
            - name: 0.8x
              value: 0.8
            - name: 0.9x
              value: 0.9
            - name: 0.95x
              value: 0.95
            - name: 0.99x
              value: 0.99
      select-tokens:
        - key: output
          name: Token to Sell
          description: Select the token you want to sell
        - key: input
          name: Token to Buy
          description: Select the token you want to purchase

    bsc:
      name: BSC
      description: Deploy an auction-based cost averaging strategy on BSC.
      deposits:
        - token: output
      fields:
        - binding: time-per-amount-epoch
          name: Budget period (in seconds)
          description: |
            The budget is spent over this time period.

            For example, if the budget is daily then this is 86400 seconds (24 * 60 * 60).
          show-custom-field: true
          presets:
            - name: Per minute (60)
              value: 60
            - name: Per hour (3600)
              value: 3600
            - name: Per day (86400)
              value: 86400
            - name: Per week (604800)
              value: 604800
            - name: Per 30 days (2592000)
              value: 2592000
            - name: Per 365 days (31536000)
              value: 31536000
        - binding: amount-per-epoch
          name: Budget (\${order.outputs.0.token.symbol} per period)
          description: |
            The amount of \${order.outputs.0.token.symbol} to spend each budget period.

            For example, if the budget is daily and this is 10 then 10 \${order.outputs.0.token.symbol} will be sold for \${order.inputs.0.token.symbol} each day.
        - binding: max-trade-amount
          name: Maximum trade size (\${order.outputs.0.token.symbol})
          description: |
            The maximum amount of \${order.outputs.0.token.symbol} to sell in a single auction.
        - binding: min-trade-amount
          name: Minimum trade size (\${order.outputs.0.token.symbol})
          description: |
            The minimum amount of \${order.outputs.0.token.symbol} to sell in a single auction.
        - binding: time-per-trade-epoch
          name: Auction period (in seconds)
          description: |
            The auction period is the time between each auction price halvening.
          show-custom-field: true
          default: 3600
          presets:
            - name: Every 20 minutes (1200)
              value: 1200
            - name: Every 30 minutes (1800)
              value: 1800
            - name: Every hour (3600)
              value: 3600
            - name: Every 2 hours (7200)
              value: 7200
            - name: Every 3 hours (10800)
              value: 10800
            - name: Every 6 hours (21600)
              value: 21600
            - name: Every 12 hours (43200)
              value: 43200
            - name: Every 24 hours (86400)
              value: 86400
        - binding: baseline
          name: Baseline \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol}
          description: |
            The absolute minimum amount of \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol} that the auction will trade at.

            I.e. for each 1 \${order.outputs.0.token.symbol} sold you will receive at least this many \${order.inputs.0.token.symbol}.

            I.e. this is calculated as the \${order.outputs.0.token.symbol} $ price divided by the \${order.inputs.0.token.symbol} $ price.
            You can find $ prices for most tokens on dex tools, dex screener and gecko terminal.

            For more information about IO ratios and how to calculate them see [Understanding IO Ratios](https://youtu.be/NdPOi1ZDnDk).
        - binding: initial-io
          name: Kickoff \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol}
          description: |
            The initial \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol} to kickoff the first auction.

            This ratio is calculated in the same way as the baseline ratio.

            It must be greater than the baseline ratio, regardless of what you are selling or buying.
            This is because getting more input per output is always better for you.
            The initial auction will start high (at the value you set here) and then drop to the baseline ratio over time.
            Subsequent auctions will start at some % above the last auction price and drop to the baseline ratio over time.
        - binding: next-trade-multiplier
          name: Auction start multiplier
          description: |
            The multiplier to apply to the last trade to kick off the next auction.
          show-custom-field: true
          default: 1.01
          presets:
            - name: 1.01x
              value: 1.01
            - name: 1.02x
              value: 1.02
            - name: 1.05x
              value: 1.05
            - name: 1.1x
              value: 1.1
        - binding: next-trade-baseline-multiplier
          name: Auction end multiplier
          description: |
            The multiplier to apply to the last trade to set the baseline for the next auction.
          show-custom-field: true
          default: 0
          presets:
            - name: Disabled (0)
              value: 0
            - name: 0.7x
              value: 0.7
            - name: 0.8x
              value: 0.8
            - name: 0.9x
              value: 0.9
            - name: 0.95x
              value: 0.95
            - name: 0.99x
              value: 0.99
      select-tokens:
        - key: output
          name: Token to Sell
          description: Select the token you want to sell
        - key: input
          name: Token to Buy
          description: Select the token you want to purchase

    base:
      name: Base
      description: Deploy an auction-based cost averaging strategy on Base.
      deposits:
        - token: output
      fields:
        - binding: time-per-amount-epoch
          name: Budget period (in seconds)
          description: |
            The budget is spent over this time period.

            For example, if the budget is daily then this is 86400 seconds (24 * 60 * 60).
          show-custom-field: true
          presets:
            - name: Per minute (60)
              value: 60
            - name: Per hour (3600)
              value: 3600
            - name: Per day (86400)
              value: 86400
            - name: Per week (604800)
              value: 604800
            - name: Per 30 days (2592000)
              value: 2592000
            - name: Per 365 days (31536000)
              value: 31536000
        - binding: amount-per-epoch
          name: Budget (\${order.outputs.0.token.symbol} per period)
          description: |
            The amount of \${order.outputs.0.token.symbol} to spend each budget period.

            For example, if the budget is daily and this is 10 then 10 \${order.outputs.0.token.symbol} will be sold for \${order.inputs.0.token.symbol} each day.
        - binding: max-trade-amount
          name: Maximum trade size (\${order.outputs.0.token.symbol})
          description: |
            The maximum amount of \${order.outputs.0.token.symbol} to sell in a single auction.
        - binding: min-trade-amount
          name: Minimum trade size (\${order.outputs.0.token.symbol})
          description: |
            The minimum amount of \${order.outputs.0.token.symbol} to sell in a single auction.
        - binding: time-per-trade-epoch
          name: Auction period (in seconds)
          description: |
            The auction period is the time between each auction price halvening.
          show-custom-field: true
          default: 3600
          presets:
            - name: Every 20 minutes (1200)
              value: 1200
            - name: Every 30 minutes (1800)
              value: 1800
            - name: Every hour (3600)
              value: 3600
            - name: Every 2 hours (7200)
              value: 7200
            - name: Every 3 hours (10800)
              value: 10800
            - name: Every 6 hours (21600)
              value: 21600
            - name: Every 12 hours (43200)
              value: 43200
            - name: Every 24 hours (86400)
              value: 86400
        - binding: baseline
          name: Baseline \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol}
          description: |
            The absolute minimum amount of \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol} that the auction will trade at.

            I.e. for each 1 \${order.outputs.0.token.symbol} sold you will receive at least this many \${order.inputs.0.token.symbol}.

            I.e. this is calculated as the \${order.outputs.0.token.symbol} $ price divided by the \${order.inputs.0.token.symbol} $ price.
            You can find $ prices for most tokens on dex tools, dex screener and gecko terminal.

            For more information about IO ratios and how to calculate them see [Understanding IO Ratios](https://youtu.be/NdPOi1ZDnDk).
        - binding: initial-io
          name: Kickoff \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol}
          description: |
            The initial \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol} to kickoff the first auction.

            This ratio is calculated in the same way as the baseline ratio.

            It must be greater than the baseline ratio, regardless of what you are selling or buying.
            This is because getting more input per output is always better for you.
            The initial auction will start high (at the value you set here) and then drop to the baseline ratio over time.
            Subsequent auctions will start at some % above the last auction price and drop to the baseline ratio over time.
        - binding: next-trade-multiplier
          name: Auction start multiplier
          description: |
            The multiplier to apply to the last trade to kick off the next auction.
          show-custom-field: true
          default: 1.01
          presets:
            - name: 1.01x
              value: 1.01
            - name: 1.02x
              value: 1.02
            - name: 1.05x
              value: 1.05
            - name: 1.1x
              value: 1.1
        - binding: next-trade-baseline-multiplier
          name: Auction end multiplier
          description: |
            The multiplier to apply to the last trade to set the baseline for the next auction.
          show-custom-field: true
          default: 0
          presets:
            - name: Disabled (0)
              value: 0
            - name: 0.7x
              value: 0.7
            - name: 0.8x
              value: 0.8
            - name: 0.9x
              value: 0.9
            - name: 0.95x
              value: 0.95
            - name: 0.99x
              value: 0.99
      select-tokens:
        - key: output
          name: Token to Sell
          description: Select the token you want to sell
        - key: input
          name: Token to Buy
          description: Select the token you want to purchase

    ethereum:
      name: Ethereum
      description: Deploy an auction-based cost averaging strategy on Ethereum.
      deposits:
        - token: output
      fields:
        - binding: time-per-amount-epoch
          name: Budget period (in seconds)
          description: |
            The budget is spent over this time period.

            For example, if the budget is daily then this is 86400 seconds (24 * 60 * 60).
          show-custom-field: true
          presets:
            - name: Per minute (60)
              value: 60
            - name: Per hour (3600)
              value: 3600
            - name: Per day (86400)
              value: 86400
            - name: Per week (604800)
              value: 604800
            - name: Per 30 days (2592000)
              value: 2592000
            - name: Per 365 days (31536000)
              value: 31536000
        - binding: amount-per-epoch
          name: Budget (\${order.outputs.0.token.symbol} per period)
          description: |
            The amount of \${order.outputs.0.token.symbol} to spend each budget period.

            For example, if the budget is daily and this is 10 then 10 \${order.outputs.0.token.symbol} will be sold for \${order.inputs.0.token.symbol} each day.
        - binding: max-trade-amount
          name: Maximum trade size (\${order.outputs.0.token.symbol})
          description: |
            The maximum amount of \${order.outputs.0.token.symbol} to sell in a single auction.
        - binding: min-trade-amount
          name: Minimum trade size (\${order.outputs.0.token.symbol})
          description: |
            The minimum amount of \${order.outputs.0.token.symbol} to sell in a single auction.
        - binding: time-per-trade-epoch
          name: Auction period (in seconds)
          description: |
            The auction period is the time between each auction price halvening.
          show-custom-field: true
          default: 3600
          presets:
            - name: Every 20 minutes (1200)
              value: 1200
            - name: Every 30 minutes (1800)
              value: 1800
            - name: Every hour (3600)
              value: 3600
            - name: Every 2 hours (7200)
              value: 7200
            - name: Every 3 hours (10800)
              value: 10800
            - name: Every 6 hours (21600)
              value: 21600
            - name: Every 12 hours (43200)
              value: 43200
            - name: Every 24 hours (86400)
              value: 86400
        - binding: baseline
          name: Baseline \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol}
          description: |
            The absolute minimum amount of \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol} that the auction will trade at.

            I.e. for each 1 \${order.outputs.0.token.symbol} sold you will receive at least this many \${order.inputs.0.token.symbol}.

            I.e. this is calculated as the \${order.outputs.0.token.symbol} $ price divided by the \${order.inputs.0.token.symbol} $ price.
            You can find $ prices for most tokens on dex tools, dex screener and gecko terminal.

            For more information about IO ratios and how to calculate them see [Understanding IO Ratios](https://youtu.be/NdPOi1ZDnDk).
        - binding: initial-io
          name: Kickoff \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol}
          description: |
            The initial \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol} to kickoff the first auction.

            This ratio is calculated in the same way as the baseline ratio.

            It must be greater than the baseline ratio, regardless of what you are selling or buying.
            This is because getting more input per output is always better for you.
            The initial auction will start high (at the value you set here) and then drop to the baseline ratio over time.
            Subsequent auctions will start at some % above the last auction price and drop to the baseline ratio over time.
        - binding: next-trade-multiplier
          name: Auction start multiplier
          description: |
            The multiplier to apply to the last trade to kick off the next auction.
          show-custom-field: true
          default: 1.01
          presets:
            - name: 1.01x
              value: 1.01
            - name: 1.02x
              value: 1.02
            - name: 1.05x
              value: 1.05
            - name: 1.1x
              value: 1.1
        - binding: next-trade-baseline-multiplier
          name: Auction end multiplier
          description: |
            The multiplier to apply to the last trade to set the baseline for the next auction.
          show-custom-field: true
          default: 0
          presets:
            - name: Disabled (0)
              value: 0
            - name: 0.7x
              value: 0.7
            - name: 0.8x
              value: 0.8
            - name: 0.9x
              value: 0.9
            - name: 0.95x
              value: 0.95
            - name: 0.99x
              value: 0.99
      select-tokens:
        - key: output
          name: Token to Sell
          description: Select the token you want to sell
        - key: input
          name: Token to Buy
          description: Select the token you want to purchase

    flare:
      name: Flare
      description: Deploy an auction-based cost averaging strategy on Flare.
      deposits:
        - token: output
      fields:
        - binding: time-per-amount-epoch
          name: Budget period (in seconds)
          description: |
            The budget is spent over this time period.

            For example, if the budget is daily then this is 86400 seconds (24 * 60 * 60).
          show-custom-field: true
          presets:
            - name: Per minute (60)
              value: 60
            - name: Per hour (3600)
              value: 3600
            - name: Per day (86400)
              value: 86400
            - name: Per week (604800)
              value: 604800
            - name: Per 30 days (2592000)
              value: 2592000
            - name: Per 365 days (31536000)
              value: 31536000
        - binding: amount-per-epoch
          name: Budget (\${order.outputs.0.token.symbol} per period)
          description: |
            The amount of \${order.outputs.0.token.symbol} to spend each budget period.

            For example, if the budget is daily and this is 10 then 10 \${order.outputs.0.token.symbol} will be sold for \${order.inputs.0.token.symbol} each day.
        - binding: max-trade-amount
          name: Maximum trade size (\${order.outputs.0.token.symbol})
          description: |
            The maximum amount of \${order.outputs.0.token.symbol} to sell in a single auction.
        - binding: min-trade-amount
          name: Minimum trade size (\${order.outputs.0.token.symbol})
          description: |
            The minimum amount of \${order.outputs.0.token.symbol} to sell in a single auction.
        - binding: time-per-trade-epoch
          name: Auction period (in seconds)
          description: |
            The auction period is the time between each auction price halvening.
          show-custom-field: true
          default: 3600
          presets:
            - name: Every 20 minutes (1200)
              value: 1200
            - name: Every 30 minutes (1800)
              value: 1800
            - name: Every hour (3600)
              value: 3600
            - name: Every 2 hours (7200)
              value: 7200
            - name: Every 3 hours (10800)
              value: 10800
            - name: Every 6 hours (21600)
              value: 21600
            - name: Every 12 hours (43200)
              value: 43200
            - name: Every 24 hours (86400)
              value: 86400
        - binding: baseline
          name: Baseline \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol}
          description: |
            The absolute minimum amount of \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol} that the auction will trade at.

            I.e. for each 1 \${order.outputs.0.token.symbol} sold you will receive at least this many \${order.inputs.0.token.symbol}.

            I.e. this is calculated as the \${order.outputs.0.token.symbol} $ price divided by the \${order.inputs.0.token.symbol} $ price.
            You can find $ prices for most tokens on dex tools, dex screener and gecko terminal.

            For more information about IO ratios and how to calculate them see [Understanding IO Ratios](https://youtu.be/NdPOi1ZDnDk).
        - binding: initial-io
          name: Kickoff \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol}
          description: |
            The initial \${order.inputs.0.token.symbol} per \${order.outputs.0.token.symbol} to kickoff the first auction.

            This ratio is calculated in the same way as the baseline ratio.

            It must be greater than the baseline ratio, regardless of what you are selling or buying.
            This is because getting more input per output is always better for you.
            The initial auction will start high (at the value you set here) and then drop to the baseline ratio over time.
            Subsequent auctions will start at some % above the last auction price and drop to the baseline ratio over time.
        - binding: next-trade-multiplier
          name: Auction start multiplier
          description: |
            The multiplier to apply to the last trade to kick off the next auction.
          show-custom-field: true
          default: 1.01
          presets:
            - name: 1.01x
              value: 1.01
            - name: 1.02x
              value: 1.02
            - name: 1.05x
              value: 1.05
            - name: 1.1x
              value: 1.1
        - binding: next-trade-baseline-multiplier
          name: Auction end multiplier
          description: |
            The multiplier to apply to the last trade to set the baseline for the next auction.
          show-custom-field: true
          default: 0
          presets:
            - name: Disabled (0)
              value: 0
            - name: 0.7x
              value: 0.7
            - name: 0.8x
              value: 0.8
            - name: 0.9x
              value: 0.9
            - name: 0.95x
              value: 0.95
            - name: 0.99x
              value: 0.99
      select-tokens:
        - key: output
          name: Token to Sell
          description: Select the token you want to sell
        - key: input
          name: Token to Buy
          description: Select the token you want to purchase

    flare-sflr-wflr:
      name: Sell WFLR for SFLR on Flare based on underlying collateral.
      description: |
        Swap WFLR for SFLR on Flare based on underlying collateral.
      deposits:
        - token: flare-wflr
      fields:
        - binding: time-per-amount-epoch
          name: Budget period (in seconds)
          description: |
            The budget is spent over this time period.
          show-custom-field: true
          presets:
            - name: Per minute (60)
              value: 60
            - name: Per hour (3600)
              value: 3600
            - name: Per day (86400)
              value: 86400
            - name: Per week (604800)
              value: 604800
        - binding: amount-per-epoch
          name: Budget (WFLR per period)
          description: |
            The amount of WFLR to spend each budget period.
        - binding: max-trade-amount
          name: Maximum trade size (WFLR)
          description: |
            The maximum amount of WFLR to sell in a single auction.
        - binding: min-trade-amount
          name: Minimum trade size (WFLR)
          description: |
            The minimum amount of WFLR to sell in a single auction.
        - binding: time-per-trade-epoch
          name: Auction period (in seconds)
          description: |
            The auction period is the time between each auction price halvening.
          show-custom-field: true
          default: 3600
          presets:
            - name: Every half hour (1800)
              value: 1800
            - name: Every hour (3600)
              value: 3600
            - name: Every 2 hours (7200)
              value: 7200
        - binding: next-trade-multiplier
          name: Auction start multiplier
          description: |
            The multiplier to apply to the last trade to kick off the next auction.
          show-custom-field: true
          default: 1.01
          presets:
            - name: 1.01x
              value: 1.01
            - name: 1.02x
              value: 1.02
            - name: 1.05x
              value: 1.05

    flare-wflr-sflr:
      name: Sell SFLR for WFLR on Flare based on underlying collateral.
      description: |
        Swap SFLR for WFLR on Flare based on underlying collateral.
      deposits:
        - token: flare-sflr
      fields:
        - binding: time-per-amount-epoch
          name: Budget period (in seconds)
          description: |
            The budget is spent over this time period.
          show-custom-field: true
          presets:
            - name: Per minute (60)
              value: 60
            - name: Per hour (3600)
              value: 3600
            - name: Per day (86400)
              value: 86400
            - name: Per week (604800)
              value: 604800
        - binding: amount-per-epoch
          name: Budget (SFLR per period)
          description: |
            The amount of SFLR to spend each budget period.
        - binding: max-trade-amount
          name: Maximum trade size (SFLR)
          description: |
            The maximum amount of SFLR to sell in a single auction.
        - binding: min-trade-amount
          name: Minimum trade size (SFLR)
          description: |
            The minimum amount of SFLR to sell in a single auction.
        - binding: time-per-trade-epoch
          name: Auction period (in seconds)
          description: |
            The auction period is the time between each auction price halvening.
          show-custom-field: true
          default: 3600
          presets:
            - name: Every half hour (1800)
              value: 1800
            - name: Every hour (3600)
              value: 3600
            - name: Every 2 hours (7200)
              value: 7200
        - binding: next-trade-multiplier
          name: Auction start multiplier
          description: |
            The multiplier to apply to the last trade to kick off the next auction.
          show-custom-field: true
          default: 1.01
          presets:
            - name: 1.01x
              value: 1.01
            - name: 1.02x
              value: 1.02
            - name: 1.05x
              value: 1.05

---
#raindex-subparser !Raindex subparser.
#subparser-0 !Subparser 0.

#time-per-amount-epoch !Duration of one unit of streaming amount halflife.
#amount-per-epoch !Amount of output token to approve for buying per epoch.
#min-trade-amount !Each trade must be at least this many output tokens.
#max-trade-amount !Each trade will be capped at this many tokens.

#time-per-trade-epoch !Duration of one unit of io ratio halflife.
#shy-epoch !Epoch below which only the minimum amount is offered.

#start-price-input !Price of the input token to start trading.
#start-price-output !Price of the output token to start trading.
#baseline-relative-ratio !Ratio of the start price to use as the baseline.
#initial-relative-ratio !Ratio of the start price to use as the initial io ratio.

#initial-io !Initial io ratio to use for the first trade. Must be greater than baseline.
#baseline !Baseline io ratio to use for the first trade.

#baseline-fn !Function to calculate the baseline for the next trade.
#initial-io-fn !Function to calculate the initial io ratio for the first trade.

#next-trade-multiplier !Start next auction at this x the last trade.
#next-trade-baseline-multiplier !Lifts the baseline to here relative to the previous trade.

#last-trade-time-key "last-trade-time"
#last-trade-io-key "last-trade-io"
#initial-time-key "initial-time"
#amount-used-key "amount-used"

#set-last-trade
last-io:,
:set(hash(order-hash() last-trade-time-key) now()),
:set(hash(order-hash() last-trade-io-key) last-io);

#set-initial-time
:set(hash(order-hash() initial-time-key) now());

#get-initial-time
_:get(hash(order-hash() initial-time-key));

#get-last-trade
last-time:get(hash(order-hash() last-trade-time-key)),
last-io:get(hash(order-hash() last-trade-io-key));

#get-epoch
initial-time: call<'get-initial-time>(),
last-time _: call<'get-last-trade>(),
duration: sub(now() any(last-time initial-time)),
total-duration: sub(now() initial-time),
ratio-freeze-amount-epochs: div(min-trade-amount amount-per-epoch),
ratio-freeze-trade-epochs: mul(ratio-freeze-amount-epochs div(time-per-amount-epoch time-per-trade-epoch)),
amount-epochs: div(total-duration time-per-amount-epoch),
trade-epochs: saturating-sub(div(duration time-per-trade-epoch) ratio-freeze-trade-epochs);

#amount-for-epoch
amount-epochs
trade-epochs:,
total-available: linear-growth(0 amount-per-epoch amount-epochs),
used: get(hash(order-hash() amount-used-key)),
unused: sub(total-available used),
decay: call<'halflife>(trade-epochs),
shy-decay: every(greater-than(trade-epochs shy-epoch) decay),
variable-component: sub(max-trade-amount min-trade-amount),
target-amount: add(min-trade-amount mul(variable-component shy-decay)),
capped-unused: min(unused target-amount);

#halflife
epoch:,
/**
 * Shrinking the multiplier like this
 * then applying it 10 times allows for
 * better precision when max-io-ratio
 * is very large, e.g. ~1e10 or ~1e20+
 *
 * This works because \`power\` loses
 * precision on base \`0.5\` when the
 * exponent is large and can even go
 * to \`0\` while the io-ratio is still
 * large. Better to keep the multiplier
 * higher precision and drop the io-ratio
 * smoothly for as long as we can.
 */
multiplier:
  power(0.5 div(epoch 10)),
val:
  mul(
    multiplier
    multiplier
    multiplier
    multiplier
    multiplier
    multiplier
    multiplier
    multiplier
    multiplier
    multiplier
  );

#io-for-epoch
epoch:,
last-io: call<'get-last-trade>(),
max-next-trade: any(mul(last-io next-trade-multiplier) call<'initial-io-fn>()),
baseline-next-trade: mul(last-io next-trade-baseline-multiplier),
real-baseline: max(baseline-next-trade call<'baseline-fn>()),
variable-component: saturating-sub(max-next-trade real-baseline),
above-baseline: mul(variable-component call<'halflife>(epoch)),
_: add(real-baseline above-baseline);

#constant-initial-io
_: initial-io;

#constant-baseline
_: baseline;

#start-price-io-ratio
/**
 * ( usd / output ) / ( usd / input )
 * = ( usd / output ) * ( input / usd )
 * = ( usd * input ) / ( output * usd )
 * = input / output
 */
_: div(start-price-output start-price-input);

#start-price-relative-baseline
start-ratio: call<'start-price-io-ratio>(),
_: mul(baseline-relative-ratio start-ratio);

#start-price-relative-initial
start-ratio: call<'start-price-io-ratio>(),
_: mul(initial-relative-ratio start-ratio);

#sflr-baseline
_: sflr-exchange-rate();

#sflr-baseline-inv
_: inv(sflr-exchange-rate());

#handle-add-order
using-words-from raindex-subparser subparser-0
:call<'set-initial-time>();

#calculate-io
using-words-from raindex-subparser subparser-0
amount-epochs
trade-epochs:call<'get-epoch>(),
max-output: call<'amount-for-epoch>(amount-epochs trade-epochs),
io: call<'io-for-epoch>(trade-epochs),
:call<'set-last-trade>(io);

#handle-io
min-amount: mul(min-trade-amount 0.9),
:ensure(greater-than-or-equal-to(output-vault-decrease() min-amount) "Min trade amount."),
used: get(hash(order-hash() amount-used-key)),
:set(hash(order-hash() amount-used-key) add(used output-vault-decrease()));
`;
