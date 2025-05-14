import type { SgTrade } from '@rainlanguage/orderbook';
import type { UTCTimestamp } from 'lightweight-charts';
import { timestampSecondsToUTCTimestamp } from '../services/time';
import { sortBy } from 'lodash';
import { formatUnits } from 'viem';

export type HistoricalOrderChartData = { value: number; time: UTCTimestamp; color?: string }[];

export function prepareHistoricalOrderChartData(takeOrderEntities: SgTrade[], colorTheme: string) {
	const transformedData = takeOrderEntities.map((d) => ({
		value: Math.abs(
			Number(
				formatUnits(
					BigInt(d.inputVaultBalanceChange.amount),
					Number(d.inputVaultBalanceChange.vault.token.decimals ?? 0)
				)
			) /
				Number(
					formatUnits(
						BigInt(d.outputVaultBalanceChange.amount),
						Number(d.outputVaultBalanceChange.vault.token.decimals ?? 0)
					)
				)
		),
		time: timestampSecondsToUTCTimestamp(BigInt(d.timestamp)),
		color: colorTheme == 'dark' ? '#5178FF' : '#4E4AF6',
		outputAmount: +d.outputVaultBalanceChange.amount
	}));

	// if we have multiple object in the array with the same timestamp, we need to merge them
	// we do this by taking the weighted average of the ioratio values for objects that share the same timestamp.
	const uniqueTimestamps = Array.from(new Set(transformedData.map((d) => d.time)));
	const finalData: HistoricalOrderChartData = [];
	uniqueTimestamps.forEach((timestamp) => {
		const objectsWithSameTimestamp = transformedData.filter((d) => d.time === timestamp);
		if (objectsWithSameTimestamp.length > 1) {
			// calculate a weighted average of the ioratio values using the amount of the output token as the weight
			const ioratioSum = objectsWithSameTimestamp.reduce(
				(acc, d) => acc + d.value * d.outputAmount,
				0
			);
			const outputAmountSum = objectsWithSameTimestamp.reduce((acc, d) => acc + d.outputAmount, 0);
			const ioratioAverage = ioratioSum / outputAmountSum;
			finalData.push({
				value: ioratioAverage,
				time: timestamp,
				color: objectsWithSameTimestamp[0].color
			});
		} else {
			finalData.push(objectsWithSameTimestamp[0]);
		}
	});

	return sortBy(finalData, (d) => d.time);
}

