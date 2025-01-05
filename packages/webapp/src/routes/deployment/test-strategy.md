
raindex-version: 8898591f3bcaa21dc91dc3b8584330fc405eadfa

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
scenarios:
  arbitrum:
    orderbook: arbitrum
    runs: 1
    bindings:
      raindex-subparser: 0xb06202aA3Fe7d85171fB7aA5f17011d17E63f382
      history-cap: '1e50'
      shy-epoch: 0.05
    scenarios:
      wbtc-weth:
        runs: 1
        bindings:
          amount-token: 0x2f2a2543B76A4166549F7aaB2e75Bef0aefC5B0f
          initial-output-token: 0x82aF49447D8a07e3bd95BD0d56f35241523fBab1
          initial-input-token: 0x2f2a2543B76A4166549F7aaB2e75Bef0aefC5B0f
      usdc-weth:
        runs: 1
        bindings:
          amount-token: 0xaf88d065e77c8cC2239327C5EDb3A432268e5831
          initial-output-token: 0x82aF49447D8a07e3bd95BD0d56f35241523fBab1
          initial-input-token: 0xaf88d065e77c8cC2239327C5EDb3A432268e5831
      weth-umja:
        runs: 1
        bindings:
          amount-token: 0x82aF49447D8a07e3bd95BD0d56f35241523fBab1
          initial-output-token: 0x16A500Aec6c37F84447ef04E66c57cfC6254cF92
          initial-input-token: 0x82aF49447D8a07e3bd95BD0d56f35241523fBab1
      usdt-kima:
        runs: 1
        bindings:
          amount-token: 0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9
          initial-output-token: 0x94fCD9c18f99538C0f7C61c5500cA79F0D5C4dab
          initial-input-token: 0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9
      glo-lusd:
        runs: 1
        bindings:
          amount-token: 0x4F604735c1cF31399C6E711D5962b2B3E0225AD3
          initial-output-token: 0x93b346b6BC2548dA6A1E7d98E9a421B42541425b
          initial-input-token: 0x4F604735c1cF31399C6E711D5962b2B3E0225AD3
  base:
    orderbook: base
    runs: 1
    bindings:
      raindex-subparser: 0x662dFd6d5B6DF94E07A60954901D3001c24F856a
      history-cap: '1e50'
      shy-epoch: 0.05
    scenarios:
      usdc-weth:
        runs: 1
        bindings:
          amount-token: 0x833589fcd6edb6e08f4c7c32d4f71b54bda02913
          initial-output-token: 0x4200000000000000000000000000000000000006
          initial-input-token: 0x833589fcd6edb6e08f4c7c32d4f71b54bda02913
      lucky-weth:
        runs: 1
        bindings:
          amount-token: 0x2C002ffEC41568d138Acc36f5894d6156398D539
          initial-output-token: 0x2C002ffEC41568d138Acc36f5894d6156398D539
          initial-input-token: 0x4200000000000000000000000000000000000006
      usdc-toshi:
        runs: 1
        bindings:
          amount-token: 0x833589fcd6edb6e08f4c7c32d4f71b54bda02913
          initial-output-token: 0xac1bd2486aaf3b5c0fc3fd868558b082a531b2b4
          initial-input-token: 0x833589fcd6edb6e08f4c7c32d4f71b54bda02913
      usdc-paid:
        runs: 1
        bindings:
          amount-token: 0x833589fcd6edb6e08f4c7c32d4f71b54bda02913
          initial-output-token: 0x655A51e6803faF50D4acE80fa501af2F29C856cF
          initial-input-token: 0x833589fcd6edb6e08f4c7c32d4f71b54bda02913
      wlth-usdc:
        runs: 1
        bindings:
          amount-token: 0x833589fcd6edb6e08f4c7c32d4f71b54bda02913
          initial-output-token: 0x99b2B1A2aDB02B38222ADcD057783D7e5D1FCC7D
          initial-input-token: 0x833589fcd6edb6e08f4c7c32d4f71b54bda02913
      usdc-blood:
        runs: 1
        bindings:
          amount-token: 0x833589fcd6edb6e08f4c7c32d4f71b54bda02913
          initial-output-token: 0x3982E57fF1b193Ca8eb03D16Db268Bd4B40818f8
          initial-input-token: 0x833589fcd6edb6e08f4c7c32d4f71b54bda02913

  flare:
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
  polygon:
    orderbook: polygon
    runs: 1
    bindings:
      raindex-subparser: 0xF9323B7d23c655122Fb0272D989b83E105cBcf9d
      history-cap: '1e50'
      shy-epoch: 0.05
    scenarios:
      quick-old-quick:
        runs: 1
        bindings:
          amount-token: 0xB5C064F955D8e7F38fE0460C556a72987494eE17
          initial-output-token: 0x831753dd7087cac61ab5644b308642cc1c33dc13
          initial-input-token: 0xB5C064F955D8e7F38fE0460C556a72987494eE17
      quick-wmatic:
        runs: 1
        bindings:
          amount-token: 0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270
          initial-output-token: 0xB5C064F955D8e7F38fE0460C556a72987494eE17
          initial-input-token: 0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270
      usdc-ioen:
        runs: 1
        bindings:
          amount-token: 0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359
          initial-output-token: 0xd0e9c8f5Fae381459cf07Ec506C1d2896E8b5df6
          initial-input-token: 0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359
      usdc-mnw:
        runs: 1
        bindings:
          amount-token: 0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359
          initial-output-token: 0x3c59798620e5fEC0Ae6dF1A19c6454094572Ab92
          initial-input-token: 0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359
      usdt-poli:
        runs: 1
        bindings:
          amount-token: 0xc2132D05D31c914a87C6611C10748AEb04B58e8F
          initial-output-token: 0x6fb54Ffe60386aC33b722be13d2549dd87BF63AF
          initial-input-token: 0xc2132D05D31c914a87C6611C10748AEb04B58e8F
      weth-mnw:
        runs: 1
        bindings:
          amount-token: 0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619
          initial-output-token: 0x3c59798620e5fEC0Ae6dF1A19c6454094572Ab92
          initial-input-token: 0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619
      usdt-nht:
        runs: 1
        bindings:
          amount-token: 0xc2132D05D31c914a87C6611C10748AEb04B58e8F
          initial-output-token: 0x84342e932797FC62814189f01F0Fb05F52519708
          initial-input-token: 0xc2132D05D31c914a87C6611C10748AEb04B58e8F
      usdce-gfi:
        runs: 1
        bindings:
          amount-token: 0x2791bca1f2de4661ed88a30c99a7a9449aa84174
          initial-output-token: 0x874e178a2f3f3f9d34db862453cd756e7eab0381
          initial-input-token: 0x2791bca1f2de4661ed88a30c99a7a9449aa84174
  bsc:
    orderbook: bsc
    runs: 1
    bindings:
      raindex-subparser: 0x662dFd6d5B6DF94E07A60954901D3001c24F856a
      history-cap: '1e50'
      shy-epoch: 0.05
    scenarios:
      tft-busd:
        runs: 1
        bindings:
          amount-token: 0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56
          initial-output-token: 0x8f0FB159380176D324542b3a7933F0C2Fd0c2bbf
          initial-input-token: 0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56
      tft-usdc:
        runs: 1
        bindings:
          amount-token: 0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d
          initial-output-token: 0x8f0FB159380176D324542b3a7933F0C2Fd0c2bbf
          initial-input-token: 0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d
  ethereum:
    orderbook: ethereum
    runs: 1
    bindings:
      raindex-subparser: 0x22410e2a46261a1B1e3899a072f303022801C764
      history-cap: '1e50'
      shy-epoch: 0.05
    scenarios:
      pai-weth:
        runs: 1
        bindings:
          amount-token: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
          initial-output-token: 0x13E4b8CfFe704d3De6F19E52b201d92c21EC18bD
          initial-input-token: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
      mnw-weth:
        runs: 1
        bindings:
          amount-token: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
          initial-output-token: 0xd3E4Ba569045546D09CF021ECC5dFe42b1d7f6E4
          initial-input-token: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
  linea:
    orderbook: linea
    runs: 1
    bindings:
      raindex-subparser: 0xF77b3c3f61af5a3cE7f7CE3cfFc117491104432E
      history-cap: '1e50'
      shy-epoch: 0.05
    scenarios:
      clip-weth:
        runs: 1
        bindings:
          amount-token: 0xe5D7C2a44FfDDf6b295A15c148167daaAf5Cf34f
          initial-output-token: 0x4Ea77a86d6E70FfE8Bb947FC86D68a7F086f198a
          initial-input-token: 0xe5D7C2a44FfDDf6b295A15c148167daaAf5Cf34f

