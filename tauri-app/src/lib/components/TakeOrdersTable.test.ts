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
  {
    id: '0x1e5266dc7f5c118dc07052b8e570094248d4eb49cb850c0f89076a93589e4215-0',
    transaction: { id: '0x1e5266dc7f5c118dc07052b8e570094248d4eb49cb850c0f89076a93589e4215' },
    sender: { id: '0x56394785a22b3be25470a0e03ed9e0a939c47b9b' },
    timestamp: '1713598073',
    order: { id: '0xeb49978e5f71f0404f699b7d74fe1061f3422965e44d9c115f5e32887b6304d1' },
    ioratio: '0.03319016179488988609103491138183367',
    input: '34187',
    input_display: '0.034187',
    input_token: {
      id: '0x96b41289d90444b8add57e6f265db5ae8651df29',
      name: 'Enosys USDT',
      symbol: 'eUSDT',
      decimals: 6,
    },
    input_ioindex: '0',
    output: '1030034147205139320',
    output_display: '1.03003414720513932',
    output_token: {
      id: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
      name: 'Wrapped Flare',
      symbol: 'WFLR',
      decimals: 18,
    },
    output_ioindex: '1',
    context: {
      calling_context: [
        '106423545254858754715383091423888345232228223509976810218083436287306831693009',
        '664893272280738449035774519872832650934729237275',
        '492250579212581818741383073260620269714363677595',
      ],
      calculations_context: ['34187000000000000', '30129410220409492494'],
      vault_inputs_context: [
        '168432354740743388412308184864856936098254780477',
        '18',
        '48759521422719384257020625088036196431539501703281865480252848872791014790211',
        '0',
        '1030034147205139320',
      ],
      vault_outputs_context: [
        '860364364687607118761851984307353862500328267561',
        '6',
        '48759521422719384257020625088036196431539501703281865480252848872791014790211',
        '34187',
        '34187',
      ],
    },
  },
  {
    id: '0x83fe02691ec9022740455ef15a19115e34882285d412c5b2b91e3ff26c05938d-0',
    transaction: { id: '0x83fe02691ec9022740455ef15a19115e34882285d412c5b2b91e3ff26c05938d' },
    sender: { id: '0x56394785a22b3be25470a0e03ed9e0a939c47b9b' },
    timestamp: '1713595716',
    order: { id: '0xeb49978e5f71f0404f699b7d74fe1061f3422965e44d9c115f5e32887b6304d1' },
    ioratio: '29.90652596079736095591891654722555',
    input: '1022414403021779379',
    input_display: '1.022414403021779379',
    input_token: {
      id: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
      name: 'Wrapped Flare',
      symbol: 'WFLR',
      decimals: 18,
    },
    input_ioindex: '1',
    output: '34187',
    output_display: '0.034187',
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
      calculations_context: ['1022414403021779379', '33437214084333622'],
      vault_inputs_context: [
        '860364364687607118761851984307353862500328267561',
        '6',
        '48759521422719384257020625088036196431539501703281865480252848872791014790211',
        '0',
        '34187',
      ],
      vault_outputs_context: [
        '168432354740743388412308184864856936098254780477',
        '18',
        '48759521422719384257020625088036196431539501703281865480252848872791014790211',
        '1022414403021779379',
        '1022414403021779379',
      ],
    },
  },
  {
    id: '0xd28f6c69fde8c5dfa2954eaaecd5d5aa537f8be18fc2912fbd9a4333391e941a-0',
    transaction: { id: '0xd28f6c69fde8c5dfa2954eaaecd5d5aa537f8be18fc2912fbd9a4333391e941a' },
    sender: { id: '0x56394785a22b3be25470a0e03ed9e0a939c47b9b' },
    timestamp: '1713419507',
    order: { id: '0xeb49978e5f71f0404f699b7d74fe1061f3422965e44d9c115f5e32887b6304d1' },
    ioratio: '0.03374267801591709915203177499152203',
    input: '34499',
    input_display: '0.034499',
    input_token: {
      id: '0x96b41289d90444b8add57e6f265db5ae8651df29',
      name: 'Enosys USDT',
      symbol: 'eUSDT',
      decimals: 6,
    },
    input_ioindex: '0',
    output: '1022414403021779379',
    output_display: '1.022414403021779379',
    output_token: {
      id: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
      name: 'Wrapped Flare',
      symbol: 'WFLR',
      decimals: 18,
    },
    output_ioindex: '1',
    context: {
      calling_context: [
        '106423545254858754715383091423888345232228223509976810218083436287306831693009',
        '664893272280738449035774519872832650934729237275',
        '492250579212581818741383073260620269714363677595',
      ],
      calculations_context: ['34499000000000000', '29636059103793715128'],
      vault_inputs_context: [
        '168432354740743388412308184864856936098254780477',
        '18',
        '48759521422719384257020625088036196431539501703281865480252848872791014790211',
        '0',
        '1022414403021779379',
      ],
      vault_outputs_context: [
        '860364364687607118761851984307353862500328267561',
        '6',
        '48759521422719384257020625088036196431539501703281865480252848872791014790211',
        '34499',
        '34499',
      ],
    },
  },
  {
    id: '0x9fc4099e5d3e8fc497814c992e70b7144d23ea3c843f12a88b057a2ec757bf0c-0',
    transaction: { id: '0x9fc4099e5d3e8fc497814c992e70b7144d23ea3c843f12a88b057a2ec757bf0c' },
    sender: { id: '0x56394785a22b3be25470a0e03ed9e0a939c47b9b' },
    timestamp: '1713415684',
    order: { id: '0xeb49978e5f71f0404f699b7d74fe1061f3422965e44d9c115f5e32887b6304d1' },
    ioratio: '29.41809730061399133308211832227021',
    input: '1014894938773882087',
    input_display: '1.014894938773882087',
    input_token: {
      id: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
      name: 'Wrapped Flare',
      symbol: 'WFLR',
      decimals: 18,
    },
    input_ioindex: '1',
    output: '34499',
    output_display: '0.034499',
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
      calculations_context: ['1014894938773882087', '33991955513563907'],
      vault_inputs_context: [
        '860364364687607118761851984307353862500328267561',
        '6',
        '48759521422719384257020625088036196431539501703281865480252848872791014790211',
        '0',
        '34499',
      ],
      vault_outputs_context: [
        '168432354740743388412308184864856936098254780477',
        '18',
        '48759521422719384257020625088036196431539501703281865480252848872791014790211',
        '1014894938773882087',
        '1014894938773882087',
      ],
    },
  },
  {
    id: '0xe70a7d109187c57a6fc2ed4a3b4828f59777afffd382f14fd7a522b9f7c4e825-0',
    transaction: { id: '0xe70a7d109187c57a6fc2ed4a3b4828f59777afffd382f14fd7a522b9f7c4e825' },
    sender: { id: '0x56394785a22b3be25470a0e03ed9e0a939c47b9b' },
    timestamp: '1713297781',
    order: { id: '0xeb49978e5f71f0404f699b7d74fe1061f3422965e44d9c115f5e32887b6304d1' },
    ioratio: '0.03398184253600254017439289028411676',
    input: '34488',
    input_display: '0.034488',
    input_token: {
      id: '0x96b41289d90444b8add57e6f265db5ae8651df29',
      name: 'Enosys USDT',
      symbol: 'eUSDT',
      decimals: 6,
    },
    input_ioindex: '0',
    output: '1014894938773882087',
    output_display: '1.014894938773882087',
    output_token: {
      id: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
      name: 'Wrapped Flare',
      symbol: 'WFLR',
      decimals: 18,
    },
    output_ioindex: '1',
    context: {
      calling_context: [
        '106423545254858754715383091423888345232228223509976810218083436287306831693009',
        '664893272280738449035774519872832650934729237275',
        '492250579212581818741383073260620269714363677595',
      ],
      calculations_context: ['34488000000000000', '29427480247444968881'],
      vault_inputs_context: [
        '168432354740743388412308184864856936098254780477',
        '18',
        '48759521422719384257020625088036196431539501703281865480252848872791014790211',
        '0',
        '1014894938773882087',
      ],
      vault_outputs_context: [
        '860364364687607118761851984307353862500328267561',
        '6',
        '48759521422719384257020625088036196431539501703281865480252848872791014790211',
        '34488',
        '34488',
      ],
    },
  },
  {
    id: '0x508987c54b158157cd56285569860f7230c86e9b5ca3ea4c39281f69cca73316-0',
    transaction: { id: '0x508987c54b158157cd56285569860f7230c86e9b5ca3ea4c39281f69cca73316' },
    sender: { id: '0x56394785a22b3be25470a0e03ed9e0a939c47b9b' },
    timestamp: '1713297770',
    order: { id: '0xeb49978e5f71f0404f699b7d74fe1061f3422965e44d9c115f5e32887b6304d1' },
    ioratio: '29.20781028594972729645093945720251',
    input: '1007318961141834195',
    input_display: '1.007318961141834195',
    input_token: {
      id: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
      name: 'Wrapped Flare',
      symbol: 'WFLR',
      decimals: 18,
    },
    input_ioindex: '1',
    output: '34488',
    output_display: '0.034488',
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
      calculations_context: ['1007318961141834195', '34236886944639600'],
      vault_inputs_context: [
        '860364364687607118761851984307353862500328267561',
        '6',
        '48759521422719384257020625088036196431539501703281865480252848872791014790211',
        '0',
        '34488',
      ],
      vault_outputs_context: [
        '168432354740743388412308184864856936098254780477',
        '18',
        '48759521422719384257020625088036196431539501703281865480252848872791014790211',
        '1007318961141834195',
        '1007318961141834195',
      ],
    },
  },
  {
    id: '0x59557f53841a647401432c3e839eba1c2f59f5bcdb4e168bbb50477be6885e0f-0',
    transaction: { id: '0x59557f53841a647401432c3e839eba1c2f59f5bcdb4e168bbb50477be6885e0f' },
    sender: { id: '0x56394785a22b3be25470a0e03ed9e0a939c47b9b' },
    timestamp: '1713193034',
    order: { id: '0xeb49978e5f71f0404f699b7d74fe1061f3422965e44d9c115f5e32887b6304d1' },
    ioratio: '0.03417983908589618024476199591899599',
    input: '34430',
    input_display: '0.03443',
    input_token: {
      id: '0x96b41289d90444b8add57e6f265db5ae8651df29',
      name: 'Enosys USDT',
      symbol: 'eUSDT',
      decimals: 6,
    },
    input_ioindex: '0',
    output: '1007318961141834195',
    output_display: '1.007318961141834195',
    output_token: {
      id: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
      name: 'Wrapped Flare',
      symbol: 'WFLR',
      decimals: 18,
    },
    output_ioindex: '1',
    context: {
      calling_context: [
        '106423545254858754715383091423888345232228223509976810218083436287306831693009',
        '664893272280738449035774519872832650934729237275',
        '492250579212581818741383073260620269714363677595',
      ],
      calculations_context: ['34430000000000000', '29257013103161028027'],
      vault_inputs_context: [
        '168432354740743388412308184864856936098254780477',
        '18',
        '48759521422719384257020625088036196431539501703281865480252848872791014790211',
        '0',
        '1007318961141834195',
      ],
      vault_outputs_context: [
        '860364364687607118761851984307353862500328267561',
        '6',
        '48759521422719384257020625088036196431539501703281865480252848872791014790211',
        '34430',
        '34430',
      ],
    },
  },
  {
    id: '0xc3a0a8673f92dbd23528e7aa07f500ae242e469036c26dd22e7682b0bbd67712-0',
    transaction: { id: '0xc3a0a8673f92dbd23528e7aa07f500ae242e469036c26dd22e7682b0bbd67712' },
    sender: { id: '0x56394785a22b3be25470a0e03ed9e0a939c47b9b' },
    timestamp: '1713186256',
    order: { id: '0xeb49978e5f71f0404f699b7d74fe1061f3422965e44d9c115f5e32887b6304d1' },
    ioratio: '29.04443799012489108335753703165844',
    input: '1000000000000000000',
    input_display: '1',
    input_token: {
      id: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
      name: 'Wrapped Flare',
      symbol: 'WFLR',
      decimals: 18,
    },
    input_ioindex: '1',
    output: '34430',
    output_display: '0.03443',
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
      calculations_context: ['1000000000000000000', '34430000000000000'],
      vault_inputs_context: [
        '860364364687607118761851984307353862500328267561',
        '6',
        '48759521422719384257020625088036196431539501703281865480252848872791014790211',
        '0',
        '34430',
      ],
      vault_outputs_context: [
        '168432354740743388412308184864856936098254780477',
        '18',
        '48759521422719384257020625088036196431539501703281865480252848872791014790211',
        '1000000000000000000',
        '1000000000000000000',
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
