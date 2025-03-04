import type { Chain } from "viem/chains";
import * as allChains from "viem/chains"

export const getViemChain = (networkKey: string) => {
	if (networkKey === 'ethereum') {
		// TODO - This isn't ideal, but it's a quick fix to get the chain id for the token list
		return allChains.mainnet;
	}
	const chain: Chain = allChains[networkKey as keyof typeof allChains];
	return chain;
};
