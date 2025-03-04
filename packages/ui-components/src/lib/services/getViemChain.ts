import * as allChains from "viem/chains"

export const getViemChain = (networkKey: string) => {
	const chain = allChains[networkKey as keyof typeof allChains];
	return chain;
};
