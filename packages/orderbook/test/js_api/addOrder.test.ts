import assert from 'assert';
import { getLocal } from 'mockttp';
import { describe, it, beforeEach, afterEach } from 'vitest';
import { Transaction, AddOrderWithOrder } from '../../dist/types/js_api.js';
import { getTransactionAddOrders } from '../../dist/cjs/js_api.js';

const transaction1: Transaction = {
	id: '0xb5d715bc74b7a7f2aac8cca544c1c95e209ed4113b82269ac3285142324bc6af',
	from: '0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11',
	blockNumber: '37432554',
	timestamp: '1739448802'
};

const mockAddOrder: AddOrderWithOrder = {
	transaction: {
		id: '0xb5d715bc74b7a7f2aac8cca544c1c95e209ed4113b82269ac3285142324bc6af',
		from: '0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11',
		blockNumber: '37432554',
		timestamp: '1739448802'
	},
	order: {
		id: '0x1a69eeb7970d3c8d5776493327fb262e31fc880c9cc4a951607418a7963d9fa1',
		orderBytes:
			'0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000f08bcbce72f62c95dcb7c07dcb5ed26acfcfbc1100000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000005c00000000000000000000000000000000000000000000000000000000000000640392c489ef67afdc348209452c338ea5ba2b6152b936e152f610d05e1a20621a40000000000000000000000005fb33d710f8b58de4c9fdec703b5c2487a5219d600000000000000000000000084c6e7f5a1e5dd89594cc25bef4722a1b8871ae60000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000049d000000000000000000000000000000000000000000000000000000000000000f0000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000c7d713b49da0000914d696e20747261646520616d6f756e742e00000000000000000000000000008b616d6f756e742d75736564000000000000000000000000000000000000000000000000000000000000000000000000000000000000000340aad21b3b70000000000000000000000000000000000000000000000000006194049f30f7200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b1a2bc2ec500000000000000000000000000000000000000000000000000000e043da6172500008f6c6173742d74726164652d74696d65000000000000000000000000000000008d6c6173742d74726164652d696f0000000000000000000000000000000000008c696e697469616c2d74696d650000000000000000000000000000000000000000000000000000000000000000000000000000000000000006f05b59d3b200000000000000000000000000000000000000000000000000008ac7230489e80000000000000000000000020000915e36ef882941816356bc3718df868054f868ad000000000000000000000000000000000000000000000000000000000000027d0a00000024007400e0015801b401e001f40218025c080500040b20000200100001001000000b120003001000010b110004001000030b0100051305000201100001011000003d120000011000020010000003100404211200001d02000001100003031000010c1200004911000003100404001000012b12000001100003031000010c1200004a0200001a0b00090b1000060b20000700100000001000011b1200001a10000047120000001000001a1000004712000001100000011000002e12000001100005011000042e120000001000053d12000001100004001000042e1200000010000601100005001000032e120000481200011d0b020a0010000001100000011000062713000001100003031000010c12000049110000001000030010000247120000001000010b110008001000050110000700100001201200001f12000001100000011000004712000000100006001000073d120000011000002b12000000100008001000043b120000160901080b1000070b10000901100008001000013d1200001b12000001100006001000013d1200000b100009001000033a120000001000040010000248120001001000000b110008001000053d12000000100006001000042b1200000a0401011a10000001100009031000010c1200004a020000001000000110000a031000010c1200004a020000040200010110000b031000010c120000491100000803000201100009031000010c120000491100000110000a031000010c12000049110000100c01030110000d001000002e1200000110000c3e120000001000010010000100100001001000010010000100100001001000010010000100100001001000013d1a0000020100010210000e3611000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000001d80c49bbbcd1c0911346656b529df9e5c2f783d0000000000000000000000000000000000000000000000000000000000000012a6e3c06415539f92823a18ba63e1c0303040c4892970a0d1e3a27663d7583b33000000000000000000000000000000000000000000000000000000000000000100000000000000000000000012e605bc104e93b45e1ad99f9e555f659051c2bb0000000000000000000000000000000000000000000000000000000000000012a6e3c06415539f92823a18ba63e1c0303040c4892970a0d1e3a27663d7583b33',
		orderHash: '0x557147dd0daa80d5beff0023fe6a3505469b2b8c4406ce1ab873e1a652572dd4',
		owner: '0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11',
		outputs: [
			{
				id: '0x49f6b665c395c7b975caa2fc167cb5119981bbb86798bcaf3c4570153d09dfcf',
				owner: '0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11',
				vaultId: '75486334982066122983501547829219246999490818941767825330875804445439814023987',
				balance: '987000000000000000',
				token: {
					id: '0x12e605bc104e93b45e1ad99f9e555f659051c2bb',
					address: '0x12e605bc104e93b45e1ad99f9e555f659051c2bb',
					name: 'Staked FLR',
					symbol: 'sFLR',
					decimals: '18'
				},
				orderbook: {
					id: '0xcee8cd002f151a536394e564b84076c41bbbcd4d'
				},
				ordersAsOutput: [
					{
						id: '0x1a69eeb7970d3c8d5776493327fb262e31fc880c9cc4a951607418a7963d9fa1',
						orderHash: '0x557147dd0daa80d5beff0023fe6a3505469b2b8c4406ce1ab873e1a652572dd4',
						active: true
					}
				],
				ordersAsInput: [],
				balanceChanges: [
					{
						__typename: 'deposit',
						data: {
							id: '0x1bf9c93f8ac04810e733b61a7d5dabba66fc1a47235e6ab027e76c9758a2a9e8',
							__typename: 'Deposit',
							amount: '1000000000000000000',
							newVaultBalance: '1000000000000000000',
							oldVaultBalance: '0',
							vault: {
								id: '0x49f6b665c395c7b975caa2fc167cb5119981bbb86798bcaf3c4570153d09dfcf',
								vault_id:
									'75486334982066122983501547829219246999490818941767825330875804445439814023987',
								token: {
									id: '0x12e605bc104e93b45e1ad99f9e555f659051c2bb',
									address: '0x12e605bc104e93b45e1ad99f9e555f659051c2bb',
									name: 'Staked FLR',
									symbol: 'sFLR',
									decimals: '18'
								}
							},
							timestamp: '1739448802',
							transaction: {
								id: '0xb5d715bc74b7a7f2aac8cca544c1c95e209ed4113b82269ac3285142324bc6af',
								from: '0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11',
								blockNumber: '37432554',
								timestamp: '1739448802'
							},
							orderbook: {
								id: '0xcee8cd002f151a536394e564b84076c41bbbcd4d'
							}
						}
					},
					{
						__typename: 'withdrawal',
						data: {
							id: '0x252f6727a7a9bf1047cd9764351e9a2514140c5664589b0e5ecc7f9a4c69329c',
							__typename: 'Withdrawal',
							amount: '-11000000000000000',
							newVaultBalance: '987000000000000000',
							oldVaultBalance: '998000000000000000',
							vault: {
								id: '0x49f6b665c395c7b975caa2fc167cb5119981bbb86798bcaf3c4570153d09dfcf',
								vault_id:
									'75486334982066122983501547829219246999490818941767825330875804445439814023987',
								token: {
									id: '0x12e605bc104e93b45e1ad99f9e555f659051c2bb',
									address: '0x12e605bc104e93b45e1ad99f9e555f659051c2bb',
									name: 'Staked FLR',
									symbol: 'sFLR',
									decimals: '18'
								}
							},
							timestamp: '1739460802',
							transaction: {
								id: '0xf4052dcf0a9ef208be249822c002bf656d273b4583e92928066fd8fb0a67c3f0',
								from: '0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11',
								blockNumber: '37439233',
								timestamp: '1739460802'
							},
							orderbook: {
								id: '0xcee8cd002f151a536394e564b84076c41bbbcd4d'
							}
						}
					},
					{
						__typename: 'withdrawal',
						data: {
							id: '0x3b272ce8735a1778d584ed2d49532d571a815909b8f89b2d7d2c6744fcf7cb7c',
							__typename: 'Withdrawal',
							amount: '-1000000000000000',
							newVaultBalance: '998000000000000000',
							oldVaultBalance: '999000000000000000',
							vault: {
								id: '0x49f6b665c395c7b975caa2fc167cb5119981bbb86798bcaf3c4570153d09dfcf',
								vault_id:
									'75486334982066122983501547829219246999490818941767825330875804445439814023987',
								token: {
									id: '0x12e605bc104e93b45e1ad99f9e555f659051c2bb',
									address: '0x12e605bc104e93b45e1ad99f9e555f659051c2bb',
									name: 'Staked FLR',
									symbol: 'sFLR',
									decimals: '18'
								}
							},
							timestamp: '1739460777',
							transaction: {
								id: '0xe3e1be9b3e11420de1f1d34f460c14d8688183b78b2dbcfd9b45560b553e451a',
								from: '0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11',
								blockNumber: '37439219',
								timestamp: '1739460777'
							},
							orderbook: {
								id: '0xcee8cd002f151a536394e564b84076c41bbbcd4d'
							}
						}
					},
					{
						__typename: 'withdrawal',
						data: {
							id: '0x9d19a7aa2486c2640669eb04c8c4ed3e11073a04767d6dcfc3468ae12f695849',
							__typename: 'Withdrawal',
							amount: '-1000000000000000',
							newVaultBalance: '999000000000000000',
							oldVaultBalance: '1000000000000000000',
							vault: {
								id: '0x49f6b665c395c7b975caa2fc167cb5119981bbb86798bcaf3c4570153d09dfcf',
								vault_id:
									'75486334982066122983501547829219246999490818941767825330875804445439814023987',
								token: {
									id: '0x12e605bc104e93b45e1ad99f9e555f659051c2bb',
									address: '0x12e605bc104e93b45e1ad99f9e555f659051c2bb',
									name: 'Staked FLR',
									symbol: 'sFLR',
									decimals: '18'
								}
							},
							timestamp: '1739460481',
							transaction: {
								id: '0x3bf239fb20fed202f04da468cc62d762390ab5f80b67b477565f740277f94df3',
								from: '0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11',
								blockNumber: '37439068',
								timestamp: '1739460481'
							},
							orderbook: {
								id: '0xcee8cd002f151a536394e564b84076c41bbbcd4d'
							}
						}
					}
				]
			}
		],
		inputs: [
			{
				id: '0x538830b4f8cc03840cea5af799dc532be4363a3ee8f4c6123dbff7a0acc86dac',
				owner: '0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11',
				vaultId: '75486334982066122983501547829219246999490818941767825330875804445439814023987',
				balance: '797990000000000000',
				token: {
					id: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
					address: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
					name: 'Wrapped Flare',
					symbol: 'WFLR',
					decimals: '18'
				},
				orderbook: {
					id: '0xcee8cd002f151a536394e564b84076c41bbbcd4d'
				},
				ordersAsOutput: [],
				ordersAsInput: [
					{
						id: '0x1a69eeb7970d3c8d5776493327fb262e31fc880c9cc4a951607418a7963d9fa1',
						orderHash: '0x557147dd0daa80d5beff0023fe6a3505469b2b8c4406ce1ab873e1a652572dd4',
						active: true
					}
				],
				balanceChanges: [
					{
						__typename: 'withdrawal',
						data: {
							id: '0x3c8de8385099c2f7775cb4695af43d7e38863ae9442402d73f70ebf865da1c4c',
							__typename: 'Withdrawal',
							amount: '-2000000000000000',
							newVaultBalance: '797990000000000000',
							oldVaultBalance: '799990000000000000',
							vault: {
								id: '0x538830b4f8cc03840cea5af799dc532be4363a3ee8f4c6123dbff7a0acc86dac',
								vault_id:
									'75486334982066122983501547829219246999490818941767825330875804445439814023987',
								token: {
									id: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
									address: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
									name: 'Wrapped Flare',
									symbol: 'WFLR',
									decimals: '18'
								}
							},
							timestamp: '1739460781',
							transaction: {
								id: '0x6198a5fdf46f37f336bbd8615c18757f3a83ead6ec63ad02d865d46feb284310',
								from: '0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11',
								blockNumber: '37439221',
								timestamp: '1739460781'
							},
							orderbook: {
								id: '0xcee8cd002f151a536394e564b84076c41bbbcd4d'
							}
						}
					},
					{
						__typename: 'withdrawal',
						data: {
							id: '0x7616be6722758517786fdcd94549ce0172d7d34fd411b5778ee0667cd1b1bdba',
							__typename: 'Withdrawal',
							amount: '-10000000000000',
							newVaultBalance: '999990000000000000',
							oldVaultBalance: '1000000000000000000',
							vault: {
								id: '0x538830b4f8cc03840cea5af799dc532be4363a3ee8f4c6123dbff7a0acc86dac',
								vault_id:
									'75486334982066122983501547829219246999490818941767825330875804445439814023987',
								token: {
									id: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
									address: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
									name: 'Wrapped Flare',
									symbol: 'WFLR',
									decimals: '18'
								}
							},
							timestamp: '1739460415',
							transaction: {
								id: '0x8562f41d7d4a8af98ed9db1fbb9575f759846edfab0c4310fc2962b93c5eac7d',
								from: '0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11',
								blockNumber: '37439034',
								timestamp: '1739460415'
							},
							orderbook: {
								id: '0xcee8cd002f151a536394e564b84076c41bbbcd4d'
							}
						}
					},
					{
						__typename: 'withdrawal',
						data: {
							id: '0x8e0c007bc831906b8b327be965e6aded6f5b8bc4905823b3047dcd2a69f01c83',
							__typename: 'Withdrawal',
							amount: '-200000000000000000',
							newVaultBalance: '799990000000000000',
							oldVaultBalance: '999990000000000000',
							vault: {
								id: '0x538830b4f8cc03840cea5af799dc532be4363a3ee8f4c6123dbff7a0acc86dac',
								vault_id:
									'75486334982066122983501547829219246999490818941767825330875804445439814023987',
								token: {
									id: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
									address: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
									name: 'Wrapped Flare',
									symbol: 'WFLR',
									decimals: '18'
								}
							},
							timestamp: '1739460627',
							transaction: {
								id: '0xb330355574bd73c72d61b102ba7d23a0e07d677cb97e71db4495d0472587649b',
								from: '0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11',
								blockNumber: '37439143',
								timestamp: '1739460627'
							},
							orderbook: {
								id: '0xcee8cd002f151a536394e564b84076c41bbbcd4d'
							}
						}
					},
					{
						__typename: 'deposit',
						data: {
							id: '0xcc853bdf3784e8c2e2ac9a43bdc9a2e56cc0d880a10ae8d25c3d675f6d114e74',
							__typename: 'Deposit',
							amount: '1000000000000000000',
							newVaultBalance: '1000000000000000000',
							oldVaultBalance: '0',
							vault: {
								id: '0x538830b4f8cc03840cea5af799dc532be4363a3ee8f4c6123dbff7a0acc86dac',
								vault_id:
									'75486334982066122983501547829219246999490818941767825330875804445439814023987',
								token: {
									id: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
									address: '0x1d80c49bbbcd1c0911346656b529df9e5c2f783d',
									name: 'Wrapped Flare',
									symbol: 'WFLR',
									decimals: '18'
								}
							},
							timestamp: '1739460078',
							transaction: {
								id: '0x1f628ccbe37c1395b81c25cc1d9bfef6266d9782c093e1c42bab225335fe8ba0',
								from: '0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11',
								blockNumber: '37438849',
								timestamp: '1739460078'
							},
							orderbook: {
								id: '0xcee8cd002f151a536394e564b84076c41bbbcd4d'
							}
						}
					}
				]
			}
		],
		orderbook: {
			id: '0xcee8cd002f151a536394e564b84076c41bbbcd4d'
		},
		active: true,
		timestampAdded: '1739448802',
		meta: '0xff0a89c674ee7874a300590a932f2a20302e2063616c63756c6174652d696f202a2f200a7573696e672d776f7264732d66726f6d20307846653234313143446131393344394534653833413563323334433746643332303130313838336143203078393135453336656638383239343138313633353662433337313844663836383035344638363861440a616d6f756e742d65706f6368730a74726164652d65706f6368733a63616c6c3c323e28292c0a6d61782d6f75747075743a2063616c6c3c333e28616d6f756e742d65706f6368732074726164652d65706f636873292c0a696f3a2063616c6c3c343e2874726164652d65706f636873292c0a3a63616c6c3c353e28696f293b0a0a2f2a20312e2068616e646c652d696f202a2f200a6d696e2d616d6f756e743a206d756c283120302e39292c0a3a656e7375726528677265617465722d7468616e2d6f722d657175616c2d746f286f75747075742d7661756c742d64656372656173652829206d696e2d616d6f756e742920224d696e20747261646520616d6f756e742e22292c0a757365643a206765742868617368286f726465722d6861736828292022616d6f756e742d757365642229292c0a3a7365742868617368286f726465722d6861736828292022616d6f756e742d757365642229206164642875736564206f75747075742d7661756c742d6465637265617365282929293b0a0a2f2a20322e206765742d65706f6368202a2f200a696e697469616c2d74696d653a2063616c6c3c363e28292c0a6c6173742d74696d65205f3a2063616c6c3c373e28292c0a6475726174696f6e3a20737562286e6f77282920616e79286c6173742d74696d6520696e697469616c2d74696d6529292c0a746f74616c2d6475726174696f6e3a20737562286e6f77282920696e697469616c2d74696d65292c0a726174696f2d667265657a652d616d6f756e742d65706f6368733a2064697628312031292c0a726174696f2d667265657a652d74726164652d65706f6368733a206d756c28726174696f2d667265657a652d616d6f756e742d65706f63687320646976283630203138303029292c0a616d6f756e742d65706f6368733a2064697628746f74616c2d6475726174696f6e203630292c0a74726164652d65706f6368733a2073617475726174696e672d73756228646976286475726174696f6e20313830302920726174696f2d667265657a652d74726164652d65706f636873293b0a0a2f2a20332e20616d6f756e742d666f722d65706f6368202a2f200a616d6f756e742d65706f6368730a74726164652d65706f6368733a2c0a746f74616c2d617661696c61626c653a206c696e6561722d67726f7774682830203120616d6f756e742d65706f636873292c0a757365643a206765742868617368286f726465722d6861736828292022616d6f756e742d757365642229292c0a756e757365643a2073756228746f74616c2d617661696c61626c652075736564292c0a64656361793a2063616c6c3c383e2874726164652d65706f636873292c0a7368792d64656361793a20657665727928677265617465722d7468616e2874726164652d65706f63687320302e303529206465636179292c0a7661726961626c652d636f6d706f6e656e743a2073756228312031292c0a7461726765742d616d6f756e743a206164642831206d756c287661726961626c652d636f6d706f6e656e74207368792d646563617929292c0a6361707065642d756e757365643a206d696e28756e75736564207461726765742d616d6f756e74293b0a0a2f2a20342e20696f2d666f722d65706f6368202a2f200a65706f63683a2c0a6c6173742d696f3a2063616c6c3c373e28292c0a6d61782d6e6578742d74726164653a20616e79286d756c286c6173742d696f20312e3031292063616c6c3c393e2829292c0a626173656c696e652d6e6578742d74726164653a206d756c286c6173742d696f2030292c0a7265616c2d626173656c696e653a206d617828626173656c696e652d6e6578742d74726164652063616c6c3c393e2829292c0a7661726961626c652d636f6d706f6e656e743a2073617475726174696e672d737562286d61782d6e6578742d7472616465207265616c2d626173656c696e65292c0a61626f76652d626173656c696e653a206d756c287661726961626c652d636f6d706f6e656e742063616c6c3c383e2865706f636829292c0a5f3a20616464287265616c2d626173656c696e652061626f76652d626173656c696e65293b0a0a2f2a20352e207365742d6c6173742d7472616465202a2f200a6c6173742d696f3a2c0a3a7365742868617368286f726465722d68617368282920226c6173742d74726164652d74696d652229206e6f772829292c0a3a7365742868617368286f726465722d68617368282920226c6173742d74726164652d696f2229206c6173742d696f293b0a0a2f2a20362e206765742d696e697469616c2d74696d65202a2f200a5f3a6765742868617368286f726465722d6861736828292022696e697469616c2d74696d652229293b0a0a2f2a20372e206765742d6c6173742d7472616465202a2f200a6c6173742d74696d653a6765742868617368286f726465722d68617368282920226c6173742d74726164652d74696d652229292c0a6c6173742d696f3a6765742868617368286f726465722d68617368282920226c6173742d74726164652d696f2229293b0a0a2f2a20382e2068616c666c696665202a2f200a65706f63683a2c0a2f2a2a0a202a20536872696e6b696e6720746865206d756c7469706c696572206c696b6520746869730a202a207468656e206170706c79696e672069742031302074696d657320616c6c6f777320666f720a202a2062657474657220707265636973696f6e207768656e206d61782d696f2d726174696f0a202a2069732076657279206c617267652c20652e672e207e31653130206f72207e316532302b0a202a0a202a205468697320776f726b7320626563617573652060706f77657260206c6f7365730a202a20707265636973696f6e206f6e20626173652060302e3560207768656e207468650a202a206578706f6e656e74206973206c6172676520616e642063616e206576656e20676f0a202a20746f20603060207768696c652074686520696f2d726174696f206973207374696c6c0a202a206c617267652e2042657474657220746f206b65657020746865206d756c7469706c6965720a202a2068696768657220707265636973696f6e20616e642064726f702074686520696f2d726174696f0a202a20736d6f6f74686c7920666f72206173206c6f6e672061732077652063616e2e0a202a2f0a6d756c7469706c6965723a0a2020706f77657228302e35206469762865706f636820313029292c0a76616c3a0a20206d756c280a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a2020293b0a0a2f2a20392e2073666c722d626173656c696e652d696e76202a2f200a5f3a20696e762873666c722d65786368616e67652d726174652829293b011bff13109e41336ff20278186170706c69636174696f6e2f6f637465742d73747265616d',
		addEvents: [
			{
				transaction: {
					id: '0xb5d715bc74b7a7f2aac8cca544c1c95e209ed4113b82269ac3285142324bc6af',
					from: '0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11',
					blockNumber: '37432554',
					timestamp: '1739448802'
				}
			}
		],
		trades: []
	}
};

const addOrders: AddOrderWithOrder[] = [mockAddOrder];

describe('Rain Orderbook JS API Package Bindgen Tests - Add Order', async function () {
	const mockServer = getLocal();
	beforeEach(() => mockServer.start(8091));
	afterEach(() => mockServer.stop());

	it('should fetch add orders for a transaction', async () => {
		await mockServer
			.forPost('/sg1')
			.thenReply(200, JSON.stringify({ data: { addOrders: addOrders } }));

		try {
			const result: AddOrderWithOrder[] = await getTransactionAddOrders(
				mockServer.url + '/sg1',
				transaction1.id
			);
			assert.equal(result[0].order.id, mockAddOrder.order.id);
		} catch (e) {
			assert.fail('expected to resolve, but failed' + (e instanceof Error ? e.message : String(e)));
		}
	});
});
