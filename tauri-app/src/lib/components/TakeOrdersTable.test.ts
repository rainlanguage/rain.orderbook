import { render, screen } from '@testing-library/svelte';
import { test } from 'vitest';
import TakeOrdersTable from './TakeOrdersTable.svelte';
import { expect } from '$lib/test/matchers';
import { mockIPC } from '@tauri-apps/api/mocks';
import { useOrderTakesList } from '$lib/stores/order';
import type { TakeOrderEntity } from '$lib/typeshare/orderTakesList';

const mockTakeOrdersList: TakeOrderEntity[] = [
  {
    id: '0xd8ee7a1d6e33f5df944d4dd81e94c0de493062486e6f8486f2c322b8d031ebe4-0',
    transaction: { id: '0xd8ee7a1d6e33f5df944d4dd81e94c0de493062486e6f8486f2c322b8d031ebe4' },
    sender: { id: '0x56394785a22b3be25470a0e03ed9e0a939c47b9b' },
    timestamp: '1713602537',
    order: { id: '0xeb49978e5f71f0404f699b7d74fe1061f3422965e44d9c115f5e32887b6304d1' },
    ioratio: '29.90807628354063066202090592334495',
    input: '1030034147205139320',
    input_display: '1.03003414720513932',
    input_token: {
      id: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
      name: 'Wrapped Flare',
      symbol: 'WFLR',
      decimals: 18,
    },
    input_ioindex: '1',
    output: '34440',
    output_display: '0.03444',
    output_token: {
      id: '0x96b41289d90444b8add57e6f265db5ae8651df29',
      name: 'Enosys USDT',
      symbol: 'eUSDT',
      decimals: 6,
    },
    output_ioindex: '0',
    context: {
      calling_context: [
        '106423545254858754715383091423888345232228223509976810218083436287306831693009',
        '664893272280738449035774519872832650934729237275',
        '492250579212581818741383073260620269714363677595',
      ],
      calculations_context: ['1030034147205139320', '33435430877117740'],
      vault_inputs_context: [
        '860364364687607118761851984307353862500328267561',
        '6',
        '48759521422719384257020625088036196431539501703281865480252848872791014790211',
        '0',
        '34440',
      ],
      vault_outputs_context: [
        '168432354740743388412308184864856936098254780477',
        '18',
        '48759521422719384257020625088036196431539501703281865480252848872791014790211',
        '1030034147205139320',
        '1030034147205139320',
      ],
    },
  },
];

test('renders table with correct data', async () => {
  mockIPC((cmd) => {
    if (cmd === 'order_takes_list') {
      return mockTakeOrdersList;
    }
  });

  const orderTakesList = useOrderTakesList('1');

  render(TakeOrdersTable, { orderTakesList });
  // letting the store update
  await new Promise((resolve) => setTimeout(resolve, 0));

  // get all the io ratios
  const rows = screen.getAllByTestId('io-ratio');

  // checking the io ratios
  for (let i = 0; i < mockTakeOrdersList.length; i++) {
    const expectedRatio =
      Number(mockTakeOrdersList[i].output_display) / Number(mockTakeOrdersList[i].input_display);
    expect(rows[i]).toHaveTextContent(expectedRatio.toString());
  }
});
