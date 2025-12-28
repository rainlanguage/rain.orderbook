import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { RaindexTrade, RaindexVaultToken } from '@rainlanguage/orderbook';
import {
	extractPairsFromTrades,
	getDefaultPair,
	getTokenLabel,
	transformPairTrades,
	getBucketSecondsForTimeDelta,
	TIME_DELTA_24_HOURS,
	TIME_DELTA_7_DAYS,
	TIME_DELTA_30_DAYS,
	TIME_DELTA_1_YEAR,
	BUCKET_SECONDS_24_HOURS,
	BUCKET_SECONDS_7_DAYS,
	BUCKET_SECONDS_30_DAYS,
	BUCKET_SECONDS_1_YEAR
} from '../lib/services/pairTradesChartData';

function createMockToken(address: string, symbol?: string, name?: string): RaindexVaultToken {
	return {
		id: address,
		address,
		symbol: symbol ?? '',
		name: name ?? '',
		decimals: BigInt(18)
	} as RaindexVaultToken;
}

function createMockTrade(
	id: string,
	timestamp: number,
	inTokenAddress: string,
	outTokenAddress: string,
	inAmount: number,
	outAmount: number,
	inSymbol?: string,
	outSymbol?: string
): RaindexTrade {
	return {
		id,
		timestamp: BigInt(timestamp),
		inputVaultBalanceChange: {
			token: createMockToken(inTokenAddress, inSymbol),
			amount: BigInt(inAmount),
			formattedAmount: String(inAmount),
			vaultId: BigInt(1)
		},
		outputVaultBalanceChange: {
			token: createMockToken(outTokenAddress, outSymbol),
			amount: BigInt(-outAmount),
			formattedAmount: String(-outAmount),
			vaultId: BigInt(2)
		},
		transaction: {
			id: 'tx-' + id,
			from: '0x1234567890abcdef1234567890abcdef12345678',
			timestamp: BigInt(timestamp),
			blockNumber: BigInt(12345)
		},
		orderHash: '0xorderhash',
		orderbook: '0xorderbook'
	} as unknown as RaindexTrade;
}

describe('getBucketSecondsForTimeDelta', () => {
	it('returns correct bucket size for 24h', () => {
		expect(getBucketSecondsForTimeDelta(TIME_DELTA_24_HOURS)).toBe(BUCKET_SECONDS_24_HOURS);
	});

	it('returns correct bucket size for 7d', () => {
		expect(getBucketSecondsForTimeDelta(TIME_DELTA_7_DAYS)).toBe(BUCKET_SECONDS_7_DAYS);
	});

	it('returns correct bucket size for 30d', () => {
		expect(getBucketSecondsForTimeDelta(TIME_DELTA_30_DAYS)).toBe(BUCKET_SECONDS_30_DAYS);
	});

	it('returns correct bucket size for 1y', () => {
		expect(getBucketSecondsForTimeDelta(TIME_DELTA_1_YEAR)).toBe(BUCKET_SECONDS_1_YEAR);
	});
});

describe('extractPairsFromTrades', () => {
	it('returns empty array for no trades', () => {
		const result = extractPairsFromTrades([]);
		expect(result).toEqual([]);
	});

	it('extracts single pair from trades', () => {
		const trades = [
			createMockTrade('1', 1000, '0xAAA', '0xBBB', 100, 200, 'AAA', 'BBB'),
			createMockTrade('2', 2000, '0xAAA', '0xBBB', 50, 100, 'AAA', 'BBB')
		];

		const result = extractPairsFromTrades(trades);
		expect(result).toHaveLength(1);
		expect(result[0].baseToken.address.toLowerCase()).toBe('0xaaa');
		expect(result[0].quoteToken.address.toLowerCase()).toBe('0xbbb');
	});

	it('extracts multiple pairs from trades', () => {
		const trades = [
			createMockTrade('1', 1000, '0xAAA', '0xBBB', 100, 200),
			createMockTrade('2', 2000, '0xCCC', '0xDDD', 50, 100)
		];

		const result = extractPairsFromTrades(trades);
		expect(result).toHaveLength(2);
	});

	it('handles reversed pair order (same pair different direction)', () => {
		const trades = [
			createMockTrade('1', 1000, '0xAAA', '0xBBB', 100, 200),
			createMockTrade('2', 2000, '0xBBB', '0xAAA', 50, 100)
		];

		const result = extractPairsFromTrades(trades);
		expect(result).toHaveLength(1);
	});
});

