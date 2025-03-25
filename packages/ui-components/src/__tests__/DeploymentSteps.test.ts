import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import DeploymentSteps from '../lib/components/deployment/DeploymentSteps.svelte';
import { DotrainOrderGui, type ScenarioCfg } from '@rainlanguage/orderbook/js_api';
import type { ComponentProps } from 'svelte';
import { writable } from 'svelte/store';
import type { AppKit } from '@reown/appkit';
import type { ConfigSource, GuiDeploymentCfg } from '@rainlanguage/orderbook/js_api';
import type { DeployModalProps, DisclaimerModalProps } from '../lib/types/modal';
import userEvent from '@testing-library/user-event';
import { useGui } from '$lib/hooks/useGui';

const { mockWagmiConfigStore, mockConnectedStore, mockSignerAddressStore } = await vi.hoisted(
	() => import('../lib/__mocks__/stores')
);

vi.mock('$lib/hooks/useGui', () => ({
	useGui: vi.fn()
}));

export type DeploymentStepsProps = ComponentProps<DeploymentSteps>;

const dotrain = `raindex-version: 8898591f3bcaa21dc91dc3b8584330fc405eadfa

gui:
  name: Two-sided dynamic spread strategies
  description: The dynamic spread strategy for market-making uses time-based adjustments to maintain liquidity by narrowing spreads as market conditions stabilize, while recalculating averages and trade sizes to mitigate risks during trends.
  deployments:
    flare-sflr-wflr:
      name: SFLR<>WFLR on Flare.
      description: Rotate sFLR (Sceptre staked FLR) and WFLR on Flare.
      fields:
        - binding: is-fast-exit
          name: Fast exit?
          description: If enabled, the strategy will attempt to exit any position it builds up in a single trade, as soon as it can do so profitably.
          presets:
            - name: Yes
              value: 1
            - name: No
              value: 0
        - binding: initial-io
          name: Initial price (WFLR per sFLR)
          description: The rough initial WFLR to sFLR ratio (e.g. 1.11).
          min: 1
        - binding: next-trade-multiplier
          name: Next trade multiplier
          description: This is the most the strategy will move the price in a single trade. Larger numbers will capture larger price jumps but trade less often, smaller numbers will trade more often but be less defensive against large price jumps in the market.
          min: 1
          presets:
            - name: 1.001x
              value: 1.001
            - name: 1.002x
              value: 1.002
            - name: 1.005x
              value: 1.005
        - binding: cost-basis-multiplier
          name: Cost basis multiplier
          description: The minimum spread applied to the breakeven in addition to the auction. This is applied in both directions so 1.01x would be a 2% total spread.
          min: 1
          presets:
            - name: 1 (auction spread only)
              value: 1
            - name: 1.0005x (0.1% total)
              value: 1.0005
            - name: 1.001x (0.2% total)
              value: 1.001
            - name: 1.0025x (0.5% total)
              value: 1.0025
            - name: 1.005x (1% total)
              value: 1.005
        - binding: time-per-epoch
          name: Time per halving (seconds)
          description: The amount of time (in seconds) between halvings of the price and the amount offered during each auction, relative to their baselines.
          min: 600
          presets:
            - name: 1 hour (3600)
              value: 3600
            - name: 2 hours (7200)
              value: 7200
            - name: 4 hours (14400)
              value: 14400
            - name: 8 hours (28800)
              value: 28800
        - binding: max-amount
          name: Max amount
          description: The maximum amount of sFLR that will be offered in a single auction.
          min: 0
          presets:
            - name: 100
              value: 100
            - name: 1000
              value: 1000
            - name: 10000
              value: 10000
            - name: 100000
              value: 100000
        - binding: min-amount
          name: Minimum amount
          description: The minimum amount of sFLR that will be offered in a single auction.
          min: 0
          presets:
            - name: 10
              value: 10
            - name: 100
              value: 100
            - name: 1000
              value: 1000

      deposits:
        - token: flare-sflr
          min: 0
          presets:
            - 0
            - 100
            - 1000
            - 10000
        - token: flare-wflr
          min: 0
          presets:
            - 0
            - 100
            - 1000
            - 10000
    flare-cusdx-cysflr:
      name: cUSDX<>cysFLR on Flare.
      description: Rotate cUSDX and cysFLR on Flare.

      fields:
        - binding: is-fast-exit
          name: Fast exit?
          description: If enabled, the strategy will attempt to exit any position it builds up in a single trade, as soon as it can do so profitably.
          presets:
            - name: Yes
              value: 1
            - name: No
              value: 0
        - binding: initial-io
          name: Initial price (cUSDX per cysFLR)
          description: The rough initial cUSDX per cysFLR ratio (e.g. 0.75).
          min: 0
        - binding: next-trade-multiplier
          name: Next trade multiplier
          description: This is the most the strategy will move the price in a single trade. Larger numbers will capture larger price jumps but trade less often, smaller numbers will trade more often but be less defensive against large price jumps in the market.
          min: 1
          presets:
            - name: 1.001x
              value: 1.001
            - name: 1.002x
              value: 1.002
            - name: 1.005x
              value: 1.005
        - binding: cost-basis-multiplier
          name: Cost basis multiplier
          description: The minimum spread applied to the breakeven in addition to the auction. This is applied in both directions so 1.01x would be a 2% total spread.
          min: 1
          presets:
            - name: 1 (auction spread only)
              value: 1
            - name: 1.0005x (0.1% total)
              value: 1.0005
            - name: 1.001x (0.2% total)
              value: 1.001
            - name: 1.0025x (0.5% total)
              value: 1.0025
            - name: 1.005x (1% total)
              value: 1.005
        - binding: time-per-epoch
          name: Time per halving (seconds)
          description: The amount of time (in seconds) between halvings of the price and the amount offered during each auction, relative to their baselines.
          min: 600
          presets:
            - name: 1 hour (3600)
              value: 3600
            - name: 2 hours (7200)
              value: 7200
            - name: 4 hours (14400)
              value: 14400
            - name: 8 hours (28800)
              value: 28800
        - binding: max-amount
          name: Max amount
          description: The maximum amount of cUSDX that will be offered in a single auction.
          min: 0
          presets:
            - name: 10
              value: 10
            - name: 100
              value: 100
            - name: 1000
              value: 1000
        - binding: min-amount
          name: Minimum amount
          description: The minimum amount of cUSDX that will be offered in a single auction.
          min: 0
          presets:
            - name: 10
              value: 10
            - name: 100
              value: 100
            - name: 1000
              value: 1000

      deposits:
        - token: flare-cysflr
          min: 0
          presets:
            - 0
            - 100
            - 200
            - 500
        - token: flare-cusdx
          min: 0
          presets:
            - 0
            - 100
            - 200
            - 500

networks:
  flare:
    rpc: https://rpc.ankr.com/flare
    chain-id: 14
    network-id: 14
    currency: FLR

scenarios:
  flare:
    deployer: flare
    orderbook: flare
    runs: 1
    bindings:
      raindex-subparser: 0xFe2411CDa193D9E4e83A5c234C7Fd320101883aC
      history-cap: '1e50'
      shy-epoch: 0.05
    scenarios:
      sflr-wflr:
        runs: 1
        bindings:
          amount-token: 0x12e605bc104e93B45e1aD99F9e555f659051c2BB
          initial-output-token: 0x12e605bc104e93B45e1aD99F9e555f659051c2BB
          initial-input-token: 0x1D80c49BbBCd1C0911346656B529DF9E5c2F783d
      cusdx-cysflr:
        runs: 1
        bindings:
          amount-token: 0xFE2907DFa8DB6e320cDbF45f0aa888F6135ec4f8
          initial-output-token: 0x19831cfB53A0dbeAD9866C43557C1D48DfF76567
          initial-input-token: 0xFE2907DFa8DB6e320cDbF45f0aa888F6135ec4f8
      sflr-joule:
        runs: 1
        bindings:
          amount-token: 0x12e605bc104e93B45e1aD99F9e555f659051c2BB
          initial-output-token: 0xE6505f92583103AF7ed9974DEC451A7Af4e3A3bE
          initial-input-token: 0x12e605bc104e93B45e1aD99F9e555f659051c2BB
      wflr-eusdt:
        runs: 1
        bindings:
          amount-token: 0xfbda5f676cb37624f28265a144a48b0d6e87d3b6
          initial-output-token: 0x12e605bc104e93B45e1aD99F9e555f659051c2BB
          initial-input-token: 0xfbda5f676cb37624f28265a144a48b0d6e87d3b6
      usdce-sflr:
        runs: 1
        bindings:
          amount-token: 0xfbda5f676cb37624f28265a144a48b0d6e87d3b6
          initial-output-token: 0x12e605bc104e93B45e1aD99F9e555f659051c2BB
          initial-input-token: 0xfbda5f676cb37624f28265a144a48b0d6e87d3b6
      usdce-cusdx:
        runs: 1
        bindings:
          amount-token: 0xfbda5f676cb37624f28265a144a48b0d6e87d3b6
          initial-output-token: 0xFE2907DFa8DB6e320cDbF45f0aa888F6135ec4f8
          initial-input-token: 0xfbda5f676cb37624f28265a144a48b0d6e87d3b6
      usdce-wflr:
        runs: 1
        bindings:
          amount-token: 0xfbda5f676cb37624f28265a144a48b0d6e87d3b6
          initial-output-token: 0x1D80c49BbBCd1C0911346656B529DF9E5c2F783d
          initial-input-token: 0xfbda5f676cb37624f28265a144a48b0d6e87d3b6
      usdce-weth:
        runs: 1
        bindings:
          amount-token: 0xfbda5f676cb37624f28265a144a48b0d6e87d3b6
          initial-output-token: 0x1502FA4be69d526124D453619276FacCab275d3D
          initial-input-token: 0xfbda5f676cb37624f28265a144a48b0d6e87d3b6

metaboards:
  flare: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-flare-0x893BBFB7/0.1/gn

subgraphs:
  flare: https://example.com/subgraph

orderbooks:
  flare:
    id: 'flare',
    address: '0x0'

deployers:
  flare:
    address: 0xE3989Ea7486c0F418C764e6c511e86f6E8830FAb

tokens:
  flare-usdce:
    network: flare
    address: 0xfbda5f676cb37624f28265a144a48b0d6e87d3b6
    decimals: 6
  flare-wflr:
    network: flare
    address: 0x1D80c49BbBCd1C0911346656B529DF9E5c2F783d
    decimals: 18
  flare-sflr:
    network: flare
    address: 0x12e605bc104e93B45e1aD99F9e555f659051c2BB
    decimals: 18
  flare-weth:
    network: flare
    address: 0x1502FA4be69d526124D453619276FacCab275d3D
    decimals: 18
  flare-cysflr:
    network: flare
    address: 0x19831cfB53A0dbeAD9866C43557C1D48DfF76567
  flare-cusdx:
    network: flare
    address: 0xFE2907DFa8DB6e320cDbF45f0aa888F6135ec4f8
  flare-joule:
    network: flare
    address: 0xE6505f92583103AF7ed9974DEC451A7Af4e3A3bE
    decimals: 18

orders:
  flare-usdce-sflr:
    network: flare
    orderbook: flare
    inputs:
      - token: flare-sflr
      - token: flare-usdce
    outputs:
      - token: flare-sflr
      - token: flare-usdce
  flare-sflr-wflr:
    network: flare
    orderbook: flare
    inputs:
      - token: flare-sflr
      - token: flare-wflr
    outputs:
      - token: flare-sflr
      - token: flare-wflr
  flare-cusdx-cysflr:
    network: flare
    orderbook: flare
    inputs:
      - token: flare-cusdx
      - token: flare-cysflr
    outputs:
      - token: flare-cusdx
      - token: flare-cysflr
  flare-sflr-joule:
    network: flare
    orderbook: flare
    inputs:
      - token: flare-sflr
      - token: flare-joule
    outputs:
      - token: flare-sflr
      - token: flare-joule
  flare-usdce-weth:
    network: flare
    orderbook: flare
    inputs:
      - token: flare-usdce
      - token: flare-weth
    outputs:
      - token: flare-usdce
      - token: flare-weth
  flare-usdce-wflr:
    network: flare
    orderbook: flare
    inputs:
      - token: flare-usdce
      - token: flare-wflr
    outputs:
      - token: flare-usdce
      - token: flare-wflr
  flare-usdce-cusdx:
    network: flare
    orderbook: flare
    inputs:
      - token: flare-usdce
      - token: flare-cusdx
    outputs:
      - token: flare-usdce
      - token: flare-cusdx

deployments:
  flare-usdce-sflr:
    order: flare-usdce-sflr
    scenario: flare.usdce-sflr
  flare-sflr-wflr:
    order: flare-sflr-wflr
    scenario: flare.sflr-wflr
  flare-cusdx-cysflr:
    order: flare-cusdx-cysflr
    scenario: flare.cusdx-cysflr
  flare-usdce-weth:
    order: flare-usdce-weth
    scenario: flare.usdce-weth
  flare-usdce-wflr:
    order: flare-usdce-wflr
    scenario: flare.usdce-wflr
  flare-usdce-cusdx:
    order: flare-usdce-cusdx
    scenario: flare.usdce-cusdx
  flare-sflr-joule:
    order: flare-sflr-joule
    scenario: flare.sflr-joule

---

#raindex-subparser !Subparser for the Raindex.

#min-amount !Amount will decay down to this number each epoch.
#max-amount !Amount will decay down from this number each epoch.
#time-per-epoch !Duration of one unit of streaming amount and io ratio halflife.
#shy-epoch !Epoch below which only the minimum amount is offered.
#next-trade-multiplier !Start next auction at this x the last trade.
#history-cap !The max amount of trade history kept for cost basis tracking (denominated in same token as tranche size).
#amount-token !The token that is the amount token for the strategy. This denominates tranche sizes.
#initial-io !The IO ratio that the strategy starts at. The quote token is the output so that the IO ratio looks like a CEX price.
#initial-output-token !Initial output token for the initial IO ratio.
#initial-input-token !Initial input token for the initial IO ratio.
#cost-basis-multiplier !Multiplier for the cost basis IO ratio. Effectively a minimum spread.

#is-fast-exit !Non-zero for fast exit behaviour.

#last-trade-io-key "last-trade-io"
#last-trade-time-key "last-trade-time"
#last-trade-output-token-key "last-trade-output-token"
#vwaio-key "cost-basis-io-ratio"

#amount-is-output
  _: equal-to(amount-token output-token());

#get-cost-basis-io-ratio
  this-total-out-key: hash(order-hash() input-token() output-token()),
  this-vwaio-key: hash(this-total-out-key vwaio-key),
  other-total-out-key: hash(order-hash() output-token() input-token()),
  other-vwaio-key: hash(other-total-out-key vwaio-key),

  this-total-out: get(this-total-out-key),
  other-total-out: get(other-total-out-key),

  this-vwaio: get(this-vwaio-key),
  other-vwaio: get(other-vwaio-key);

#set-cost-basis-io-ratio
  /* first reduce outstanding inventory */
  this-total-out-key
  this-vwaio-key
  other-total-out-key
  other-vwaio-key
  this-total-out
  other-total-out
  this-vwaio
  other-vwaio: call<'get-cost-basis-io-ratio>(),

  other-reduction-out: min(other-total-out input-vault-increase()),
  reduced-other-total-out: sub(other-total-out other-reduction-out),

  :set(other-total-out-key reduced-other-total-out),
  :set(other-vwaio-key every(reduced-other-total-out other-vwaio)),

  /* then increase our inventory */
  this-total-in: mul(this-total-out this-vwaio),
  this-remaining-in: sub(input-vault-increase() other-reduction-out),
  this-new-in: add(this-total-in this-remaining-in),
  this-remaining-out: div(this-remaining-in calculated-io-ratio()),
  this-new-out: add(this-total-out this-remaining-out),
  this-new-vwaio: every(this-new-out div(this-new-in any(this-new-out max-value()))),
  cap-out: if(call<'amount-is-output>() history-cap div(history-cap any(this-new-vwaio calculated-io-ratio()))),
  capped-out: min(this-new-out cap-out),

  :set(this-total-out-key capped-out),
  :set(this-vwaio-key this-new-vwaio);

#halflife
epoch:,
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
  );

#set-last-trade
last-io:,
:set(hash(order-hash() last-trade-time-key) now()),
:set(hash(order-hash() last-trade-io-key) last-io),
:set(hash(order-hash() last-trade-output-token-key) output-token());

#handle-add-order
using-words-from raindex-subparser
:set(hash(order-hash() last-trade-time-key) now()),
:set(hash(order-hash() last-trade-io-key) initial-io),
:set(hash(order-hash() last-trade-output-token-key) initial-output-token),
this-total-out-key: hash(order-hash() initial-input-token initial-output-token),
:set(this-total-out-key 1e-18),
:set(hash(this-total-out-key vwaio-key) initial-io);

#get-last-trade
stored-last-io:get(hash(order-hash() last-trade-io-key)),
stored-last-output-token:get(hash(order-hash() last-trade-output-token-key)),
last-time:get(hash(order-hash() last-trade-time-key)),
_: if(equal-to(stored-last-output-token output-token()) stored-last-io inv(stored-last-io));

#get-epoch
last-time _: call<'get-last-trade>(),
duration: sub(now() last-time),
epochs: div(duration time-per-epoch);

#amount-for-epoch
epoch io:,
decay: call<'halflife>(epoch),
shy-decay: every(greater-than(epoch shy-epoch) decay),
variable-component: sub(max-amount min-amount),
base-amount: add(min-amount mul(variable-component shy-decay)),
_: if(call<'amount-is-output>() base-amount mul(base-amount inv(io)));

#io-for-epoch
epoch:,
last-io: call<'get-last-trade>(),
this-vwaio
other-vwaio: call<'get-cost-basis-io-ratio>(),
cost-basis-io: mul(any(this-vwaio inv(any(other-vwaio max-value()))) cost-basis-multiplier),
max-next-trade: mul(max(cost-basis-io last-io) next-trade-multiplier),
baseline: any(cost-basis-io last-io),
variable-component: sub(max-next-trade baseline),
decay: call<'halflife>(epoch),
above-baseline: mul(variable-component decay),
_: add(baseline above-baseline);

#calculate-io
using-words-from raindex-subparser
epoch:call<'get-epoch>(),
io: call<'io-for-epoch>(epoch),
epoch-max-output: call<'amount-for-epoch>(epoch io),
other-total-out
_
other-vwaio: call<'get-cost-basis-io-ratio>(),
max-output: max(
  epoch-max-output
  every(
    is-fast-exit
    mul(other-total-out other-vwaio))),
_: io,
:call<'set-last-trade>(io);

#handle-io
min-trade-amount: mul(min-amount 0.9),
:ensure(
  greater-than-or-equal-to(
    if(call<'amount-is-output>() output-vault-decrease() input-vault-increase())
    min-trade-amount)
  "Min trade amount."),
:call<'set-cost-basis-io-ratio>();`;

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
	dotrain,
	strategyDetail: {
		name: 'SFLR<>WFLR on Flare',
		description: 'Rotate sFLR (Sceptre staked FLR) and WFLR on Flare.',
		short_description: 'Rotate sFLR (Sceptre staked FLR) and WFLR on Flare.'
	},
	deployment: mockDeployment,
	wagmiConfig: mockWagmiConfigStore,
	wagmiConnected: mockConnectedStore,
	signerAddress: mockSignerAddressStore,
	appKitModal: writable({} as AppKit),
	handleDeployModal: vi.fn() as unknown as (args: DeployModalProps) => void,
	handleDisclaimerModal: vi.fn() as unknown as (args: DisclaimerModalProps) => void,
	settings: writable({} as ConfigSource)
};