networks:
  flare:
    rpc: https://rpc.ankr.com/flare
    chain-id: 14
    network-id: 14
    currency: FLR
  base:
    rpc: https://mainnet.base.org
    chain-id: 8453
    network-id: 8453
    currency: ETH
  arbitrum:
    rpc: https://rpc.ankr.com/arbitrum
    chain-id: 42161
    network-id: 42161
    currency: ETH
  polygon:
    rpc: https://rpc.ankr.com/polygon
    chain-id: 137
    network-id: 137
    currency: POL
  bsc:
    rpc: https://rpc.ankr.com/bsc
    chain-id: 56
    network-id: 56
    currency: BNB
  ethereum:
    rpc: https://rpc.ankr.com/eth
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
    network: base
    subgraph: base
  arbitrum:
    address: 0x550878091b2B1506069F61ae59e3A5484Bca9166
    network: arbitrum
    subgraph: arbitrum
  polygon:
    address: 0x7D2f700b1f6FD75734824EA4578960747bdF269A
    network: polygon
    subgraph: polygon
  bsc:
    address: 0xd2938E7c9fe3597F78832CE780Feb61945c377d7
    network: bsc
    subgraph: bsc
  ethereum:
    address: 0x0eA6d458488d1cf51695e1D6e4744e6FB715d37C
    network: ethereum
    subgraph: ethereum
  linea:
    address: 0x22410e2a46261a1B1e3899a072f303022801C764
    network: linea
    subgraph: linea

