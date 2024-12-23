export type DeploymentStep = {
	label: string;
	type: 'tokens' | 'fields' | 'deposits' | 'vaults';
	fields: any[]; // Changed from 'items' to 'fields'
};

// Example object:
const exampleDeploymentSteps = {
	selectTokens: {
		label: 'Select Tokens',
		type: 'tokens',
		fields: [
			{ token: 'USDC', address: '0x0000000000000000000000000000000000000000' },
			{ token: 'WETH', address: '0x0000000000000000000000000000000000000000' }
		]
	},
	fieldDefinitions: {
		label: 'Field Values',
		type: 'fields',
		fields: [
			{
				name: 'Price',
				binding: 'price',
				presets: [
					{ id: 'preset1', name: 'Market Price' },
					{ id: 'preset2', name: 'Custom Price' }
				]
			},
			{
				name: 'Amount',
				binding: 'amount',
				presets: [
					{ id: 'small', name: 'Small' },
					{ id: 'medium', name: 'Medium' },
					{ id: 'large', name: 'Large' }
				]
			}
		]
	},
	deposits: {
		label: 'Deposits',
		type: 'deposits',
		fields: [
			{
				token_name: 'USDC',
				token: {
					address: '0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48',
					name: 'USD Coin',
					symbol: 'USDC',
					decimals: 6
				},
				presets: ['100', '1000', '10000']
			}
		]
	},
	vaults: {
		label: 'Vault IDs',
		type: 'vaults',
		fields: [
			{
				type: 'input',
				index: 0,
				id: '1',
				tokenInfo: {
					name: 'USD Coin',
					symbol: 'USDC',
					decimals: 6
				}
			},
			{
				type: 'output',
				index: 0,
				id: '2',
				tokenInfo: {
					name: 'Wrapped Ether',
					symbol: 'WETH',
					decimals: 18
				}
			}
		]
	}
};