describe('DeploymentSteps', () => {
	let guiInstance: DotrainOrderGui;
	let mockGui: DotrainOrderGui;

	beforeEach(() => {
		vi.clearAllMocks();
		guiInstance = new DotrainOrderGui();

		(DotrainOrderGui.prototype.areAllTokensSelected as Mock).mockReturnValue(false);
		(DotrainOrderGui.prototype.getSelectTokens as Mock).mockReturnValue([]);
		(DotrainOrderGui.prototype.getNetworkKey as Mock).mockReturnValue('flare');
		(DotrainOrderGui.prototype.getCurrentDeployment as Mock).mockReturnValue(mockDeployment);
		(DotrainOrderGui.prototype.getAllFieldDefinitions as Mock).mockReturnValue({ value: [] });
		(DotrainOrderGui.prototype.hasAnyDeposit as Mock).mockReturnValue({ value: false });
		(DotrainOrderGui.prototype.hasAnyVaultId as Mock).mockReturnValue(false);
		(DotrainOrderGui.prototype.getAllTokenInfos as Mock).mockResolvedValue([]);
		(DotrainOrderGui.prototype.getCurrentDeploymentDetails as Mock).mockReturnValue({
			value: {
				name: 'Test Deployment',
				description: 'This is a test deployment description'
			}
		});
		mockGui = guiInstance;
		vi.mocked(useGui).mockReturnValue(mockGui);
	});

	it('shows deployment details when provided', async () => {
		render(DeploymentSteps, { props: defaultProps });

		await waitFor(() => {
			expect(screen.getByText('SFLR<>WFLR on Flare')).toBeInTheDocument();
		});
	});

	it('shows select tokens section when tokens need to be selected', async () => {
		(DotrainOrderGui.prototype.getSelectTokens as Mock).mockReturnValue(['token1', 'token2']);

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

	it('shows deploy strategy or wallet connect button when all required fields are filled', async () => {
		const mockSelectTokens = [
			{ key: 'token1', name: 'Token 1', description: undefined },
			{ key: 'token2', name: 'Token 2', description: undefined }
		];

		// Set up specific mocks for this test
		(DotrainOrderGui.prototype.getSelectTokens as Mock).mockReturnValue(mockSelectTokens);
		(DotrainOrderGui.prototype.getTokenInfo as Mock).mockImplementation(() => {});
		(DotrainOrderGui.prototype.areAllTokensSelected as Mock).mockReturnValue(true);
		(DotrainOrderGui.prototype.isSelectTokenSet as Mock).mockReturnValue(false);
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

		(DotrainOrderGui.prototype.getAllTokenInfos as Mock).mockResolvedValue([
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
		mockConnectedStore.mockSetSubscribeValue(true);

		render(DeploymentSteps, { props: defaultProps });

		await waitFor(() => {
			expect(screen.getByText('Deploy Strategy')).toBeInTheDocument();
		});
		mockConnectedStore.mockSetSubscribeValue(false);

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
		(DotrainOrderGui.prototype.getSelectTokens as Mock).mockReturnValue(mockSelectTokens);
		(DotrainOrderGui.prototype.getTokenInfo as Mock).mockImplementation(() => {});
		(DotrainOrderGui.prototype.areAllTokensSelected as Mock).mockReturnValue(true);
		(DotrainOrderGui.prototype.isSelectTokenSet as Mock).mockReturnValue(false);
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

		(DotrainOrderGui.prototype.getAllTokenInfos as Mock).mockResolvedValue([
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
		(DotrainOrderGui.prototype.getTokenInfo as Mock).mockResolvedValue({
			address: '0x1',
			decimals: 18,
			name: 'Token 1',
			symbol: 'TKN1'
		});
		await user.type(selectTokenInput, '0x1');

		const selectTokenOutput = screen.getAllByRole('textbox')[1];
		(DotrainOrderGui.prototype.getTokenInfo as Mock).mockResolvedValue({
			address: '0x2',
			decimals: 18,
			name: 'Token 2',
			symbol: 'TKN2'
		});
		await user.type(selectTokenOutput, '0x2');

		await waitFor(() => {
			expect(mockGui.getAllTokenInfos).toHaveBeenCalled();
		});

		selectTokenInput = screen.getAllByRole('textbox')[0];
		(DotrainOrderGui.prototype.getTokenInfo as Mock).mockResolvedValue({
			address: '0x3',
			decimals: 18,
			name: 'Token 3',
			symbol: 'TKN3'
		});
		await user.type(selectTokenInput, '0x3');

		(DotrainOrderGui.prototype.getAllTokenInfos as Mock).mockResolvedValue([
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
		});
	});
});