deployers:
  flare:
    address: 0xE3989Ea7486c0F418C764e6c511e86f6E8830FAb
  base:
    address: 0xC1A14cE2fd58A3A2f99deCb8eDd866204eE07f8D
    network: base
  arbitrum:
    address: 0x9B0D254bd858208074De3d2DaF5af11b3D2F377F
    network: arbitrum
  polygon:
    address: 0xE7116BC05C8afe25e5B54b813A74F916B5D42aB1
    network: polygon
  ethereum:
    address: 0xd19581a021f4704ad4eBfF68258e7A0a9DB1CD77
    network: ethereum
  linea:
    address: 0xA2f56F8F74B7d04d61f281BE6576b6155581dcBA
    network: linea
  bsc:
    address: 0xA2f56F8F74B7d04d61f281BE6576b6155581dcBA
    network: bsc

tokens:
  arbitrum-usdc:
    network: arbitrum
    address: 0xaf88d065e77c8cC2239327C5EDb3A432268e5831
    decimals: 6
  arbitrum-wbtc:
    network: arbitrum
    address: 0x2f2a2543B76A4166549F7aaB2e75Bef0aefC5B0f
    decimals: 8
  arbitrum-weth:
    network: arbitrum
    address: 0x82aF49447D8a07e3bd95BD0d56f35241523fBab1
    decimals: 18
  arbitrum-umja:
    network: arbitrum
    address: 0x16A500Aec6c37F84447ef04E66c57cfC6254cF92
    decimals: 18
  arbitrum-glo:
    network: arbitrum
    address: 0x4F604735c1cF31399C6E711D5962b2B3E0225AD3
    decimals: 18
  arbitrum-lusd:
    network: arbitrum
    address: 0x93b346b6BC2548dA6A1E7d98E9a421B42541425b
    decimals: 18
  arbitrum-usdt:
    network: arbitrum
    address: 0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9
    decimals: 6
  arbitrum-kima:
    network: arbitrum
    address: 0x94fCD9c18f99538C0f7C61c5500cA79F0D5C4dab
    decimals: 18
  base-toshi:
    network: base
    address: 0xac1bd2486aaf3b5c0fc3fd868558b082a531b2b4
    decimals: 18
  base-usdc:
    network: base
    address: 0x833589fcd6edb6e08f4c7c32d4f71b54bda02913
    decimals: 6
  base-weth:
    network: base
    address: 0x4200000000000000000000000000000000000006
    decimals: 18
  base-paid:
    network: base
    address: 0x655A51e6803faF50D4acE80fa501af2F29C856cF
    decimals: 18
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
  polygon-wmatic:
    network: polygon
    address: 0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270
    decimals: 18
  polygon-poli:
    network: polygon
    address: 0x6fb54Ffe60386aC33b722be13d2549dd87BF63AF
    decimals: 18
  polygon-quick-old:
    network: polygon
    address: 0x831753dd7087cac61ab5644b308642cc1c33dc13
    decimals: 18
  polygon-quick:
    network: polygon
    address: 0xB5C064F955D8e7F38fE0460C556a72987494eE17
    decimals: 18
  polygon-ioen:
    network: polygon
    address: 0xd0e9c8f5Fae381459cf07Ec506C1d2896E8b5df6
    decimals: 18
  polygon-mnw:
    network: polygon
    address: 0x3c59798620e5fEC0Ae6dF1A19c6454094572Ab92
    decimals: 18
  polygon-gfi:
    network: polygon
    address: 0x874e178a2f3f3f9d34db862453cd756e7eab0381
    decimals: 18
  polygon-usdc:
    network: polygon
    address: 0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359
    decimals: 6
  polygon-usdt:
    network: polygon
    address: 0xc2132D05D31c914a87C6611C10748AEb04B58e8F
    decimals: 6
  polygon-nht:
    network: polygon
    address: 0x84342e932797FC62814189f01F0Fb05F52519708
    decimals: 18
  polygon-weth:
    network: polygon
    address: 0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619
    decimals: 18
  polygon-usdce:
    network: polygon
    address: 0x2791bca1f2de4661ed88a30c99a7a9449aa84174
    decimals: 6
  bsc-tft:
    network: bsc
    address: 0x8f0FB159380176D324542b3a7933F0C2Fd0c2bbf
    decimals: 7
  bsc-busd:
    network: bsc
    address: 0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56
    decimals: 18
  bsc-usdc:
    network: bsc
    address: 0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d
    decimals: 18
  base-wlth:
    network: base
    address: 0x99b2B1A2aDB02B38222ADcD057783D7e5D1FCC7D
    decimals: 18
  ethereum-pai:
    network: ethereum
    address: 0x13E4b8CfFe704d3De6F19E52b201d92c21EC18bD
    decimals: 18
  ethereum-mnw:
    network: ethereum
    address: 0xd3E4Ba569045546D09CF021ECC5dFe42b1d7f6E4
    decimals: 18
  ethereum-weth:
    network: ethereum
    address: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
    decimals: 18
  linea-clip:
    network: linea
    address: 0x4Ea77a86d6E70FfE8Bb947FC86D68a7F086f198a
    decimals: 18
  linea-weth:
    network: linea
    address: 0xe5D7C2a44FfDDf6b295A15c148167daaAf5Cf34f
    decimals: 18
  base-blood:
    network: base
    address: 0x3982E57fF1b193Ca8eb03D16Db268Bd4B40818f8
    decimals: 18
  base-lucky:
    network: base
    address: 0x2C002ffEC41568d138Acc36f5894d6156398D539
    decimals: 18