describe('getDefaultPair', () => {
	it('returns null for empty trades', () => {
		expect(getDefaultPair([])).toBeNull();
	});

	it('returns pair from oldest trade', () => {
		const trades = [
			createMockTrade('1', 2000, '0xAAA', '0xBBB', 100, 200, 'AAA', 'BBB'),
			createMockTrade('2', 1000, '0xCCC', '0xDDD', 50, 100, 'CCC', 'DDD')
		];

		const result = getDefaultPair(trades);
		expect(result).not.toBeNull();
		expect(result!.baseToken.address.toLowerCase()).toBe('0xccc');
		expect(result!.quoteToken.address.toLowerCase()).toBe('0xddd');
	});
});

describe('getTokenLabel', () => {
	it('returns symbol when available', () => {
		const token = createMockToken('0x1234567890123456789012345678901234567890', 'ETH');
		expect(getTokenLabel(token)).toBe('ETH');
	});

	it('returns short address when no symbol', () => {
		const token = createMockToken('0x1234567890123456789012345678901234567890', '');
		expect(getTokenLabel(token)).toBe('0x1234...7890');
	});

	it('returns full address if too short', () => {
		const token = createMockToken('0x1234', '');
		expect(getTokenLabel(token)).toBe('0x1234');
	});
});

describe('transformPairTrades', () => {
	beforeEach(() => {
		vi.useFakeTimers();
		vi.setSystemTime(new Date(1700000000 * 1000));
	});

	it('returns empty data when no trades match the pair', () => {
		const trades = [createMockTrade('1', 1699990000, '0xAAA', '0xBBB', 100, 200)];

		const result = transformPairTrades({
			trades,
			baseTokenAddress: '0xCCC',
			quoteTokenAddress: '0xDDD',
			timeDeltaSeconds: TIME_DELTA_24_HOURS
		});

		expect(result.success).toBe(true);
		if (result.success) {
			expect(result.data.pricePoints).toHaveLength(0);
			expect(result.data.buyVolumePoints).toHaveLength(0);
			expect(result.data.sellVolumePoints).toHaveLength(0);
		}
	});

	it('filters trades outside time window', () => {
		const trades = [
			createMockTrade('1', 1699990000, '0xAAA', '0xBBB', 100, 200),
			createMockTrade('2', 1600000000, '0xAAA', '0xBBB', 50, 100)
		];

		const result = transformPairTrades({
			trades,
			baseTokenAddress: '0xAAA',
			quoteTokenAddress: '0xBBB',
			timeDeltaSeconds: TIME_DELTA_24_HOURS
		});

		expect(result.success).toBe(true);
		if (result.success) {
			expect(result.data.pricePoints).toHaveLength(1);
		}
	});

	it('correctly identifies BUY trades (order received base)', () => {
		const trades = [createMockTrade('1', 1699990000, '0xAAA', '0xBBB', 100, 200)];

		const result = transformPairTrades({
			trades,
			baseTokenAddress: '0xAAA',
			quoteTokenAddress: '0xBBB',
			timeDeltaSeconds: TIME_DELTA_24_HOURS
		});

		expect(result.success).toBe(true);
		if (result.success) {
			expect(result.data.buyVolumePoints).toHaveLength(1);
			expect(result.data.sellVolumePoints).toHaveLength(0);
		}
	});

	it('correctly identifies SELL trades (order gave base)', () => {
		const trades = [createMockTrade('1', 1699990000, '0xBBB', '0xAAA', 200, 100)];

		const result = transformPairTrades({
			trades,
			baseTokenAddress: '0xAAA',
			quoteTokenAddress: '0xBBB',
			timeDeltaSeconds: TIME_DELTA_24_HOURS
		});

		expect(result.success).toBe(true);
		if (result.success) {
			expect(result.data.buyVolumePoints).toHaveLength(0);
			expect(result.data.sellVolumePoints).toHaveLength(1);
		}
	});

	it('calculates price as quote/base', () => {
		const trades = [createMockTrade('1', 1699990000, '0xAAA', '0xBBB', 100, 200)];

		const result = transformPairTrades({
			trades,
			baseTokenAddress: '0xAAA',
			quoteTokenAddress: '0xBBB',
			timeDeltaSeconds: TIME_DELTA_24_HOURS
		});

		expect(result.success).toBe(true);
		if (result.success) {
			expect(result.data.pricePoints).toHaveLength(1);
			expect(result.data.pricePoints[0].value).toBe(2);
		}
	});

	it('merges trades with same timestamp', () => {
		const trades = [
			createMockTrade('1', 1699990000, '0xAAA', '0xBBB', 100, 200),
			createMockTrade('2', 1699990000, '0xAAA', '0xBBB', 100, 200)
		];

		const result = transformPairTrades({
			trades,
			baseTokenAddress: '0xAAA',
			quoteTokenAddress: '0xBBB',
			timeDeltaSeconds: TIME_DELTA_24_HOURS
		});

		expect(result.success).toBe(true);
		if (result.success) {
			expect(result.data.pricePoints).toHaveLength(1);
		}
	});

	it('aggregates volume into buckets', () => {
		const bucketSeconds = BUCKET_SECONDS_24_HOURS;
		const bucketStart = Math.floor(1699990000 / bucketSeconds) * bucketSeconds;

		const trades = [
			createMockTrade('1', bucketStart + 100, '0xAAA', '0xBBB', 100, 200),
			createMockTrade('2', bucketStart + 200, '0xAAA', '0xBBB', 50, 100)
		];

		const result = transformPairTrades({
			trades,
			baseTokenAddress: '0xAAA',
			quoteTokenAddress: '0xBBB',
			timeDeltaSeconds: TIME_DELTA_24_HOURS
		});

		expect(result.success).toBe(true);
		if (result.success) {
			expect(result.data.buyVolumePoints).toHaveLength(1);
			expect(result.data.buyVolumePoints[0].value).toBe(150);
		}
	});

	it('sell volume is negative (bars go down)', () => {
		const trades = [createMockTrade('1', 1699990000, '0xBBB', '0xAAA', 200, 100)];

		const result = transformPairTrades({
			trades,
			baseTokenAddress: '0xAAA',
			quoteTokenAddress: '0xBBB',
			timeDeltaSeconds: TIME_DELTA_24_HOURS
		});

		expect(result.success).toBe(true);
		if (result.success) {
			expect(result.data.sellVolumePoints).toHaveLength(1);
			expect(result.data.sellVolumePoints[0].value).toBeLessThan(0);
		}
	});

	it('sorts price points by time', () => {
		const trades = [
			createMockTrade('1', 1699995000, '0xAAA', '0xBBB', 100, 200),
			createMockTrade('2', 1699990000, '0xAAA', '0xBBB', 50, 100)
		];

		const result = transformPairTrades({
			trades,
			baseTokenAddress: '0xAAA',
			quoteTokenAddress: '0xBBB',
			timeDeltaSeconds: TIME_DELTA_24_HOURS
		});

		expect(result.success).toBe(true);
		if (result.success) {
			expect(result.data.pricePoints[0].time).toBeLessThan(result.data.pricePoints[1].time);
		}
	});
});
