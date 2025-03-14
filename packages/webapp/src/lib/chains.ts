import { mainnet, polygon, arbitrum, base, flare, linea, bsc } from 'wagmi/chains';
import type { Chain } from 'wagmi/chains';
export const SupportedChains = {
	mainnet,
	polygon,
	arbitrum,
	base,
	flare,
	linea,
	bsc
} as const;
export const supportedChainsList = [mainnet, polygon, arbitrum, base, flare, linea, bsc] as Chain[];