orders:
  arbitrum-wbtc-weth:
    network: arbitrum
    orderbook: arbitrum
    inputs:
      - token: arbitrum-wbtc
      - token: arbitrum-weth
    outputs:
      - token: arbitrum-wbtc
      - token: arbitrum-weth
  arbitrum-usdc-weth:
    network: arbitrum
    orderbook: arbitrum
    inputs:
      - token: arbitrum-usdc
      - token: arbitrum-weth
    outputs:
      - token: arbitrum-usdc
      - token: arbitrum-weth
  arbitrum-usdt-kima:
    network: arbitrum
    orderbook: arbitrum
    inputs:
      - token: arbitrum-usdt
      - token: arbitrum-kima
    outputs:
      - token: arbitrum-usdt
      - token: arbitrum-kima
  arbitrum-weth-umja:
    network: arbitrum
    orderbook: arbitrum
    inputs:
      - token: arbitrum-weth
      - token: arbitrum-umja
    outputs:
      - token: arbitrum-weth
      - token: arbitrum-umja
  arbitrum-glo-lusd:
    network: arbitrum
    orderbook: arbitrum
    inputs:
      - token: arbitrum-glo
      - token: arbitrum-lusd
    outputs:
      - token: arbitrum-glo
      - token: arbitrum-lusd
  base-usdc-weth:
    network: base
    orderbook: base
    inputs:
      - token: base-usdc
      - token: base-weth
    outputs:
      - token: base-usdc
      - token: base-weth
  base-lucky-weth:
    network: base
    orderbook: base
    inputs:
      - token: base-lucky
      - token: base-weth
    outputs:
      - token: base-lucky
      - token: base-weth
  base-usdc-toshi:
    network: base
    orderbook: base
    inputs:
      - token: base-usdc
      - token: base-toshi
    outputs:
      - token: base-usdc
      - token: base-toshi
  base-usdc-paid:
    network: base
    orderbook: base
    inputs:
      - token: base-usdc
      - token: base-paid
    outputs:
      - token: base-usdc
      - token: base-paid
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
  polygon-quick-old-quick:
    network: polygon
    orderbook: polygon
    inputs:
      - token: polygon-quick-old
      - token: polygon-quick
    outputs:
      - token: polygon-quick-old
      - token: polygon-quick
  polygon-quick-wmatic:
    network: polygon
    orderbook: polygon
    inputs:
      - token: polygon-quick
      - token: polygon-wmatic
    outputs:
      - token: polygon-quick
      - token: polygon-wmatic
  polygon-usdc-ioen:
    network: polygon
    orderbook: polygon
    inputs:
      - token: polygon-usdc
      - token: polygon-ioen
    outputs:
      - token: polygon-usdc
      - token: polygon-ioen
  polygon-usdc-mnw:
    network: polygon
    orderbook: polygon
    inputs:
      - token: polygon-usdc
      - token: polygon-mnw
    outputs:
      - token: polygon-usdc
      - token: polygon-mnw
  polygon-usdt-poli:
    network: polygon
    orderbook: polygon
    inputs:
      - token: polygon-usdt
      - token: polygon-poli
    outputs:
      - token: polygon-usdt
      - token: polygon-poli
  polygon-weth-mnw:
    network: polygon
    orderbook: polygon
    inputs:
      - token: polygon-weth
      - token: polygon-mnw
    outputs:
      - token: polygon-weth
      - token: polygon-mnw
  polygon-usdt-nht:
    network: polygon
    orderbook: polygon
    inputs:
      - token: polygon-usdt
      - token: polygon-nht
    outputs:
      - token: polygon-usdt
      - token: polygon-nht
  polygon-usdce-gfi:
    network: polygon
    orderbook: polygon
    inputs:
      - token: polygon-usdce
      - token: polygon-gfi
    outputs:
      - token: polygon-usdce
      - token: polygon-gfi
  bsc-tft-busd:
    network: bsc
    orderbook: bsc
    inputs:
      - token: bsc-tft
      - token: bsc-busd
    outputs:
      - token: bsc-tft
      - token: bsc-busd
  bsc-tft-usdc:
    network: bsc
    orderbook: bsc
    inputs:
      - token: bsc-tft
      - token: bsc-usdc
    outputs:
      - token: bsc-tft
      - token: bsc-usdc
  base-wlth-usdc:
    network: base
    orderbook: base
    inputs:
      - token: base-wlth
      - token: base-usdc
    outputs:
      - token: base-wlth
      - token: base-usdc
  ethereum-pai-weth:
    network: ethereum
    orderbook: ethereum
    inputs:
      - token: ethereum-pai
      - token: ethereum-weth
    outputs:
      - token: ethereum-pai
      - token: ethereum-weth
  ethereum-mnw-weth:
    network: ethereum
    orderbook: ethereum
    inputs:
      - token: ethereum-mnw
      - token: ethereum-weth
    outputs:
      - token: ethereum-mnw
      - token: ethereum-weth
  linea-clip-weth:
    network: linea
    orderbook: linea
    inputs:
      - token: linea-clip
      - token: linea-weth
    outputs:
      - token: linea-clip
      - token: linea-weth
  base-usdc-blood:
    network: base
    orderbook: base
    inputs:
      - token: base-usdc
      - token: base-blood
    outputs:
      - token: base-usdc
      - token: base-blood

