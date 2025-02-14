import assert from 'assert';
import { getLocal } from 'mockttp';
import { describe, it, beforeEach, afterEach } from 'vitest';
import { Transaction, AddOrder } from '../../dist/types/js_api.js';
import { getTransaction, getTransactionAddOrders } from '../../dist/cjs/js_api.js';

const transaction1 = {
	id: 'tx1',
	from: '0x1',
	blockNumber: '1',
	timestamp: '1'
} as unknown as Transaction;

const addOrders = [
	{
		id: 'addOrder1',
		order: {
			id: 'order1',
		}
	} as unknown as AddOrder,
	{
		id: 'addOrder2',
		order: {
			id: 'order2',
		}
	} as unknown as AddOrder,
] as unknown as AddOrder[];

describe('Rain Orderbook JS API Package Bindgen Tests - Add Order', async function () {
	// const mockServer = getLocal();
	// beforeEach(() => mockServer.start(8095));
	// afterEach(() => mockServer.stop());

	it('should fetch add orders for a transaction', async () => {
		// await mockServer
		// 	.forPost('/sg2')
		// 	.thenReply(200, JSON.stringify({ data: { add_orders: addOrders } }));

		const result: AddOrder[] = await getTransactionAddOrders('https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-flare/2024-12-13-9dc7/gn', '0xb5d715bc74b7a7f2aac8cca544c1c95e209ed4113b82269ac3285142324bc6af');
        console.log(result);
	});
});
