import type { OrderDetailExtended } from '@rainlanguage/orderbook/js_api';

export const mockOrderDetailsExtended: OrderDetailExtended = {
	order: {
		id: 'order1',
		orderBytes: '0x123456',
		orderHash: '0xabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdef',
		owner: '0x1111111111111111111111111111111111111111',
		outputs: [
			{
				id: 'vault1',
				token: {
					id: 'token1',
					address: '0xaaaaaa1111111111111111111111111111111111',
					name: 'Token1',
					symbol: 'TK1',
					decimals: '18'
				},
				balance: '1000',
				vaultId: '0x1111111111111111111111111111111111111111111111111111111111111111',
				orderbook: { id: '0x1111111111111111111111111111111111111111' },
				owner: '0x1111111111111111111111111111111111111111',
				ordersAsOutput: [],
				ordersAsInput: [],
				balanceChanges: []
			}
		],
		inputs: [
			{
				id: 'vault2',
				token: {
					id: 'token2',
					address: '0xbbbbbb2222222222222222222222222222222222',
					name: 'Token2',
					symbol: 'TK2',
					decimals: '18'
				},
				balance: '500',
				vaultId: '0x2222222222222222222222222222222222222222222222222222222222222222',
				orderbook: { id: '0x1111111111111111111111111111111111111111' },
				owner: '0x1111111111111111111111111111111111111111',
				ordersAsOutput: [],
				ordersAsInput: [],
				balanceChanges: []
			}
		],
		active: true,
		addEvents: [
			{
				transaction: {
					id: '0x2222222222222222222222222222222222222222222222222222222222222222',
					from: '0x1111111111111111111111111111111111111111',
					blockNumber: '12345',
					timestamp: '1620000000'
				}
			}
		],
		meta: 'metadata1',
		timestampAdded: '1620000000',
		orderbook: {
			id: '0x00'
		},
		trades: []
	},
	rainlang: 'rainlang1'
};