deployments:
  arbitrum-wbtc-weth:
    order: arbitrum-wbtc-weth
    scenario: arbitrum.wbtc-weth
  arbitrum-usdc-weth:
    order: arbitrum-usdc-weth
    scenario: arbitrum.usdc-weth
  arbitrum-usdt-kima:
    order: arbitrum-usdt-kima
    scenario: arbitrum.usdt-kima
  arbitrum-weth-umja:
    order: arbitrum-weth-umja
    scenario: arbitrum.weth-umja
  arbitrum-glo-lusd:
    order: arbitrum-glo-lusd
    scenario: arbitrum.glo-lusd
  base-usdc-weth:
    order: base-usdc-weth
    scenario: base.usdc-weth
  base-lucky-weth:
    order: base-lucky-weth
    scenario: base.lucky-weth
  base-usdc-toshi:
    order: base-usdc-toshi
    scenario: base.usdc-toshi
  base-usdc-paid:
    order: base-usdc-paid
    scenario: base.usdc-paid
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
  polygon-quick-old-quick:
    order: polygon-quick-old-quick
    scenario: polygon.quick-old-quick
  polygon-quick-wmatic:
    order: polygon-quick-wmatic
    scenario: polygon.quick-wmatic
  polygon-usdc-ioen:
    order: polygon-usdc-ioen
    scenario: polygon.usdc-ioen
  polygon-usdc-mnw:
    order: polygon-usdc-mnw
    scenario: polygon.usdc-mnw
  polygon-usdt-poli:
    order: polygon-usdt-poli
    scenario: polygon.usdt-poli
  polygon-weth-mnw:
    order: polygon-weth-mnw
    scenario: polygon.weth-mnw
  polygon-usdt-nht:
    order: polygon-usdt-nht
    scenario: polygon.usdt-nht
  polygon-usdce-gfi:
    order: polygon-usdce-gfi
    scenario: polygon.usdce-gfi
  bsc-tft-busd:
    order: bsc-tft-busd
    scenario: bsc.tft-busd
  bsc-tft-usdc:
    order: bsc-tft-usdc
    scenario: bsc.tft-usdc
  base-wlth-usdc:
    order: base-wlth-usdc
    scenario: base.wlth-usdc
  ethereum-pai-weth:
    order: ethereum-pai-weth
    scenario: ethereum.pai-weth
  ethereum-mnw-weth:
    order: ethereum-mnw-weth
    scenario: ethereum.mnw-weth
  linea-clip-weth:
    order: linea-clip-weth
    scenario: linea.clip-weth
  base-usdc-blood:
    order: base-usdc-blood
    scenario: base.usdc-blood

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
/**
 * Shrinking the multiplier like this
 * then applying it 10 times allows for
 * better precision when max-io-ratio
 * is very large, e.g. ~1e10 or ~1e20+
 *
 * This works because `power` loses
 * precision on base `0.5` when the
 * exponent is large and can even go
 * to `0` while the io-ratio is still
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
:call<'set-cost-basis-io-ratio>();