if (import.meta.vitest) {
	const { it, expect } = import.meta.vitest;

	it('transforms and sorts data as expected', () => {
		const takeOrderEntities: SgTrade[] = [
			{
				id: '1',
				timestamp: '1632000000',
				tradeEvent: {
					sender: 'sender_address',
					transaction: {
						id: 'transaction_id',
						from: 'sender_address',
						timestamp: '1632000000',
						blockNumber: '0'
					}
				},
				outputVaultBalanceChange: {
					amount: '100',
					vault: {
						id: '1',
						vaultId: 'vault-id1',
						token: {
							id: 'output_token',
							address: 'output_token',
							name: 'output_token',
							symbol: 'output_token',
							decimals: '1'
						}
					},
					id: '1',
					__typename: 'Withdraw',
					newVaultBalance: '0',
					oldVaultBalance: '0',
					timestamp: '0',
					transaction: {
						id: 'transaction_id',
						from: 'sender_address',
						timestamp: '1632000000',
						blockNumber: '0'
					},
					orderbook: { id: '1' }
				},
				order: {
					id: 'order_id',
					orderHash: 'orderHash'
				},
				inputVaultBalanceChange: {
					vault: {
						id: '1',
						vaultId: 'vault-id1',
						token: {
							id: 'output_token',
							address: 'output_token',
							name: 'output_token',
							symbol: 'output_token',
							decimals: '1'
						}
					},
					amount: '50',
					id: '1',
					__typename: 'Withdraw',
					newVaultBalance: '0',
					oldVaultBalance: '0',
					timestamp: '0',
					transaction: {
						id: 'transaction_id',
						from: 'sender_address',
						timestamp: '1632000000',
						blockNumber: '0'
					},
					orderbook: { id: '1' }
				},
				orderbook: {
					id: '0x00'
				}
			},
			{
				id: '2',
				timestamp: '1631000000',
				tradeEvent: {
					sender: 'sender_address',
					transaction: {
						id: 'transaction_id',
						from: 'sender_address',
						timestamp: '1631000000',
						blockNumber: '0'
					}
				},
				outputVaultBalanceChange: {
					amount: '100',
					vault: {
						id: '1',
						vaultId: 'vault-id1',
						token: {
							id: 'output_token',
							address: 'output_token',
							name: 'output_token',
							symbol: 'output_token',
							decimals: '1'
						}
					},
					id: '1',
					__typename: 'Withdraw',
					newVaultBalance: '0',
					oldVaultBalance: '0',
					timestamp: '0',
					transaction: {
						id: 'transaction_id',
						from: 'sender_address',
						timestamp: '1632000000',
						blockNumber: '0'
					},
					orderbook: { id: '1' }
				},
				order: {
					id: 'order_id',
					orderHash: 'orderHash'
				},
				inputVaultBalanceChange: {
					vault: {
						id: '1',
						vaultId: 'vault-id1',
						token: {
							id: 'output_token',
							address: 'output_token',
							name: 'output_token',
							symbol: 'output_token',
							decimals: '1'
						}
					},
					amount: '50',
					id: '1',
					__typename: 'Withdraw',
					newVaultBalance: '0',
					oldVaultBalance: '0',
					timestamp: '0',
					transaction: {
						id: 'transaction_id',
						from: 'sender_address',
						timestamp: '1632000000',
						blockNumber: '0'
					},
					orderbook: { id: '1' }
				},
				orderbook: {
					id: '0x00'
				}
			},
			{
				id: '3',
				timestamp: '1630000000',
				tradeEvent: {
					sender: 'sender_address',
					transaction: {
						id: 'transaction_id',
						from: 'sender_address',
						timestamp: '1630000000',
						blockNumber: '0'
					}
				},
				outputVaultBalanceChange: {
					amount: '100',
					vault: {
						id: '1',
						vaultId: 'vault-id1',
						token: {
							id: 'output_token',
							address: 'output_token',
							name: 'output_token',
							symbol: 'output_token',
							decimals: '1'
						}
					},
					id: '1',
					__typename: 'Withdraw',
					newVaultBalance: '0',
					oldVaultBalance: '0',
					timestamp: '0',
					transaction: {
						id: 'transaction_id',
						from: 'sender_address',
						timestamp: '1632000000',
						blockNumber: '0'
					},
					orderbook: { id: '1' }
				},
				order: {
					id: 'order_id',
					orderHash: 'orderHash'
				},
				inputVaultBalanceChange: {
					vault: {
						id: '1',
						vaultId: 'vault-id1',
						token: {
							id: 'output_token',
							address: 'output_token',
							name: 'output_token',
							symbol: 'output_token',
							decimals: '1'
						}
					},
					amount: '50',
					id: '1',
					__typename: 'Withdraw',
					newVaultBalance: '0',
					oldVaultBalance: '0',
					timestamp: '0',
					transaction: {
						id: 'transaction_id',
						from: 'sender_address',
						timestamp: '1632000000',
						blockNumber: '0'
					},
					orderbook: { id: '1' }
				},
				orderbook: {
					id: '0x00'
				}
			}
		];

		const result = prepareHistoricalOrderChartData(takeOrderEntities, 'dark');

		expect(result.length).toEqual(3);
		expect(result[0].value).toEqual(0.5);
		expect(result[0].time).toEqual(1630000000);
		expect(result[1].value).toEqual(0.5);
		expect(result[1].time).toEqual(1631000000);
		expect(result[2].value).toEqual(0.5);
		expect(result[2].time).toEqual(1632000000);

		// check the color
		expect(result[0].color).toEqual('#5178FF');
		expect(result[1].color).toEqual('#5178FF');
		expect(result[2].color).toEqual('#5178FF');
	});

	it('handles the case where multiple trades have the same timestamp', () => {
		const takeOrderEntities: SgTrade[] = [
			{
				id: '1',
				timestamp: '1632000000',
				tradeEvent: {
					sender: 'sender_address',
					transaction: {
						id: 'transaction_id',
						from: 'sender_address',
						timestamp: '1632000000',
						blockNumber: '0'
					}
				},
				outputVaultBalanceChange: {
					amount: '100',
					vault: {
						id: '1',
						vaultId: 'vault-id1',
						token: {
							id: 'output_token',
							address: 'output_token',
							name: 'output_token',
							symbol: 'output_token',
							decimals: '1'
						}
					},
					id: '1',
					__typename: 'Withdraw',
					newVaultBalance: '0',
					oldVaultBalance: '0',
					timestamp: '0',
					transaction: {
						id: 'transaction_id',
						from: 'sender_address',
						timestamp: '1632000000',
						blockNumber: '0'
					},
					orderbook: { id: '1' }
				},
				order: {
					id: 'order_id',
					orderHash: 'orderHash'
				},
				inputVaultBalanceChange: {
					vault: {
						id: '1',
						vaultId: 'vault-id1',
						token: {
							id: 'output_token',
							address: 'output_token',
							name: 'output_token',
							symbol: 'output_token',
							decimals: '1'
						}
					},
					amount: '50',
					id: '1',
					__typename: 'Withdraw',
					newVaultBalance: '0',
					oldVaultBalance: '0',
					timestamp: '0',
					transaction: {
						id: 'transaction_id',
						from: 'sender_address',
						timestamp: '1632000000',
						blockNumber: '0'
					},
					orderbook: { id: '1' }
				},
				orderbook: {
					id: '0x00'
				}
			},
			{
				id: '2',
				timestamp: '1632000000',
				tradeEvent: {
					sender: 'sender_address',
					transaction: {
						id: 'transaction_id',
						from: 'sender_address',
						timestamp: '1632000000',
						blockNumber: '0'
					}
				},
				outputVaultBalanceChange: {
					amount: '200',
					vault: {
						id: '1',
						vaultId: 'vault-id1',
						token: {
							id: 'output_token',
							address: 'output_token',
							name: 'output_token',
							symbol: 'output_token',
							decimals: '1'
						}
					},
					id: '1',
					__typename: 'Withdraw',
					newVaultBalance: '0',
					oldVaultBalance: '0',
					timestamp: '0',
					transaction: {
						id: 'transaction_id',
						from: 'sender_address',
						timestamp: '1632000000',
						blockNumber: '0'
					},
					orderbook: { id: '1' }
				},
				order: {
					id: 'order_id',
					orderHash: 'orderHash'
				},
				inputVaultBalanceChange: {
					vault: {
						id: '1',
						vaultId: 'vault-id1',
						token: {
							id: 'output_token',
							address: 'output_token',
							name: 'output_token',
							symbol: 'output_token',
							decimals: '1'
						}
					},
					amount: '50',
					id: '1',
					__typename: 'Withdraw',
					newVaultBalance: '0',
					oldVaultBalance: '0',
					timestamp: '0',
					transaction: {
						id: 'transaction_id',
						from: 'sender_address',
						timestamp: '1632000000',
						blockNumber: '0'
					},
					orderbook: { id: '1' }
				},
				orderbook: {
					id: '0x00'
				}
			},
			{
				id: '3',
				timestamp: '1632000000',
				tradeEvent: {
					sender: 'sender_address',
					transaction: {
						id: 'transaction_id',
						from: 'sender_address',
						timestamp: '1632000000',
						blockNumber: '0'
					}
				},
				outputVaultBalanceChange: {
					amount: '400',
					vault: {
						id: '1',
						vaultId: 'vault-id1',
						token: {
							id: 'output_token',
							address: 'output_token',
							name: 'output_token',
							symbol: 'output_token',
							decimals: '1'
						}
					},
					id: '1',
					__typename: 'Withdraw',
					newVaultBalance: '0',
					oldVaultBalance: '0',
					timestamp: '0',
					transaction: {
						id: 'transaction_id',
						from: 'sender_address',
						timestamp: '1632000000',
						blockNumber: '0'
					},
					orderbook: { id: '1' }
				},
				order: {
					id: 'order_id',
					orderHash: 'orderHash'
				},
				inputVaultBalanceChange: {
					vault: {
						id: '1',
						vaultId: 'vault-id1',
						token: {
							id: 'output_token',
							address: 'output_token',
							name: 'output_token',
							symbol: 'output_token',
							decimals: '1'
						}
					},
					amount: '50',
					id: '1',
					__typename: 'Withdraw',
					newVaultBalance: '0',
					oldVaultBalance: '0',
					timestamp: '0',
					transaction: {
						id: 'transaction_id',
						from: 'sender_address',
						timestamp: '1632000000',
						blockNumber: '0'
					},
					orderbook: { id: '1' }
				},
				orderbook: {
					id: '0x00'
				}
			}
		];

		const result = prepareHistoricalOrderChartData(takeOrderEntities, 'dark');

		// calculate the weighted average of the ioratio values
		const ioratioSum = 0.5 * 100 + 0.25 * 200 + 0.125 * 400;
		const outputAmountSum = 100 + 200 + 400;
		const ioratioAverage = ioratioSum / outputAmountSum;

		expect(result.length).toEqual(1);
		expect(result[0].value).toEqual(ioratioAverage);
	});
}
