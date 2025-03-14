import * as chains from 'viem/chains';

export const getExplorerLink = async (hash: string, chainId: number, type: 'tx' | 'address') => {
	const chain = Object.values(chains).find((chain) => chain.id === chainId);
	if (chain?.blockExplorers) {
		return chain.blockExplorers.default.url + `/${type}/${hash}`;
	} else {
		return '';
	}
};