export const mockOrder = {
	id: '0x9229dadc45c673afcbc393231d5ab0e15bb65719daa5d58bd85adfca3fd60d48',
	orderBytes:
		'0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000b7e455bac373194f20d5962bd8ae1c0884d3436a00000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000062000000000000000000000000000000000000000000000000000000000000006a06322e24062c0c4fe1d4d97a4ecd7fe28f3ab9e86f31d5c5e7f2b3b6d1533ba62000000000000000000000000bd8849759749b4d8506bc851acef0e19f34eabee0000000000000000000000008d96ea3ef24d7123882c51ce4325b89bc0d63f9e000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000004e3000000000000000000000000000000000000000000000000000000000000001100000000000000000000000000000000000000000000000000b1a2bc2ec500000000000000000000000000000000000000000000000000000c7d713b49da0000914d696e20747261646520616d6f756e742e00000000000000000000000000008b616d6f756e742d7573656400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000853a0d2313c000000000000000000000000000000000000000000000000124bc0ddd92e560000000000000000000000000000000000000000000000000000c328093e61ee4000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000016345785d8a00000000000000000000000000000000000000000000000000000e043da6172500008f6c6173742d74726164652d74696d65000000000000000000000000000000008d6c6173742d74726164652d696f0000000000000000000000000000000000008c696e697469616c2d74696d650000000000000000000000000000000000000000000000000000000000000000000000000000000000000006f05b59d3b200000000000000000000000000000000000000000000000000008ac7230489e800000000000000000000000000000000000000000000000000d8d726b7177a8000000000000000000000000000000000000000000000000000d551185379fa1c000000000000000000000000000000000000000000000000000000000000000002830b00000024007400e0015801b401e001f40218025c0264080500040b20000200100001001000000b120003001000010b110004001000030b0100051305000201100001011000003d120000011000020010000003100404211200001d02000001100003031000010c1200004911000003100404001000012b12000001100003031000010c1200004a0200001a0b00090b1000060b20000700100000001000011b1200001a10000047120000001000001a1000004712000001100004011000002e12000001100006011000052e120000001000053d12000001100005001000042e1200000010000601100006001000032e120000481200011d0b020a0010000001100004011000072713000001100003031000010c12000049110000001000030010000247120000001000010b110008001000050110000000100001201200001f12000001100000011000084712000000100006001000073d120000011000002b12000000100008001000043b120000160901080b1000070b10000901100009001000013d1200001b12000001100007001000013d1200000b10000a001000033a120000001000040010000248120001001000000b110008001000053d12000000100006001000042b1200000a0401011a1000000110000a031000010c1200004a020000001000000110000b031000010c1200004a020000040200010110000c031000010c12000049110000080300020110000a031000010c120000491100000110000b031000010c12000049110000100c01030110000e001000002e1200000110000d3e120000001000010010000100100001001000010010000100100001001000010010000100100001001000013d1a0000010100010110000f010100010110001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000af88d065e77c8cc2239327c5edb3a432268e58310000000000000000000000000000000000000000000000000000000000000006b798f8557fa0e7787ba14dc96c22d2f7d2314689d209a54104e1c6685ad1e39c000000000000000000000000000000000000000000000000000000000000000100000000000000000000000082af49447d8a07e3bd95bd0d56f35241523fbab10000000000000000000000000000000000000000000000000000000000000012b798f8557fa0e7787ba14dc96c22d2f7d2314689d209a54104e1c6685ad1e39c',
	orderHash: '0x522af734207572d3c1d2cff938dc355578391b62b7b5e6f45185c481141416f7',
	owner: '0xb7e455bac373194f20d5962bd8ae1c0884d3436a',
	outputs: [
		{
			id: '0x81bd468a9c493e7427ec9b3cafd52cc780a5a0d9074043bcbae608c5f39a6118',
			owner: '0xb7e455bac373194f20d5962bd8ae1c0884d3436a',
			vaultId: '83043525982714387441210092990847911536190235553161609246480078437778300461980',
			balance: '0',
			token: {
				id: '0x82af49447d8a07e3bd95bd0d56f35241523fbab1',
				address: '0x82af49447d8a07e3bd95bd0d56f35241523fbab1',
				name: 'Wrapped Ether',
				symbol: 'WETH',
				decimals: '18'
			},
			orderbook: {
				id: '0x550878091b2b1506069f61ae59e3a5484bca9166'
			},
			ordersAsOutput: [
				{
					id: '0x9229dadc45c673afcbc393231d5ab0e15bb65719daa5d58bd85adfca3fd60d48',
					orderHash: '0x522af734207572d3c1d2cff938dc355578391b62b7b5e6f45185c481141416f7',
					active: false
				}
			],
			ordersAsInput: [],
			balanceChanges: [
				{
					__typename: 'deposit',
					data: {
						id: '0x4085027ab87a1aa27267de3187559dcea51a49bc0dd3789ca65af38c2ba3728e',
						typename: 'Deposit',
						amount: '504119298720293716',
						newVaultBalance: '504119298720293716',
						oldVaultBalance: '0',
						vault: {
							id: '0x81bd468a9c493e7427ec9b3cafd52cc780a5a0d9074043bcbae608c5f39a6118',
							vault_id:
								'83043525982714387441210092990847911536190235553161609246480078437778300461980',
							token: {
								id: '0x82af49447d8a07e3bd95bd0d56f35241523fbab1',
								address: '0x82af49447d8a07e3bd95bd0d56f35241523fbab1',
								name: 'Wrapped Ether',
								symbol: 'WETH',
								decimals: '18'
							}
						},
						timestamp: '1733398198',
						transaction: {
							id: '0xa62c5dd249c906d69604dcccf935b490757ba6286eef3744b9b7e991d11c1b0f',
							from: '0xb7e455bac373194f20d5962bd8ae1c0884d3436a',
							blockNumber: '281580844',
							timestamp: '1733398198'
						},
						orderbook: {
							id: '0x550878091b2b1506069f61ae59e3a5484bca9166'
						}
					}
				},
				{
					__typename: 'withdrawal',
					data: {
						id: '0x78cc1de97a4afeedb0b0b11f273b21bedf7642256893073b81b253d1845ac802',
						typename: 'Withdrawal',
						amount: '-504119298720293716',
						newVaultBalance: '0',
						oldVaultBalance: '504119298720293716',
						vault: {
							id: '0x81bd468a9c493e7427ec9b3cafd52cc780a5a0d9074043bcbae608c5f39a6118',
							vault_id:
								'83043525982714387441210092990847911536190235553161609246480078437778300461980',
							token: {
								id: '0x82af49447d8a07e3bd95bd0d56f35241523fbab1',
								address: '0x82af49447d8a07e3bd95bd0d56f35241523fbab1',
								name: 'Wrapped Ether',
								symbol: 'WETH',
								decimals: '18'
							}
						},
						timestamp: '1733408176',
						transaction: {
							id: '0xf07aaa0493d64cb42e94ed72c4f47a1e57885dc1d71a1b4586f9cf987fa068ee',
							from: '0xb7e455bac373194f20d5962bd8ae1c0884d3436a',
							blockNumber: '281620727',
							timestamp: '1733408176'
						},
						orderbook: {
							id: '0x550878091b2b1506069f61ae59e3a5484bca9166'
						}
					}
				}
			]
		}
	],
	inputs: [
		{
			id: '0x1f97367df62b14b4088b8ea897f1b0adc021001a55397d513f8ff6c39d54b671',
			owner: '0xb7e455bac373194f20d5962bd8ae1c0884d3436a',
			vaultId: '83043525982714387441210092990847911536190235553161609246480078437778300461980',
			balance: '0',
			token: {
				id: '0xaf88d065e77c8cc2239327c5edb3a432268e5831',
				address: '0xaf88d065e77c8cc2239327c5edb3a432268e5831',
				name: 'USD Coin',
				symbol: 'USDC',
				decimals: '6'
			},
			orderbook: {
				id: '0x550878091b2b1506069f61ae59e3a5484bca9166'
			},
			ordersAsOutput: [],
			ordersAsInput: [
				{
					id: '0x9229dadc45c673afcbc393231d5ab0e15bb65719daa5d58bd85adfca3fd60d48',
					orderHash: '0x522af734207572d3c1d2cff938dc355578391b62b7b5e6f45185c481141416f7',
					active: false
				}
			],
			balanceChanges: []
		}
	],
	orderbook: {
		id: '0x550878091b2b1506069f61ae59e3a5484bca9166'
	},
	active: false,
	timestampAdded: '1733398040',
	meta: '0xff0a89c674ee7874a300590ac02f2a20302e2063616c63756c6174652d696f202a2f200a7573696e672d776f7264732d66726f6d20307862303632303261413346653764383531373166423761413566313730313164313745363366333832203078623036323032614133466537643835313731664237614135663137303131643137453633663338320a616d6f756e742d65706f6368730a74726164652d65706f6368733a63616c6c3c323e28292c0a6d61782d6f75747075743a2063616c6c3c333e28616d6f756e742d65706f6368732074726164652d65706f636873292c0a696f3a2063616c6c3c343e2874726164652d65706f636873292c0a3a63616c6c3c353e28696f293b0a0a2f2a20312e2068616e646c652d696f202a2f200a6d696e2d616d6f756e743a206d756c28302e303520302e39292c0a3a656e7375726528677265617465722d7468616e2d6f722d657175616c2d746f286f75747075742d7661756c742d64656372656173652829206d696e2d616d6f756e742920224d696e20747261646520616d6f756e742e22292c0a757365643a206765742868617368286f726465722d6861736828292022616d6f756e742d757365642229292c0a3a7365742868617368286f726465722d6861736828292022616d6f756e742d757365642229206164642875736564206f75747075742d7661756c742d6465637265617365282929293b0a0a2f2a20322e206765742d65706f6368202a2f200a696e697469616c2d74696d653a2063616c6c3c363e28292c0a6c6173742d74696d65205f3a2063616c6c3c373e28292c0a6475726174696f6e3a20737562286e6f77282920616e79286c6173742d74696d6520696e697469616c2d74696d6529292c0a746f74616c2d6475726174696f6e3a20737562286e6f77282920696e697469616c2d74696d65292c0a726174696f2d667265657a652d616d6f756e742d65706f6368733a2064697628302e303520302e36292c0a726174696f2d667265657a652d74726164652d65706f6368733a206d756c28726174696f2d667265657a652d616d6f756e742d65706f63687320646976283836343030203336303029292c0a616d6f756e742d65706f6368733a2064697628746f74616c2d6475726174696f6e203836343030292c0a74726164652d65706f6368733a2073617475726174696e672d73756228646976286475726174696f6e20333630302920726174696f2d667265657a652d74726164652d65706f636873293b0a0a2f2a20332e20616d6f756e742d666f722d65706f6368202a2f200a616d6f756e742d65706f6368730a74726164652d65706f6368733a2c0a746f74616c2d617661696c61626c653a206c696e6561722d67726f777468283020302e3620616d6f756e742d65706f636873292c0a757365643a206765742868617368286f726465722d6861736828292022616d6f756e742d757365642229292c0a756e757365643a2073756228746f74616c2d617661696c61626c652075736564292c0a64656361793a2063616c6c3c383e2874726164652d65706f636873292c0a7368792d64656361793a20657665727928677265617465722d7468616e2874726164652d65706f63687320302e303529206465636179292c0a7661726961626c652d636f6d706f6e656e743a2073756228302e3120302e3035292c0a7461726765742d616d6f756e743a2061646428302e3035206d756c287661726961626c652d636f6d706f6e656e74207368792d646563617929292c0a6361707065642d756e757365643a206d696e28756e75736564207461726765742d616d6f756e74293b0a0a2f2a20342e20696f2d666f722d65706f6368202a2f200a65706f63683a2c0a6c6173742d696f3a2063616c6c3c373e28292c0a6d61782d6e6578742d74726164653a20616e79286d756c286c6173742d696f20312e3031292063616c6c3c393e2829292c0a626173656c696e652d6e6578742d74726164653a206d756c286c6173742d696f2030292c0a7265616c2d626173656c696e653a206d617828626173656c696e652d6e6578742d74726164652063616c6c3c31303e2829292c0a7661726961626c652d636f6d706f6e656e743a2073617475726174696e672d737562286d61782d6e6578742d7472616465207265616c2d626173656c696e65292c0a61626f76652d626173656c696e653a206d756c287661726961626c652d636f6d706f6e656e742063616c6c3c383e2865706f636829292c0a5f3a20616464287265616c2d626173656c696e652061626f76652d626173656c696e65293b0a0a2f2a20352e207365742d6c6173742d7472616465202a2f200a6c6173742d696f3a2c0a3a7365742868617368286f726465722d68617368282920226c6173742d74726164652d74696d652229206e6f772829292c0a3a7365742868617368286f726465722d68617368282920226c6173742d74726164652d696f2229206c6173742d696f293b0a0a2f2a20362e206765742d696e697469616c2d74696d65202a2f200a5f3a6765742868617368286f726465722d6861736828292022696e697469616c2d74696d652229293b0a0a2f2a20372e206765742d6c6173742d7472616465202a2f200a6c6173742d74696d653a6765742868617368286f726465722d68617368282920226c6173742d74726164652d74696d652229292c0a6c6173742d696f3a6765742868617368286f726465722d68617368282920226c6173742d74726164652d696f2229293b0a0a2f2a20382e2068616c666c696665202a2f200a65706f63683a2c0a2f2a2a0a202a20536872696e6b696e6720746865206d756c7469706c696572206c696b6520746869730a202a207468656e206170706c79696e672069742031302074696d657320616c6c6f777320666f720a202a2062657474657220707265636973696f6e207768656e206d61782d696f2d726174696f0a202a2069732076657279206c617267652c20652e672e207e31653130206f72207e316532302b0a202a0a202a205468697320776f726b7320626563617573652060706f77657260206c6f7365730a202a20707265636973696f6e206f6e20626173652060302e3560207768656e207468650a202a206578706f6e656e74206973206c6172676520616e642063616e206576656e20676f0a202a20746f20603060207768696c652074686520696f2d726174696f206973207374696c6c0a202a206c617267652e2042657474657220746f206b65657020746865206d756c7469706c6965720a202a2068696768657220707265636973696f6e20616e642064726f702074686520696f2d726174696f0a202a20736d6f6f74686c7920666f72206173206c6f6e672061732077652063616e2e0a202a2f0a6d756c7469706c6965723a0a2020706f77657228302e35206469762865706f636820313029292c0a76616c3a0a20206d756c280a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a2020293b0a0a2f2a20392e20636f6e7374616e742d696e697469616c2d696f202a2f200a5f3a20343030303b0a0a2f2a2031302e20636f6e7374616e742d626173656c696e65202a2f200a5f3a20333933353b011bff13109e41336ff20278186170706c69636174696f6e2f6f637465742d73747265616d',
	addEvents: [
		{
			transaction: {
				id: '0xb47869bda04e3407e41120ae3a031af874fe0b82ba39a99496acffc257180553',
				from: '0xb7e455bac373194f20d5962bd8ae1c0884d3436a',
				blockNumber: '281580211',
				timestamp: '1733398040'
			}
		}
	],
	trades: []
};
