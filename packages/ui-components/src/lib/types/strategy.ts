import type { NameAndDescriptionCfg } from '@rainlanguage/orderbook/js_api';

export type StrategyDetail = {
	details: NameAndDescriptionCfg | null;
	name: string;
	dotrain: string;
	error?: unknown;
};
