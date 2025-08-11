use super::*;

impl OrderbookSubgraphClient {
    /// Fetch single order
    pub async fn order_detail(&self, id: &Id) -> Result<SgOrder, OrderbookSubgraphClientError> {
        let data = self
            .query::<SgOrderDetailByIdQuery, SgIdQueryVariables>(SgIdQueryVariables { id })
            .await?;
        let order = data.order.ok_or(OrderbookSubgraphClientError::Empty)?;

        Ok(order)
    }

    /// Fetch batch orders given their order id
    pub async fn batch_order_detail(
        &self,
        id_list: Vec<SgBytes>,
    ) -> Result<Vec<SgOrder>, OrderbookSubgraphClientError> {
        if id_list.is_empty() {
            return Ok(vec![]);
        }
        let data = self
            .query::<SgBatchOrderDetailQuery, SgBatchOrderDetailQueryVariables>(
                SgBatchOrderDetailQueryVariables {
                    id_list: SgOrderIdList { id_in: id_list },
                },
            )
            .await?;

        Ok(data.orders)
    }

    fn build_filter_with_tokens(
        tokens: Vec<String>,
        basic_filters: SgOrdersListQueryFilters,
    ) -> SgOrdersListQueryAnyFilters {
        let vault_token_filter = Some(SgVaultTokenFilter {
            token_in: tokens.clone(),
        });

        let filter_with_inputs = SgOrdersListQueryFilters {
            inputs_: vault_token_filter.clone(),
            ..basic_filters.clone()
        };
        let filter_with_outputs = SgOrdersListQueryFilters {
            outputs_: vault_token_filter.clone(),
            ..basic_filters.clone()
        };
        SgOrdersListQueryAnyFilters {
            or: vec![filter_with_inputs, filter_with_outputs],
        }
    }

    /// Fetch all orders, paginated
    pub async fn orders_list(
        &self,
        filter_args: SgOrdersListFilterArgs,
        pagination_args: SgPaginationArgs,
    ) -> Result<Vec<SgOrder>, OrderbookSubgraphClientError> {
        let pagination_variables = Self::parse_pagination_args(pagination_args);

        let has_basic_filters = !filter_args.owners.is_empty()
            || filter_args.active.is_some()
            || filter_args.order_hash.is_some();
        let has_token_filters = !filter_args.tokens.is_empty();

        let filters = if has_basic_filters || has_token_filters {
            let basic_filters = SgOrdersListQueryFilters {
                owner_in: filter_args.owners.clone(),
                active: filter_args.active,
                order_hash: filter_args.order_hash.clone(),
                inputs_: None,
                outputs_: None,
            };

            Some(if has_token_filters {
                Self::build_filter_with_tokens(filter_args.tokens, basic_filters)
            } else {
                SgOrdersListQueryAnyFilters {
                    or: vec![basic_filters],
                }
            })
        } else {
            None
        };

        let variables = SgOrdersListQueryVariables {
            first: pagination_variables.first,
            skip: pagination_variables.skip,
            filters,
        };

        let data = self
            .query::<SgOrdersListQuery, SgOrdersListQueryVariables>(variables)
            .await?;

        Ok(data.orders)
    }

    /// Fetch all pages of orders_list query
    pub async fn orders_list_all(&self) -> Result<Vec<SgOrder>, OrderbookSubgraphClientError> {
        let mut all_pages_merged = vec![];
        let mut page = 1;

        loop {
            let page_data = self
                .orders_list(
                    SgOrdersListFilterArgs {
                        owners: vec![],
                        active: None,
                        order_hash: None,
                        tokens: vec![],
                    },
                    SgPaginationArgs {
                        page,
                        page_size: ALL_PAGES_QUERY_PAGE_SIZE,
                    },
                )
                .await?;
            if page_data.is_empty() {
                break;
            }
            all_pages_merged.extend(page_data);
            page += 1
        }
        Ok(all_pages_merged)
    }

