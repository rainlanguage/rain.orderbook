import { describe, it } from 'vitest';
import { DotrainOrder } from '../../dist/cjs';
import { assert } from 'chai';

describe('Rain Orderbook Common Package Bindgen Tests', async function () {
	const dotrain = `
networks:
    some-network:
        rpc: http://localhost:8080/rpc-url
        chain-id: 123
        network-id: 123
        currency: ETH

subgraphs:
    some-sg: https://www.some-sg.com

deployers:
    some-deployer:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba

orderbooks:
    some-orderbook:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: some-network
        subgraph: some-sg

tokens:
    token1:
        network: some-network
        address: 0xc2132d05d31c914a87c6611c10748aeb04b58e8f
        decimals: 6
        label: T1
        symbol: T1
    token2:
        network: some-network
        address: 0x8f3cf7ad23cd3cadbd9735aff958023239c6a063
        decimals: 18
        label: T2
        symbol: T2

scenarios:
    some-scenario:
        network: some-network
        deployer: some-deployer
        bindings:
            key: 10

orders:
    some-order:
      inputs:
        - token: token1
          vault-id: 1
      outputs:
        - token: token2
          vault-id: 1
      deployer: some-deployer
      orderbook: some-orderbook

deployments:
    some-deployment:
        scenario: some-scenario
        order: some-order
---
#key !Test binding
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
`;

	it('should compose deployment to rainlang', async () => {
		const dotrainOrder = await DotrainOrder.create(dotrain);
		const result = await dotrainOrder.composeDeploymentToRainlang('some-deployment');
		const expected = `/* 0. calculate-io */ 
_ _: 0 0;

/* 1. handle-io */ 
:;`;

		assert.equal(result, expected);
	});

	it('should compose scenario to rainlang with config', async () => {
		const config = `
scenarios:
    config-scenario:
        network: some-network
        deployer: some-deployer
        bindings:
            key: 10
`;
		const dotrainOrder = await DotrainOrder.create(dotrain, [config]);
		const result = await dotrainOrder.composeScenarioToRainlang('config-scenario');
		const expected = `/* 0. calculate-io */ 
_ _: 0 0;

/* 1. handle-io */ 
:;`;

		assert.equal(result, expected);
	});
});