    /// Fetch single order given its hash
    pub async fn order_detail_by_hash(
        &self,
        hash: SgBytes,
    ) -> Result<SgOrder, OrderbookSubgraphClientError> {
        let data = self
            .query::<SgOrderDetailByHashQuery, SgOrderDetailByHashQueryVariables>(
                SgOrderDetailByHashQueryVariables { hash },
            )
            .await?;
        let order = data
            .orders
            .first()
            .ok_or(OrderbookSubgraphClientError::Empty)?;
        Ok(order.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::common::{SgBigInt, SgBytes, SgOrder, SgOrderbook, SgOrdersListFilterArgs};
    use crate::utils::float::*;
    use cynic::Id;
    use httpmock::prelude::*;
    use rain_math_float::Float;
    use reqwest::Url;
    use serde_json::json;

    fn setup_client(server: &MockServer) -> OrderbookSubgraphClient {
        let url = Url::parse(&server.url("")).unwrap();
        OrderbookSubgraphClient::new(url)
    }

    fn default_sg_order() -> SgOrder {
        SgOrder {
            id: SgBytes("0x1a69eeb7970d3c8d5776493327fb262e31fc880c9cc4a951607418a7963d9fa1".to_string()),
            order_bytes: SgBytes("0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000f08bcbce72f62c95dcb7c07dcb5ed26acfcfbc1100000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000005c00000000000000000000000000000000000000000000000000000000000000640392c489ef67afdc348209452c338ea5ba2b6152b936e152f610d05e1a20621a40000000000000000000000005fb33d710f8b58de4c9fdec703b5c2487a5219d600000000000000000000000084c6e7f5a1e5dd89594cc25bef4722a1b8871ae60000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000049d000000000000000000000000000000000000000000000000000000000000000f0000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000c7d713b49da0000914d696e20747261646520616d6f756e742e00000000000000000000000000008b616d6f756e742d75736564000000000000000000000000000000000000000000000000000000000000000000000000000000000000000340aad21b3b70000000000000000000000000000000000000000000000000006194049f30f7200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b1a2bc2ec500000000000000000000000000000000000000000000000000000e043da6172500008f6c6173742d74726164652d74696d65000000000000000000000000000000008d6c6173742d74726164652d696f0000000000000000000000000000000000008c696e697469616c2d74696d650000000000000000000000000000000000000000000000000000000000000000000000000000000000000006f05b59d3b200000000000000000000000000000000000000000000000000008ac7230489e80000000000000000000000020000915e36ef882941816356bc3718df868054f868ad000000000000000000000000000000000000000000000000000000000000027d0a00000024007400e0015801b401e001f40218025c080500040b20000200100001001000000b120003001000010b110004001000030b0100051305000201100001011000003d120000011000020010000003100404211200001d02000001100003031000010c1200004911000003100404001000012b12000001100003031000010c1200004a0200001a0b00090b1000060b20000700100000001000011b1200001a10000047120000001000001a1000004712000001100000011000002e12000001100005011000042e120000001000053d12000001100004001000042e1200000010000601100005001000032e120000481200011d0b020a0010000001100000011000062713000001100003031000010c12000049110000001000030010000247120000001000010b110008001000050110000700100001201200001f12000001100000011000004712000000100006001000073d120000011000002b12000000100008001000043b120000160901080b1000070b10000901100008001000013d1200001b12000001100006001000013d1200000b100009001000033a120000001000040010000248120001001000000b110008001000053d12000000100006001000042b1200000a0401011a10000001100009031000010c1200004a020000001000000110000a031000010c1200004a020000040200010110000b031000010c120000491100000803000201100009031000010c120000491100000110000a031000010c12000049110000100c01030110000d001000002e1200000110000c3e1200000010000100100001001000010010000100100001001000010010000100100001001000013d1a0000020100010210000e3611000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000001d80c49bbbcd1c0911346656b529df9e5c2f783d0000000000000000000000000000000000000000000000000000000000000012a6e3c06415539f92823a18ba63e1c0303040c4892970a0d1e3a27663d7583b33000000000000000000000000000000000000000000000000000000000000000100000000000000000000000012e605bc104e93b45e1ad99f9e555f659051c2bb0000000000000000000000000000000000000000000000000000000000000012a6e3c06415539f92823a18ba63e1c0303040c4892970a0d1e3a27663d7583b33".to_string()),
            order_hash: SgBytes("0x557147dd0daa80d5beff0023fe6a3505469b2b8c4406ce1ab873e1a652572dd4".to_string()),
            owner: SgBytes("0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11".to_string()),
            outputs: vec![
                SgVault {
                    id: SgBytes("0x49f6b665c395c7b975caa2fc167cb5119981bbb86798bcaf3c4570153d09dfcf".to_string()),
                    owner: SgBytes("0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11".to_string()),
                    vault_id: SgBytes("75486334982066122983501547829219246999490818941767825330875804445439814023987".to_string()),
                    balance: SgBytes(Float::parse("0.987".to_string()).unwrap().get_inner().to_string()),
                    token: SgErc20 {
                        id: SgBytes("0x12e605bc104e93b45e1ad99f9e555f659051c2bb".to_string()),
                        address: SgBytes("0x12e605bc104e93b45e1ad99f9e555f659051c2bb".to_string()),
                        name: Some("Staked FLR".to_string()),
                        symbol: Some("sFLR".to_string()),
                        decimals: Some(SgBigInt("18".to_string())),
                    },
                    orderbook: SgOrderbook {
                        id: SgBytes("0xcee8cd002f151a536394e564b84076c41bbbcd4d".to_string()),
                    },
                    orders_as_output: vec![SgOrderAsIO {
                        id: SgBytes("0x1a69eeb7970d3c8d5776493327fb262e31fc880c9cc4a951607418a7963d9fa1".to_string()),
                        order_hash: SgBytes("0x557147dd0daa80d5beff0023fe6a3505469b2b8c4406ce1ab873e1a652572dd4".to_string()),
                        active: true,
                    }],
                    orders_as_input: vec![],
                    balance_changes: vec![],
                },
                SgVault {
                    id: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                    token: SgErc20 {
                        id: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                        address: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                        name: Some("T1".to_string()),
                        symbol: Some("T1".to_string()),
                        decimals: Some(SgBigInt("0".to_string())),
                    },
                    balance: SgBytes(F0.as_hex()),
                    vault_id: SgBytes("0".to_string()),
                    owner: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                    orders_as_output: vec![],
                    orders_as_input: vec![],
                    balance_changes: vec![],
                    orderbook: SgOrderbook {
                        id: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                    }
                }
            ],
            inputs: vec![
                SgVault {
                    id: SgBytes("0x538830b4f8cc03840cea5af799dc532be4363a3ee8f4c6123dbff7a0acc86dac".to_string()),
                    owner: SgBytes("0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11".to_string()),
                    vault_id: SgBytes("75486334982066122983501547829219246999490818941767825330875804445439814023987".to_string()),
                    balance: SgBytes(Float::parse("0.79799".to_string()).unwrap().get_inner().to_string()),
                    token: SgErc20 {
                        id: SgBytes("0x1d80c49bbbcd1c0911346656b529df9e5c2f783d".to_string()),
                        address: SgBytes("0x1d80c49bbbcd1c0911346656b529df9e5c2f783d".to_string()),
                        name: Some("Wrapped Flare".to_string()),
                        symbol: Some("WFLR".to_string()),
                        decimals: Some(SgBigInt("18".to_string())),
                    },
                    orderbook: SgOrderbook {
                        id: SgBytes("0xcee8cd002f151a536394e564b84076c41bbbcd4d".to_string()),
                    },
                    orders_as_output: vec![],
                    orders_as_input: vec![SgOrderAsIO {
                        id: SgBytes("0x1a69eeb7970d3c8d5776493327fb262e31fc880c9cc4a951607418a7963d9fa1".to_string()),
                        order_hash: SgBytes("0x557147dd0daa80d5beff0023fe6a3505469b2b8c4406ce1ab873e1a652572dd4".to_string()),
                        active: true,
                    }],
                    balance_changes: vec![],
                },
                SgVault {
                    id: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                    token: SgErc20 {
                        id: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                        address: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                        name: Some("T1".to_string()),
                        symbol: Some("T1".to_string()),
                        decimals: Some(SgBigInt("0".to_string())),
                    },
                    balance: SgBytes(F0.as_hex()),
                    vault_id: SgBytes("0".to_string()),
                    owner: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                    orders_as_output: vec![],
                    orders_as_input: vec![],
                    balance_changes: vec![],
                    orderbook: SgOrderbook {
                        id: SgBytes("0x0000000000000000000000000000000000000000".to_string()),
                    }
                }
            ],
            orderbook: SgOrderbook {
                id: SgBytes("0xcee8cd002f151a536394e564b84076c41bbbcd4d".to_string()),
            },
            active: true,
            timestamp_added: SgBigInt("1739448802".to_string()),
            meta: Some(SgBytes("0xff0a89c674ee7874a300590a932f2a20302e2063616c63756c6174652d696f202a2f200a7573696e672d776f7264732d66726f6d20307846653234313143446131393344394534653833413563323334433746643332303130313838336143203078393135453336656638383239343138313633353662433337313844663836383035344638363861440a616d6f756e742d65706f6368730a74726164652d65706f6368733a63616c6c3c323e28292c0a6d61782d6f75747075743a2063616c6c3c333e28616d6f756e742d65706f6368732074726164652d65706f636873292c0a696f3a2063616c6c3c343e2874726164652d65706f636873292c0a3a63616c6c3c353e28696f293b0a0a2f2a20312e2068616e646c652d696f202a2f200a6d696e2d616d6f756e743a206d756c283120302e39292c0a3a656e7375726528677265617465722d7468616e2d6f722d657175616c2d746f286f75747075742d7661756c742d64656372656173652829206d696e2d616d6f756e742920224d696e20747261646520616d6f756e742e22292c0a757365643a206765742868617368286f726465722d6861736828292022616d6f756e742d757365642229292c0a3a7365742868617368286f726465722d6861736828292022616d6f756e742d757365642229206164642875736564206f75747075742d7661756c742d6465637265617365282929293b0a0a2f2a20322e206765742d65706f6368202a2f200a696e697469616c2d74696d653a2063616c6c3c363e28292c0a6c6173742d74696d65205f3a2063616c6c3c373e28292c0a6475726174696f6e3a20737562286e6f77282920616e79286c6173742d74696d6520696e697469616c2d74696d652929292c0a746f74616c2d6475726174696f6e3a20737562286e6f77282920696e697469616c2d74696d65292c0a726174696f2d667265657a652d616d6f756e742d65706f6368733a2064697628312031292c0a726174696f2d667265657a652d74726164652d65706f6368733a206d756c28726174696f2d667265657a652d616d6f756e742d65706f63687320646976283630203138303029292c0a616d6f756e742d65706f6368733a2064697628746f74616c2d6475726174696f6e203630292c0a74726164652d65706f6368733a2073617475726174696e672d73756228646976286475726174696f6e20313830302920726174696f2d667265657a652d74726164652d65706f636873293b0a0a2f2a20332e20616d6f756e742d666f722d65706f6368202a2f200a616d6f756e742d65706f6368730a74726164652d65706f6368733a2c0a746f74616c2d617661696c61626c653a206c696e6561722d67726f7774682830203120616d6f756e742d65706f636873292c0a757365643a206765742868617368286f726465722d6861736828292022616d6f756e742d757365642229292c0a756e757365643a2073756228746f74616c2d617661696c61626c652075736564292c0a64656361793a2063616c6c3c383e2874726164652d65706f636873292c0a7368792d64656361793a20657665727928677265617465722d7468616e2874726164652d65706f63687320302e303529206465636179292c0a7661726961626c652d636f6d706f6e656e743a2073756228312031292c0a7461726765742d616d6f756e743a206164642831206d756c287661726961626c652d636f6d706f6e656e74207368792d646563617929292c0a6361707065642d756e757365643a206d696e28756e75736564207461726765742d616d6f756e74293b0a0a2f2a20342e20696f2d666f722d65706f6368202a2f200a65706f63683a2c0a6c6173742d696f3a2063616c6c3c373e28292c0a6d61782d6e6578742d74726164653a20616e79286d756c286c6173742d696f20312e3031292063616c6c3c393e2829292c0a626173656c696e652d6e6578742d74726164653a206d756c286c6173742d696f2030292c0a7265616c2d626173656c696e653a206d617828626173656c696e652d6e6578742d74726164652063616c6c3c393e2829292c0a7661726961626c652d636f6d706f6e656e743a2073617475726174696e672d737562286d61782d6e6578742d7472616465207265616c2d626173656c696e65292c0a61626f76652d626173656c696e653a206d756c287661726961626c652d636f6d706f6e656e742063616c6c3c383e2865706f636829292c0a5f3a20616464287265616c2d626173656c696e652061626f76652d626173656c696e65293b0a0a2f2a20352e207365742d6c6173742d7472616465202a2f200a6c6173742d696f3a2c0a3a7365742868617368286f726465722d68617368282920226c6173742d74726164652d74696d652229206e6f772829292c0a3a7365742868617368286f726465722d68617368282920226c6173742d74726164652d696f2229206c6173742d696f293b0a0a2f2a20362e206765742d696e697469616c2d74696d65202a2f200a5f3a6765742868617368286f726465722d6861736828292022696e697469616c2d74696d652229293b0a0a2f2a20372e206765742d6c6173742d7472616465202a2f200a6c6173742d74696d653a6765742868617368286f726465722d68617368282920226c6173742d74726164652d74696d652229292c0a6c6173742d696f3a6765742868617368286f726465722d68617368282920226c6173742d74726164652d696f2229293b0a0a2f2a20382e2068616c666c696665202a2f200a65706f63683a2c0a2f2a2a0a202a20536872696e6b696e6720746865206d756c7469706c696572206c696b6520746869730a202a207468656e206170706c79696e672069742031302074696d657320616c6c6f777320666f720a202a2062657474657220707265636973696f6e207768656e206d61782d696f2d726174696f0a202a2069732076657279206c617267652c20652e672e207e31653130206f72207e316532302b0a202a0a202a205468697320776f726b7320626563617573652060706f77657260206c6f7365730a202a20707265636973696f6e206f6e20626173652060302e3560207768656e207468650a202a206578706f6e656e74206973206c6172676520616e642063616e206576656e20676f0a202a20746f20603060207768696c652074686520696f2d726174696f206973207374696c6c0a202a206c617267652e2042657474657220746f206b65657020746865206d756c7469706c6965720a202a2068696768657220707265636973696f6e20616e642064726f702074686520696f2d726174696f0a202a20736d6f6f74686c7920666f72206173206c6f6e672061732077652063616e2e0a202a0a6d756c7469706c6965723a0a2020706f77657228302e35206469762865706f636820313029292c0a76616c3a0a20206d756c280a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a202020206d756c7469706c6965720a2020293b0a0a2f2a20392e2073666c722d626173656c696e652d696e76202a2f200a5f3a20696e762873666c722d65786368616e67652d726174652829293b011bff13109e41336ff20278186170706c69636174696f6e2f6f637465742d73747265616d".to_string())),
            add_events: vec![SgAddOrder {
                transaction: SgTransaction {
                    id: SgBytes("0xb5d715bc74b7a7f2aac8cca544c1c95e209ed4113b82269ac3285142324bc6af".to_string()),
                    from: SgBytes("0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11".to_string()),
                    block_number: SgBigInt("37432554".to_string()),
                    timestamp: SgBigInt("1739448802".to_string()),
                },
            }],
            trades: vec![],
            remove_events: vec![],
        }
    }

    fn assert_sg_order_eq(actual: &SgOrder, expected: &SgOrder) {
        assert_eq!(actual.id, expected.id);
        assert_eq!(actual.owner, expected.owner);
        assert_eq!(actual.active, expected.active);
        assert_eq!(actual.orderbook.id, expected.orderbook.id);
        assert_eq!(actual.inputs.len(), expected.inputs.len());
        for (actual_input, expected_input) in actual.inputs.iter().zip(expected.inputs.iter()) {
            assert_eq!(actual_input.id, expected_input.id);
            assert_eq!(actual_input.owner, expected_input.owner);
            assert_eq!(actual_input.vault_id, expected_input.vault_id);
            assert_eq!(actual_input.balance, expected_input.balance);
            assert_eq!(actual_input.token.id, expected_input.token.id);
            assert_eq!(actual_input.token.address, expected_input.token.address);
            assert_eq!(actual_input.token.name, expected_input.token.name);
            assert_eq!(actual_input.token.symbol, expected_input.token.symbol);
            assert_eq!(actual_input.token.decimals, expected_input.token.decimals);
            assert_eq!(actual_input.orderbook.id, expected_input.orderbook.id);
        }
        assert_eq!(actual.outputs.len(), expected.outputs.len());
        for (actual_output, expected_output) in actual.outputs.iter().zip(expected.outputs.iter()) {
            assert_eq!(actual_output.id, expected_output.id);
            assert_eq!(actual_output.owner, expected_output.owner);
            assert_eq!(actual_output.vault_id, expected_output.vault_id);
            assert_eq!(actual_output.balance, expected_output.balance);
            assert_eq!(actual_output.token.id, expected_output.token.id);
            assert_eq!(actual_output.token.address, expected_output.token.address);
            assert_eq!(actual_output.token.name, expected_output.token.name);
            assert_eq!(actual_output.token.symbol, expected_output.token.symbol);
            assert_eq!(actual_output.token.decimals, expected_output.token.decimals);
            assert_eq!(actual_output.orderbook.id, expected_output.orderbook.id);
        }
    }

    #[tokio::test]
    async fn test_order_detail_found() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let order_id_str = "0x123";
        let order_id = Id::new(order_id_str);
        let expected_order = default_sg_order();

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"order": expected_order}}));
        });

        let result = client.order_detail(&order_id).await;
        assert!(result.is_ok());
        let order = result.unwrap();
        assert_sg_order_eq(&order, &expected_order);
    }

    #[tokio::test]
    async fn test_order_detail_not_found() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let order_id_str = "0xnotfound";
        let order_id = Id::new(order_id_str);

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).json_body(json!({"data": {"order": null}}));
        });

        let result = client.order_detail(&order_id).await;
        assert!(matches!(result, Err(OrderbookSubgraphClientError::Empty)));
    }

    #[tokio::test]
    async fn test_order_detail_network_error() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let order_id = Id::new("0x123");

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });

        let result = client.order_detail(&order_id).await;
        assert!(matches!(
            result,
            Err(OrderbookSubgraphClientError::CynicClientError(_))
        ));
    }

    #[tokio::test]
    async fn test_batch_order_detail_all_found() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let id_list_str = ["id1".to_string(), "id2".to_string()];
        let id_list_sgbytes: Vec<SgBytes> =
            id_list_str.iter().map(|s| SgBytes(s.clone())).collect();
        let expected_orders = vec![default_sg_order(), default_sg_order()];

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"orders": expected_orders}}));
        });

        let result = client.batch_order_detail(id_list_sgbytes).await;
        assert!(result.is_ok());
        let orders = result.unwrap();
        assert_eq!(orders.len(), expected_orders.len());
        for (order, expected_order) in orders.iter().zip(expected_orders.iter()) {
            assert_sg_order_eq(order, expected_order);
        }
    }

    #[tokio::test]
    async fn test_batch_order_detail_some_found() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let id_list_str = ["id1".to_string(), "nonexistent".to_string()];
        let id_list_sgbytes: Vec<SgBytes> =
            id_list_str.iter().map(|s| SgBytes(s.clone())).collect();
        let expected_orders = vec![default_sg_order()]; // Only the found one

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"orders": expected_orders }}));
        });

        let result = client.batch_order_detail(id_list_sgbytes).await;
        assert!(result.is_ok());
        let orders = result.unwrap();
        assert_eq!(orders.len(), expected_orders.len());
        for (order, expected_order) in orders.iter().zip(expected_orders.iter()) {
            assert_sg_order_eq(order, expected_order);
        }
    }

    #[tokio::test]
    async fn test_batch_order_detail_none_found() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let id_list_str = ["non1".to_string(), "non2".to_string()];
        let id_list_sgbytes: Vec<SgBytes> =
            id_list_str.iter().map(|s| SgBytes(s.clone())).collect();

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).json_body(json!({"data": {"orders": []}}));
        });

        let result = client.batch_order_detail(id_list_sgbytes).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_batch_order_detail_empty_input_list() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let id_list_sgbytes: Vec<SgBytes> = vec![];

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).json_body(json!({"data": {"orders": []}}));
        });

        let result = client.batch_order_detail(id_list_sgbytes).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_batch_order_detail_network_error() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let id_list_str = ["id1".to_string()];
        let id_list_sgbytes: Vec<SgBytes> =
            id_list_str.iter().map(|s| SgBytes(s.clone())).collect();

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });

        let result = client.batch_order_detail(id_list_sgbytes).await;
        assert!(matches!(
            result,
            Err(OrderbookSubgraphClientError::CynicClientError(_))
        ));
    }

    #[tokio::test]
    async fn test_orders_list_no_filters() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let filter_args = SgOrdersListFilterArgs {
            // Manual instantiation
            owners: vec![],
            active: None,
            order_hash: None,
            tokens: vec![],
        };
        let pagination_args = SgPaginationArgs {
            page: 1,
            page_size: 10,
        };
        let expected_orders = vec![default_sg_order()];

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"orders": expected_orders}}));
        });

        let result = client.orders_list(filter_args, pagination_args).await;
        assert!(result.is_ok());
        let orders = result.unwrap();
        assert_eq!(orders.len(), expected_orders.len());
        for (order, expected_order) in orders.iter().zip(expected_orders.iter()) {
            assert_sg_order_eq(order, expected_order);
        }
    }

    #[tokio::test]
    async fn test_orders_list_with_filters() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let owner_address_str = "owner1";
        let owner_address_sg = SgBytes(owner_address_str.to_string());
        let filter_args = SgOrdersListFilterArgs {
            owners: vec![owner_address_sg.clone()],
            active: Some(true),
            order_hash: None,
            tokens: vec![],
        };
        let pagination_args = SgPaginationArgs {
            page: 1,
            page_size: 5,
        };
        let expected_orders = vec![default_sg_order()];

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"orders": expected_orders}}));
        });

        let result = client.orders_list(filter_args, pagination_args).await;
        assert!(result.is_ok());
        let orders = result.unwrap();
        assert_eq!(orders.len(), expected_orders.len());
        for (order, expected_order) in orders.iter().zip(expected_orders.iter()) {
            assert_sg_order_eq(order, expected_order);
        }
    }

    #[tokio::test]
    async fn test_orders_list_empty_result() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let filter_args = SgOrdersListFilterArgs {
            owners: vec![],
            active: None,
            order_hash: None,
            tokens: vec![],
        };
        let pagination_args = SgPaginationArgs {
            page: 1,
            page_size: 10,
        };

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).json_body(json!({"data": {"orders": []}}));
        });

        let result = client.orders_list(filter_args, pagination_args).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_orders_list_network_error() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let filter_args = SgOrdersListFilterArgs {
            owners: vec![],
            active: None,
            order_hash: None,
            tokens: vec![],
        };
        let pagination_args = SgPaginationArgs {
            page: 1,
            page_size: 10,
        };

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });

        let result = client.orders_list(filter_args, pagination_args).await;
        assert!(matches!(
            result,
            Err(OrderbookSubgraphClientError::CynicClientError(_))
        ));
    }

    #[tokio::test]
    async fn test_orders_list_all_multiple_pages() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let orders_page1: Vec<SgOrder> = (0..ALL_PAGES_QUERY_PAGE_SIZE)
            .map(|_| default_sg_order())
            .collect();
        let orders_page2: Vec<SgOrder> = (0..50).map(|_| default_sg_order()).collect();

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains("\"first\":200")
                .body_contains("\"skip\":0");
            then.status(200)
                .json_body(json!({"data": {"orders": orders_page1}}));
        });
        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains("\"first\":200")
                .body_contains("\"skip\":200");
            then.status(200)
                .json_body(json!({"data": {"orders": orders_page2}}));
        });
        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains("\"first\":200")
                .body_contains("\"skip\":400");
            then.status(200).json_body(json!({"data": {"orders": []}}));
        });

        let result = client.orders_list_all().await;
        assert!(result.is_ok());
        let orders = result.unwrap();
        assert_eq!(orders.len(), ALL_PAGES_QUERY_PAGE_SIZE as usize + 50);
    }

    #[tokio::test]
    async fn test_orders_list_all_no_orders() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).json_body(json!({"data": {"orders": []}}));
        });

        let result = client.orders_list_all().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_orders_list_all_network_error_on_page() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let orders_page1: Vec<SgOrder> = (0..ALL_PAGES_QUERY_PAGE_SIZE)
            .map(|_| default_sg_order())
            .collect();

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains("\"first\":200")
                .body_contains("\"skip\":0");
            then.status(200)
                .json_body(json!({"data": {"orders": orders_page1}}));
        });
        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains("\"first\":200")
                .body_contains("\"skip\":200");
            then.status(500);
        });

        let result = client.orders_list_all().await;
        assert!(matches!(
            result,
            Err(OrderbookSubgraphClientError::CynicClientError(_))
        ));
    }

    #[tokio::test]
    async fn test_order_detail_by_hash_found() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let order_hash_str = "0xhash123";
        let order_hash_sg = SgBytes(order_hash_str.to_string());
        let expected_order = default_sg_order();

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"orders": [expected_order.clone()]}}));
        });

        let result = client.order_detail_by_hash(order_hash_sg).await;
        assert!(result.is_ok());
        let order = result.unwrap();
        assert_sg_order_eq(&order, &expected_order);
    }

    #[tokio::test]
    async fn test_order_detail_by_hash_not_found() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let order_hash_str = "0xnonexistenthash";
        let order_hash_sg = SgBytes(order_hash_str.to_string());

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).json_body(json!({"data": {"orders": []}}));
        });

        let result = client.order_detail_by_hash(order_hash_sg).await;
        assert!(matches!(result, Err(OrderbookSubgraphClientError::Empty)));
    }

    #[tokio::test]
    async fn test_order_detail_by_hash_network_error() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let order_hash_str = "0xhash123";
        let order_hash_sg = SgBytes(order_hash_str.to_string());

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });

        let result = client.order_detail_by_hash(order_hash_sg).await;
        assert!(matches!(
            result,
            Err(OrderbookSubgraphClientError::CynicClientError(_))
        ));
    }

    #[tokio::test]
    async fn test_orders_list_with_token_filter() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let token_address = "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913".to_string();
        let filter_args = SgOrdersListFilterArgs {
            owners: vec![],
            active: None,
            order_hash: None,
            tokens: vec![token_address.clone()],
        };
        let pagination_args = SgPaginationArgs {
            page: 1,
            page_size: 10,
        };
        let expected_orders = vec![default_sg_order()];

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(format!("\"token_in\":[\"{}\"]", token_address))
                .body_contains("\"inputs_\":")
                .body_contains("\"outputs_\":");
            then.status(200)
                .json_body(json!({"data": {"orders": expected_orders}}));
        });

        let result = client.orders_list(filter_args, pagination_args).await;
        assert!(result.is_ok());
        let orders = result.unwrap();
        assert_eq!(orders.len(), expected_orders.len());
    }

    #[tokio::test]
    async fn test_orders_list_with_multiple_token_filters() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let token1 = "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d".to_string();
        let token2 = "0x12e605bc104e93b45e1ad99f9e555f659051c2bb".to_string();
        let filter_args = SgOrdersListFilterArgs {
            owners: vec![],
            active: None,
            order_hash: None,
            tokens: vec![token1.clone(), token2.clone()],
        };
        let pagination_args = SgPaginationArgs {
            page: 1,
            page_size: 10,
        };
        let expected_orders = vec![default_sg_order(), default_sg_order()];

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(format!("\"token_in\":[\"{}\",\"{}\"]", token1, token2))
                .body_contains("\"inputs_\":")
                .body_contains("\"outputs_\":");
            then.status(200)
                .json_body(json!({"data": {"orders": expected_orders}}));
        });

        let result = client.orders_list(filter_args, pagination_args).await;
        assert!(result.is_ok());
        let orders = result.unwrap();
        assert_eq!(orders.len(), expected_orders.len());
    }

    #[tokio::test]
    async fn test_orders_list_with_combined_filters() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let token_address = "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913".to_string();
        let owner_address = SgBytes("0xowner123".to_string());
        let filter_args = SgOrdersListFilterArgs {
            owners: vec![owner_address.clone()],
            active: Some(true),
            order_hash: None,
            tokens: vec![token_address.clone()],
        };
        let pagination_args = SgPaginationArgs {
            page: 1,
            page_size: 5,
        };
        let expected_orders = vec![default_sg_order()];

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains("\"owner_in\":[\"0xowner123\"]")
                .body_contains("\"active\":true")
                .body_contains(format!("\"token_in\":[\"{}\"]", token_address))
                .body_contains("\"inputs_\":")
                .body_contains("\"outputs_\":");
            then.status(200)
                .json_body(json!({"data": {"orders": expected_orders}}));
        });

        let result = client.orders_list(filter_args, pagination_args).await;
        assert!(result.is_ok());
        let orders = result.unwrap();
        assert_eq!(orders.len(), expected_orders.len());
    }
}
