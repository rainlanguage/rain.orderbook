pub use rainterpreter_expression_deployer_np::*;
/// This module was auto-generated with ethers-rs Abigen.
/// More information at: <https://github.com/gakonst/ethers-rs>
#[allow(
    clippy::enum_variant_names,
    clippy::too_many_arguments,
    clippy::upper_case_acronyms,
    clippy::type_complexity,
    dead_code,
    non_camel_case_types,
)]
pub mod rainterpreter_expression_deployer_np {
    const _: () = {
        ::core::include_bytes!(
            "/home/nanezx/rain/rain.orderbook/subgraph/tests/utils/deploy/touch_deployer/RainterpreterExpressionDeployer_ABI.json",
        );
    };
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::Some(::ethers::core::abi::ethabi::Constructor {
                inputs: ::std::vec![
                    ::ethers::core::abi::ethabi::Param {
                        name: ::std::borrow::ToOwned::to_owned("config_"),
                        kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                            ::std::vec![
                                ::ethers::core::abi::ethabi::ParamType::Address,
                                ::ethers::core::abi::ethabi::ParamType::Address,
                                ::ethers::core::abi::ethabi::ParamType::Bytes,
                            ],
                        ),
                        internal_type: ::core::option::Option::Some(
                            ::std::borrow::ToOwned::to_owned(
                                "struct RainterpreterExpressionDeployerConstructionConfig",
                            ),
                        ),
                    },
                ],
            }),
            functions: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("deployExpression"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("deployExpression"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("sources_"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes[]"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("constants_"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256[]"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("minOutputs_"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256[]"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("contract IInterpreterV1"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "contract IInterpreterStoreV1",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("interpreter"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("interpreter"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("contract IInterpreterV1"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("store"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("store"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "contract IInterpreterStoreV1",
                                        ),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("supportsInterface"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("supportsInterface"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("interfaceId_"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
            ]),
            events: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("DISpair"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("DISpair"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("sender"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("deployer"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("interpreter"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("store"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("opMeta"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ExpressionAddress"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("ExpressionAddress"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("sender"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("expression"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NewExpression"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("NewExpression"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("sender"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("sources"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                        ),
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("constants"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                        ),
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("minOutputs"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                        ),
                                    ),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
            ]),
            errors: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("BadDynamicLength"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("BadDynamicLength"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("dynamicLength"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("standardOpsLength"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("DoWhileMaxInputs"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("DoWhileMaxInputs"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("inputs"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InsufficientLoopOutputs"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InsufficientLoopOutputs",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("inputs"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("outputs"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("MinFinalStack"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("MinFinalStack"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("minStackOutputs"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "actualStackOutputs",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("MinStackBottom"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("MinStackBottom"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("MissingEntrypoint"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("MissingEntrypoint"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "expectedEntrypoints",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("actualEntrypoints"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NotPosIntPrice"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("NotPosIntPrice"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("price"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Int(256usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("int256"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OutOfBoundsConstantsRead"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "OutOfBoundsConstantsRead",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("constantsLength"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("constantsRead"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OutOfBoundsStackRead"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "OutOfBoundsStackRead",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("stackTopIndex"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("stackRead"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("PRBMath_MulDiv18_Overflow"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "PRBMath_MulDiv18_Overflow",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("x"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("y"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("PRBMath_MulDiv_Overflow"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "PRBMath_MulDiv_Overflow",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("x"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("y"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("denominator"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("PRBMath_UD60x18_Ceil_Overflow"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "PRBMath_UD60x18_Ceil_Overflow",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("x"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("UD60x18"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("PRBMath_UD60x18_Exp2_InputTooBig"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "PRBMath_UD60x18_Exp2_InputTooBig",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("x"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("UD60x18"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("PRBMath_UD60x18_Exp_InputTooBig"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "PRBMath_UD60x18_Exp_InputTooBig",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("x"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("UD60x18"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("PRBMath_UD60x18_Gm_Overflow"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "PRBMath_UD60x18_Gm_Overflow",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("x"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("UD60x18"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("y"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("UD60x18"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "PRBMath_UD60x18_Log_InputTooSmall",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "PRBMath_UD60x18_Log_InputTooSmall",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("x"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("UD60x18"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("PRBMath_UD60x18_Sqrt_Overflow"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "PRBMath_UD60x18_Sqrt_Overflow",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("x"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("UD60x18"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("StackPopUnderflow"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("StackPopUnderflow"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "stackHighwaterIndex",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("stackTopIndex"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("StalePrice"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("StalePrice"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("updatedAt"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("staleAfter"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("TruncatedEncoding"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("TruncatedEncoding"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("startBit"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("length"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "UnexpectedInterpreterBytecodeHash",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "UnexpectedInterpreterBytecodeHash",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "actualBytecodeHash",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("UnexpectedOpMetaHash"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "UnexpectedOpMetaHash",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("actualOpMeta"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("UnexpectedPointers"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("UnexpectedPointers"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("actualPointers"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("UnexpectedStoreBytecodeHash"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "UnexpectedStoreBytecodeHash",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "actualBytecodeHash",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("WriteError"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("WriteError"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ZeroInputs"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("ZeroInputs"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
            ]),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static RAINTERPRETEREXPRESSIONDEPLOYERNP_ABI: ::ethers::contract::Lazy<
        ::ethers::core::abi::Abi,
    > = ::ethers::contract::Lazy::new(__abi);
    pub struct RainterpreterExpressionDeployerNP<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for RainterpreterExpressionDeployerNP<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for RainterpreterExpressionDeployerNP<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for RainterpreterExpressionDeployerNP<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for RainterpreterExpressionDeployerNP<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(RainterpreterExpressionDeployerNP))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> RainterpreterExpressionDeployerNP<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    RAINTERPRETEREXPRESSIONDEPLOYERNP_ABI.clone(),
                    client,
                ),
            )
        }
        ///Calls the contract's `deployExpression` (0x5511cb67) function
        pub fn deploy_expression(
            &self,
            sources: ::std::vec::Vec<::ethers::core::types::Bytes>,
            constants: ::std::vec::Vec<::ethers::core::types::U256>,
            min_outputs: ::std::vec::Vec<::ethers::core::types::U256>,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (
                ::ethers::core::types::Address,
                ::ethers::core::types::Address,
                ::ethers::core::types::Address,
            ),
        > {
            self.0
                .method_hash([85, 17, 203, 103], (sources, constants, min_outputs))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `interpreter` (0x3a35cf17) function
        pub fn interpreter(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([58, 53, 207, 23], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `store` (0x975057e7) function
        pub fn store(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([151, 80, 87, 231], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `supportsInterface` (0x01ffc9a7) function
        pub fn supports_interface(
            &self,
            interface_id: [u8; 4],
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([1, 255, 201, 167], interface_id)
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `DISpair` event
        pub fn di_spair_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, DispairFilter> {
            self.0.event()
        }
        ///Gets the contract's `ExpressionAddress` event
        pub fn expression_address_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ExpressionAddressFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `NewExpression` event
        pub fn new_expression_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            NewExpressionFilter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            RainterpreterExpressionDeployerNPEvents,
        > {
            self.0.event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for RainterpreterExpressionDeployerNP<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `BadDynamicLength` with signature `BadDynamicLength(uint256,uint256)` and selector `0xc8b56901`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "BadDynamicLength", abi = "BadDynamicLength(uint256,uint256)")]
    pub struct BadDynamicLength {
        pub dynamic_length: ::ethers::core::types::U256,
        pub standard_ops_length: ::ethers::core::types::U256,
    }
    ///Custom Error type `DoWhileMaxInputs` with signature `DoWhileMaxInputs(uint256)` and selector `0x316e6a37`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "DoWhileMaxInputs", abi = "DoWhileMaxInputs(uint256)")]
    pub struct DoWhileMaxInputs {
        pub inputs: ::ethers::core::types::U256,
    }
    ///Custom Error type `InsufficientLoopOutputs` with signature `InsufficientLoopOutputs(uint256,uint256)` and selector `0x508a8d2f`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "InsufficientLoopOutputs",
        abi = "InsufficientLoopOutputs(uint256,uint256)"
    )]
    pub struct InsufficientLoopOutputs {
        pub inputs: ::ethers::core::types::U256,
        pub outputs: ::ethers::core::types::U256,
    }
    ///Custom Error type `MinFinalStack` with signature `MinFinalStack(uint256,uint256)` and selector `0xf993c6e7`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "MinFinalStack", abi = "MinFinalStack(uint256,uint256)")]
    pub struct MinFinalStack {
        pub min_stack_outputs: ::ethers::core::types::U256,
        pub actual_stack_outputs: ::ethers::core::types::U256,
    }
    ///Custom Error type `MinStackBottom` with signature `MinStackBottom()` and selector `0x271592cf`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "MinStackBottom", abi = "MinStackBottom()")]
    pub struct MinStackBottom;
    ///Custom Error type `MissingEntrypoint` with signature `MissingEntrypoint(uint256,uint256)` and selector `0x7d2d70db`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "MissingEntrypoint", abi = "MissingEntrypoint(uint256,uint256)")]
    pub struct MissingEntrypoint {
        pub expected_entrypoints: ::ethers::core::types::U256,
        pub actual_entrypoints: ::ethers::core::types::U256,
    }
    ///Custom Error type `NotPosIntPrice` with signature `NotPosIntPrice(int256)` and selector `0x3351e26f`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "NotPosIntPrice", abi = "NotPosIntPrice(int256)")]
    pub struct NotPosIntPrice {
        pub price: ::ethers::core::types::I256,
    }
    ///Custom Error type `OutOfBoundsConstantsRead` with signature `OutOfBoundsConstantsRead(uint256,uint256)` and selector `0x890a8e6a`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "OutOfBoundsConstantsRead",
        abi = "OutOfBoundsConstantsRead(uint256,uint256)"
    )]
    pub struct OutOfBoundsConstantsRead {
        pub constants_length: ::ethers::core::types::U256,
        pub constants_read: ::ethers::core::types::U256,
    }
    ///Custom Error type `OutOfBoundsStackRead` with signature `OutOfBoundsStackRead(uint256,uint256)` and selector `0x1cb73c16`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "OutOfBoundsStackRead",
        abi = "OutOfBoundsStackRead(uint256,uint256)"
    )]
    pub struct OutOfBoundsStackRead {
        pub stack_top_index: ::ethers::core::types::U256,
        pub stack_read: ::ethers::core::types::U256,
    }
    ///Custom Error type `PRBMath_MulDiv18_Overflow` with signature `PRBMath_MulDiv18_Overflow(uint256,uint256)` and selector `0x5173648d`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "PRBMath_MulDiv18_Overflow",
        abi = "PRBMath_MulDiv18_Overflow(uint256,uint256)"
    )]
    pub struct PRBMath_MulDiv18_Overflow {
        pub x: ::ethers::core::types::U256,
        pub y: ::ethers::core::types::U256,
    }
    ///Custom Error type `PRBMath_MulDiv_Overflow` with signature `PRBMath_MulDiv_Overflow(uint256,uint256,uint256)` and selector `0x63a05778`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "PRBMath_MulDiv_Overflow",
        abi = "PRBMath_MulDiv_Overflow(uint256,uint256,uint256)"
    )]
    pub struct PRBMath_MulDiv_Overflow {
        pub x: ::ethers::core::types::U256,
        pub y: ::ethers::core::types::U256,
        pub denominator: ::ethers::core::types::U256,
    }
    ///Custom Error type `PRBMath_UD60x18_Ceil_Overflow` with signature `PRBMath_UD60x18_Ceil_Overflow(uint256)` and selector `0x6149f6b9`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "PRBMath_UD60x18_Ceil_Overflow",
        abi = "PRBMath_UD60x18_Ceil_Overflow(uint256)"
    )]
    pub struct PRBMath_UD60x18_Ceil_Overflow {
        pub x: ::ethers::core::types::U256,
    }
    ///Custom Error type `PRBMath_UD60x18_Exp2_InputTooBig` with signature `PRBMath_UD60x18_Exp2_InputTooBig(uint256)` and selector `0xb3b6ba1f`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "PRBMath_UD60x18_Exp2_InputTooBig",
        abi = "PRBMath_UD60x18_Exp2_InputTooBig(uint256)"
    )]
    pub struct PRBMath_UD60x18_Exp2_InputTooBig {
        pub x: ::ethers::core::types::U256,
    }
    ///Custom Error type `PRBMath_UD60x18_Exp_InputTooBig` with signature `PRBMath_UD60x18_Exp_InputTooBig(uint256)` and selector `0x1af63aca`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "PRBMath_UD60x18_Exp_InputTooBig",
        abi = "PRBMath_UD60x18_Exp_InputTooBig(uint256)"
    )]
    pub struct PRBMath_UD60x18_Exp_InputTooBig {
        pub x: ::ethers::core::types::U256,
    }
    ///Custom Error type `PRBMath_UD60x18_Gm_Overflow` with signature `PRBMath_UD60x18_Gm_Overflow(uint256,uint256)` and selector `0xae7f3b37`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "PRBMath_UD60x18_Gm_Overflow",
        abi = "PRBMath_UD60x18_Gm_Overflow(uint256,uint256)"
    )]
    pub struct PRBMath_UD60x18_Gm_Overflow {
        pub x: ::ethers::core::types::U256,
        pub y: ::ethers::core::types::U256,
    }
    ///Custom Error type `PRBMath_UD60x18_Log_InputTooSmall` with signature `PRBMath_UD60x18_Log_InputTooSmall(uint256)` and selector `0x36d32ef0`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "PRBMath_UD60x18_Log_InputTooSmall",
        abi = "PRBMath_UD60x18_Log_InputTooSmall(uint256)"
    )]
    pub struct PRBMath_UD60x18_Log_InputTooSmall {
        pub x: ::ethers::core::types::U256,
    }
    ///Custom Error type `PRBMath_UD60x18_Sqrt_Overflow` with signature `PRBMath_UD60x18_Sqrt_Overflow(uint256)` and selector `0xedc236ad`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "PRBMath_UD60x18_Sqrt_Overflow",
        abi = "PRBMath_UD60x18_Sqrt_Overflow(uint256)"
    )]
    pub struct PRBMath_UD60x18_Sqrt_Overflow {
        pub x: ::ethers::core::types::U256,
    }
    ///Custom Error type `StackPopUnderflow` with signature `StackPopUnderflow(uint256,uint256)` and selector `0x625e32e4`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "StackPopUnderflow", abi = "StackPopUnderflow(uint256,uint256)")]
    pub struct StackPopUnderflow {
        pub stack_highwater_index: ::ethers::core::types::U256,
        pub stack_top_index: ::ethers::core::types::U256,
    }
    ///Custom Error type `StalePrice` with signature `StalePrice(uint256,uint256)` and selector `0x2730eb48`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "StalePrice", abi = "StalePrice(uint256,uint256)")]
    pub struct StalePrice {
        pub updated_at: ::ethers::core::types::U256,
        pub stale_after: ::ethers::core::types::U256,
    }
    ///Custom Error type `TruncatedEncoding` with signature `TruncatedEncoding(uint256,uint256)` and selector `0x2ccabbc2`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "TruncatedEncoding", abi = "TruncatedEncoding(uint256,uint256)")]
    pub struct TruncatedEncoding {
        pub start_bit: ::ethers::core::types::U256,
        pub length: ::ethers::core::types::U256,
    }
    ///Custom Error type `UnexpectedInterpreterBytecodeHash` with signature `UnexpectedInterpreterBytecodeHash(bytes32)` and selector `0x1dd8527e`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "UnexpectedInterpreterBytecodeHash",
        abi = "UnexpectedInterpreterBytecodeHash(bytes32)"
    )]
    pub struct UnexpectedInterpreterBytecodeHash {
        pub actual_bytecode_hash: [u8; 32],
    }
    ///Custom Error type `UnexpectedOpMetaHash` with signature `UnexpectedOpMetaHash(bytes32)` and selector `0x87a1fcae`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "UnexpectedOpMetaHash", abi = "UnexpectedOpMetaHash(bytes32)")]
    pub struct UnexpectedOpMetaHash {
        pub actual_op_meta: [u8; 32],
    }
    ///Custom Error type `UnexpectedPointers` with signature `UnexpectedPointers(bytes)` and selector `0x9835e402`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "UnexpectedPointers", abi = "UnexpectedPointers(bytes)")]
    pub struct UnexpectedPointers {
        pub actual_pointers: ::ethers::core::types::Bytes,
    }
    ///Custom Error type `UnexpectedStoreBytecodeHash` with signature `UnexpectedStoreBytecodeHash(bytes32)` and selector `0xcc0415fd`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "UnexpectedStoreBytecodeHash",
        abi = "UnexpectedStoreBytecodeHash(bytes32)"
    )]
    pub struct UnexpectedStoreBytecodeHash {
        pub actual_bytecode_hash: [u8; 32],
    }
    ///Custom Error type `WriteError` with signature `WriteError()` and selector `0x08d4abb6`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "WriteError", abi = "WriteError()")]
    pub struct WriteError;
    ///Custom Error type `ZeroInputs` with signature `ZeroInputs()` and selector `0x904c1f6d`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "ZeroInputs", abi = "ZeroInputs()")]
    pub struct ZeroInputs;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum RainterpreterExpressionDeployerNPErrors {
        BadDynamicLength(BadDynamicLength),
        DoWhileMaxInputs(DoWhileMaxInputs),
        InsufficientLoopOutputs(InsufficientLoopOutputs),
        MinFinalStack(MinFinalStack),
        MinStackBottom(MinStackBottom),
        MissingEntrypoint(MissingEntrypoint),
        NotPosIntPrice(NotPosIntPrice),
        OutOfBoundsConstantsRead(OutOfBoundsConstantsRead),
        OutOfBoundsStackRead(OutOfBoundsStackRead),
        PRBMath_MulDiv18_Overflow(PRBMath_MulDiv18_Overflow),
        PRBMath_MulDiv_Overflow(PRBMath_MulDiv_Overflow),
        PRBMath_UD60x18_Ceil_Overflow(PRBMath_UD60x18_Ceil_Overflow),
        PRBMath_UD60x18_Exp2_InputTooBig(PRBMath_UD60x18_Exp2_InputTooBig),
        PRBMath_UD60x18_Exp_InputTooBig(PRBMath_UD60x18_Exp_InputTooBig),
        PRBMath_UD60x18_Gm_Overflow(PRBMath_UD60x18_Gm_Overflow),
        PRBMath_UD60x18_Log_InputTooSmall(PRBMath_UD60x18_Log_InputTooSmall),
        PRBMath_UD60x18_Sqrt_Overflow(PRBMath_UD60x18_Sqrt_Overflow),
        StackPopUnderflow(StackPopUnderflow),
        StalePrice(StalePrice),
        TruncatedEncoding(TruncatedEncoding),
        UnexpectedInterpreterBytecodeHash(UnexpectedInterpreterBytecodeHash),
        UnexpectedOpMetaHash(UnexpectedOpMetaHash),
        UnexpectedPointers(UnexpectedPointers),
        UnexpectedStoreBytecodeHash(UnexpectedStoreBytecodeHash),
        WriteError(WriteError),
        ZeroInputs(ZeroInputs),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for RainterpreterExpressionDeployerNPErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded) = <BadDynamicLength as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::BadDynamicLength(decoded));
            }
            if let Ok(decoded) = <DoWhileMaxInputs as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::DoWhileMaxInputs(decoded));
            }
            if let Ok(decoded) = <InsufficientLoopOutputs as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InsufficientLoopOutputs(decoded));
            }
            if let Ok(decoded) = <MinFinalStack as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::MinFinalStack(decoded));
            }
            if let Ok(decoded) = <MinStackBottom as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::MinStackBottom(decoded));
            }
            if let Ok(decoded) = <MissingEntrypoint as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::MissingEntrypoint(decoded));
            }
            if let Ok(decoded) = <NotPosIntPrice as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NotPosIntPrice(decoded));
            }
            if let Ok(decoded) = <OutOfBoundsConstantsRead as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OutOfBoundsConstantsRead(decoded));
            }
            if let Ok(decoded) = <OutOfBoundsStackRead as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OutOfBoundsStackRead(decoded));
            }
            if let Ok(decoded) = <PRBMath_MulDiv18_Overflow as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PRBMath_MulDiv18_Overflow(decoded));
            }
            if let Ok(decoded) = <PRBMath_MulDiv_Overflow as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PRBMath_MulDiv_Overflow(decoded));
            }
            if let Ok(decoded) = <PRBMath_UD60x18_Ceil_Overflow as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PRBMath_UD60x18_Ceil_Overflow(decoded));
            }
            if let Ok(decoded) = <PRBMath_UD60x18_Exp2_InputTooBig as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PRBMath_UD60x18_Exp2_InputTooBig(decoded));
            }
            if let Ok(decoded) = <PRBMath_UD60x18_Exp_InputTooBig as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PRBMath_UD60x18_Exp_InputTooBig(decoded));
            }
            if let Ok(decoded) = <PRBMath_UD60x18_Gm_Overflow as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PRBMath_UD60x18_Gm_Overflow(decoded));
            }
            if let Ok(decoded) = <PRBMath_UD60x18_Log_InputTooSmall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PRBMath_UD60x18_Log_InputTooSmall(decoded));
            }
            if let Ok(decoded) = <PRBMath_UD60x18_Sqrt_Overflow as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PRBMath_UD60x18_Sqrt_Overflow(decoded));
            }
            if let Ok(decoded) = <StackPopUnderflow as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::StackPopUnderflow(decoded));
            }
            if let Ok(decoded) = <StalePrice as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::StalePrice(decoded));
            }
            if let Ok(decoded) = <TruncatedEncoding as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::TruncatedEncoding(decoded));
            }
            if let Ok(decoded) = <UnexpectedInterpreterBytecodeHash as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::UnexpectedInterpreterBytecodeHash(decoded));
            }
            if let Ok(decoded) = <UnexpectedOpMetaHash as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::UnexpectedOpMetaHash(decoded));
            }
            if let Ok(decoded) = <UnexpectedPointers as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::UnexpectedPointers(decoded));
            }
            if let Ok(decoded) = <UnexpectedStoreBytecodeHash as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::UnexpectedStoreBytecodeHash(decoded));
            }
            if let Ok(decoded) = <WriteError as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::WriteError(decoded));
            }
            if let Ok(decoded) = <ZeroInputs as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ZeroInputs(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for RainterpreterExpressionDeployerNPErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::BadDynamicLength(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::DoWhileMaxInputs(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InsufficientLoopOutputs(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MinFinalStack(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MinStackBottom(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MissingEntrypoint(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotPosIntPrice(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OutOfBoundsConstantsRead(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OutOfBoundsStackRead(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PRBMath_MulDiv18_Overflow(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PRBMath_MulDiv_Overflow(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PRBMath_UD60x18_Ceil_Overflow(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PRBMath_UD60x18_Exp2_InputTooBig(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PRBMath_UD60x18_Exp_InputTooBig(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PRBMath_UD60x18_Gm_Overflow(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PRBMath_UD60x18_Log_InputTooSmall(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PRBMath_UD60x18_Sqrt_Overflow(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::StackPopUnderflow(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::StalePrice(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TruncatedEncoding(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::UnexpectedInterpreterBytecodeHash(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::UnexpectedOpMetaHash(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::UnexpectedPointers(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::UnexpectedStoreBytecodeHash(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::WriteError(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ZeroInputs(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for RainterpreterExpressionDeployerNPErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <BadDynamicLength as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <DoWhileMaxInputs as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InsufficientLoopOutputs as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <MinFinalStack as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <MinStackBottom as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <MissingEntrypoint as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotPosIntPrice as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OutOfBoundsConstantsRead as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OutOfBoundsStackRead as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <PRBMath_MulDiv18_Overflow as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <PRBMath_MulDiv_Overflow as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <PRBMath_UD60x18_Ceil_Overflow as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <PRBMath_UD60x18_Exp2_InputTooBig as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <PRBMath_UD60x18_Exp_InputTooBig as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <PRBMath_UD60x18_Gm_Overflow as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <PRBMath_UD60x18_Log_InputTooSmall as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <PRBMath_UD60x18_Sqrt_Overflow as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <StackPopUnderflow as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <StalePrice as ::ethers::contract::EthError>::selector() => true,
                _ if selector
                    == <TruncatedEncoding as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <UnexpectedInterpreterBytecodeHash as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <UnexpectedOpMetaHash as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <UnexpectedPointers as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <UnexpectedStoreBytecodeHash as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <WriteError as ::ethers::contract::EthError>::selector() => true,
                _ if selector
                    == <ZeroInputs as ::ethers::contract::EthError>::selector() => true,
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for RainterpreterExpressionDeployerNPErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::BadDynamicLength(element) => ::core::fmt::Display::fmt(element, f),
                Self::DoWhileMaxInputs(element) => ::core::fmt::Display::fmt(element, f),
                Self::InsufficientLoopOutputs(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::MinFinalStack(element) => ::core::fmt::Display::fmt(element, f),
                Self::MinStackBottom(element) => ::core::fmt::Display::fmt(element, f),
                Self::MissingEntrypoint(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotPosIntPrice(element) => ::core::fmt::Display::fmt(element, f),
                Self::OutOfBoundsConstantsRead(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OutOfBoundsStackRead(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PRBMath_MulDiv18_Overflow(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PRBMath_MulDiv_Overflow(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PRBMath_UD60x18_Ceil_Overflow(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PRBMath_UD60x18_Exp2_InputTooBig(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PRBMath_UD60x18_Exp_InputTooBig(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PRBMath_UD60x18_Gm_Overflow(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PRBMath_UD60x18_Log_InputTooSmall(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PRBMath_UD60x18_Sqrt_Overflow(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::StackPopUnderflow(element) => ::core::fmt::Display::fmt(element, f),
                Self::StalePrice(element) => ::core::fmt::Display::fmt(element, f),
                Self::TruncatedEncoding(element) => ::core::fmt::Display::fmt(element, f),
                Self::UnexpectedInterpreterBytecodeHash(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::UnexpectedOpMetaHash(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::UnexpectedPointers(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::UnexpectedStoreBytecodeHash(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::WriteError(element) => ::core::fmt::Display::fmt(element, f),
                Self::ZeroInputs(element) => ::core::fmt::Display::fmt(element, f),
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<BadDynamicLength>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: BadDynamicLength) -> Self {
            Self::BadDynamicLength(value)
        }
    }
    impl ::core::convert::From<DoWhileMaxInputs>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: DoWhileMaxInputs) -> Self {
            Self::DoWhileMaxInputs(value)
        }
    }
    impl ::core::convert::From<InsufficientLoopOutputs>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: InsufficientLoopOutputs) -> Self {
            Self::InsufficientLoopOutputs(value)
        }
    }
    impl ::core::convert::From<MinFinalStack>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: MinFinalStack) -> Self {
            Self::MinFinalStack(value)
        }
    }
    impl ::core::convert::From<MinStackBottom>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: MinStackBottom) -> Self {
            Self::MinStackBottom(value)
        }
    }
    impl ::core::convert::From<MissingEntrypoint>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: MissingEntrypoint) -> Self {
            Self::MissingEntrypoint(value)
        }
    }
    impl ::core::convert::From<NotPosIntPrice>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: NotPosIntPrice) -> Self {
            Self::NotPosIntPrice(value)
        }
    }
    impl ::core::convert::From<OutOfBoundsConstantsRead>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: OutOfBoundsConstantsRead) -> Self {
            Self::OutOfBoundsConstantsRead(value)
        }
    }
    impl ::core::convert::From<OutOfBoundsStackRead>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: OutOfBoundsStackRead) -> Self {
            Self::OutOfBoundsStackRead(value)
        }
    }
    impl ::core::convert::From<PRBMath_MulDiv18_Overflow>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: PRBMath_MulDiv18_Overflow) -> Self {
            Self::PRBMath_MulDiv18_Overflow(value)
        }
    }
    impl ::core::convert::From<PRBMath_MulDiv_Overflow>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: PRBMath_MulDiv_Overflow) -> Self {
            Self::PRBMath_MulDiv_Overflow(value)
        }
    }
    impl ::core::convert::From<PRBMath_UD60x18_Ceil_Overflow>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: PRBMath_UD60x18_Ceil_Overflow) -> Self {
            Self::PRBMath_UD60x18_Ceil_Overflow(value)
        }
    }
    impl ::core::convert::From<PRBMath_UD60x18_Exp2_InputTooBig>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: PRBMath_UD60x18_Exp2_InputTooBig) -> Self {
            Self::PRBMath_UD60x18_Exp2_InputTooBig(value)
        }
    }
    impl ::core::convert::From<PRBMath_UD60x18_Exp_InputTooBig>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: PRBMath_UD60x18_Exp_InputTooBig) -> Self {
            Self::PRBMath_UD60x18_Exp_InputTooBig(value)
        }
    }
    impl ::core::convert::From<PRBMath_UD60x18_Gm_Overflow>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: PRBMath_UD60x18_Gm_Overflow) -> Self {
            Self::PRBMath_UD60x18_Gm_Overflow(value)
        }
    }
    impl ::core::convert::From<PRBMath_UD60x18_Log_InputTooSmall>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: PRBMath_UD60x18_Log_InputTooSmall) -> Self {
            Self::PRBMath_UD60x18_Log_InputTooSmall(value)
        }
    }
    impl ::core::convert::From<PRBMath_UD60x18_Sqrt_Overflow>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: PRBMath_UD60x18_Sqrt_Overflow) -> Self {
            Self::PRBMath_UD60x18_Sqrt_Overflow(value)
        }
    }
    impl ::core::convert::From<StackPopUnderflow>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: StackPopUnderflow) -> Self {
            Self::StackPopUnderflow(value)
        }
    }
    impl ::core::convert::From<StalePrice> for RainterpreterExpressionDeployerNPErrors {
        fn from(value: StalePrice) -> Self {
            Self::StalePrice(value)
        }
    }
    impl ::core::convert::From<TruncatedEncoding>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: TruncatedEncoding) -> Self {
            Self::TruncatedEncoding(value)
        }
    }
    impl ::core::convert::From<UnexpectedInterpreterBytecodeHash>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: UnexpectedInterpreterBytecodeHash) -> Self {
            Self::UnexpectedInterpreterBytecodeHash(value)
        }
    }
    impl ::core::convert::From<UnexpectedOpMetaHash>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: UnexpectedOpMetaHash) -> Self {
            Self::UnexpectedOpMetaHash(value)
        }
    }
    impl ::core::convert::From<UnexpectedPointers>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: UnexpectedPointers) -> Self {
            Self::UnexpectedPointers(value)
        }
    }
    impl ::core::convert::From<UnexpectedStoreBytecodeHash>
    for RainterpreterExpressionDeployerNPErrors {
        fn from(value: UnexpectedStoreBytecodeHash) -> Self {
            Self::UnexpectedStoreBytecodeHash(value)
        }
    }
    impl ::core::convert::From<WriteError> for RainterpreterExpressionDeployerNPErrors {
        fn from(value: WriteError) -> Self {
            Self::WriteError(value)
        }
    }
    impl ::core::convert::From<ZeroInputs> for RainterpreterExpressionDeployerNPErrors {
        fn from(value: ZeroInputs) -> Self {
            Self::ZeroInputs(value)
        }
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "DISpair", abi = "DISpair(address,address,address,address,bytes)")]
    pub struct DispairFilter {
        pub sender: ::ethers::core::types::Address,
        pub deployer: ::ethers::core::types::Address,
        pub interpreter: ::ethers::core::types::Address,
        pub store: ::ethers::core::types::Address,
        pub op_meta: ::ethers::core::types::Bytes,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "ExpressionAddress", abi = "ExpressionAddress(address,address)")]
    pub struct ExpressionAddressFilter {
        pub sender: ::ethers::core::types::Address,
        pub expression: ::ethers::core::types::Address,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "NewExpression",
        abi = "NewExpression(address,bytes[],uint256[],uint256[])"
    )]
    pub struct NewExpressionFilter {
        pub sender: ::ethers::core::types::Address,
        pub sources: ::std::vec::Vec<::ethers::core::types::Bytes>,
        pub constants: ::std::vec::Vec<::ethers::core::types::U256>,
        pub min_outputs: ::std::vec::Vec<::ethers::core::types::U256>,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum RainterpreterExpressionDeployerNPEvents {
        DispairFilter(DispairFilter),
        ExpressionAddressFilter(ExpressionAddressFilter),
        NewExpressionFilter(NewExpressionFilter),
    }
    impl ::ethers::contract::EthLogDecode for RainterpreterExpressionDeployerNPEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = DispairFilter::decode_log(log) {
                return Ok(
                    RainterpreterExpressionDeployerNPEvents::DispairFilter(decoded),
                );
            }
            if let Ok(decoded) = ExpressionAddressFilter::decode_log(log) {
                return Ok(
                    RainterpreterExpressionDeployerNPEvents::ExpressionAddressFilter(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded) = NewExpressionFilter::decode_log(log) {
                return Ok(
                    RainterpreterExpressionDeployerNPEvents::NewExpressionFilter(decoded),
                );
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for RainterpreterExpressionDeployerNPEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::DispairFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpressionAddressFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NewExpressionFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<DispairFilter>
    for RainterpreterExpressionDeployerNPEvents {
        fn from(value: DispairFilter) -> Self {
            Self::DispairFilter(value)
        }
    }
    impl ::core::convert::From<ExpressionAddressFilter>
    for RainterpreterExpressionDeployerNPEvents {
        fn from(value: ExpressionAddressFilter) -> Self {
            Self::ExpressionAddressFilter(value)
        }
    }
    impl ::core::convert::From<NewExpressionFilter>
    for RainterpreterExpressionDeployerNPEvents {
        fn from(value: NewExpressionFilter) -> Self {
            Self::NewExpressionFilter(value)
        }
    }
    ///Container type for all input parameters for the `deployExpression` function with signature `deployExpression(bytes[],uint256[],uint256[])` and selector `0x5511cb67`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "deployExpression",
        abi = "deployExpression(bytes[],uint256[],uint256[])"
    )]
    pub struct DeployExpressionCall {
        pub sources: ::std::vec::Vec<::ethers::core::types::Bytes>,
        pub constants: ::std::vec::Vec<::ethers::core::types::U256>,
        pub min_outputs: ::std::vec::Vec<::ethers::core::types::U256>,
    }
    ///Container type for all input parameters for the `interpreter` function with signature `interpreter()` and selector `0x3a35cf17`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "interpreter", abi = "interpreter()")]
    pub struct InterpreterCall;
    ///Container type for all input parameters for the `store` function with signature `store()` and selector `0x975057e7`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "store", abi = "store()")]
    pub struct StoreCall;
    ///Container type for all input parameters for the `supportsInterface` function with signature `supportsInterface(bytes4)` and selector `0x01ffc9a7`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "supportsInterface", abi = "supportsInterface(bytes4)")]
    pub struct SupportsInterfaceCall {
        pub interface_id: [u8; 4],
    }
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum RainterpreterExpressionDeployerNPCalls {
        DeployExpression(DeployExpressionCall),
        Interpreter(InterpreterCall),
        Store(StoreCall),
        SupportsInterface(SupportsInterfaceCall),
    }
    impl ::ethers::core::abi::AbiDecode for RainterpreterExpressionDeployerNPCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <DeployExpressionCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::DeployExpression(decoded));
            }
            if let Ok(decoded) = <InterpreterCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::Interpreter(decoded));
            }
            if let Ok(decoded) = <StoreCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::Store(decoded));
            }
            if let Ok(decoded) = <SupportsInterfaceCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SupportsInterface(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for RainterpreterExpressionDeployerNPCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::DeployExpression(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Interpreter(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Store(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::SupportsInterface(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for RainterpreterExpressionDeployerNPCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::DeployExpression(element) => ::core::fmt::Display::fmt(element, f),
                Self::Interpreter(element) => ::core::fmt::Display::fmt(element, f),
                Self::Store(element) => ::core::fmt::Display::fmt(element, f),
                Self::SupportsInterface(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<DeployExpressionCall>
    for RainterpreterExpressionDeployerNPCalls {
        fn from(value: DeployExpressionCall) -> Self {
            Self::DeployExpression(value)
        }
    }
    impl ::core::convert::From<InterpreterCall>
    for RainterpreterExpressionDeployerNPCalls {
        fn from(value: InterpreterCall) -> Self {
            Self::Interpreter(value)
        }
    }
    impl ::core::convert::From<StoreCall> for RainterpreterExpressionDeployerNPCalls {
        fn from(value: StoreCall) -> Self {
            Self::Store(value)
        }
    }
    impl ::core::convert::From<SupportsInterfaceCall>
    for RainterpreterExpressionDeployerNPCalls {
        fn from(value: SupportsInterfaceCall) -> Self {
            Self::SupportsInterface(value)
        }
    }
    ///Container type for all return fields from the `deployExpression` function with signature `deployExpression(bytes[],uint256[],uint256[])` and selector `0x5511cb67`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct DeployExpressionReturn(
        pub ::ethers::core::types::Address,
        pub ::ethers::core::types::Address,
        pub ::ethers::core::types::Address,
    );
    ///Container type for all return fields from the `interpreter` function with signature `interpreter()` and selector `0x3a35cf17`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct InterpreterReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `store` function with signature `store()` and selector `0x975057e7`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct StoreReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `supportsInterface` function with signature `supportsInterface(bytes4)` and selector `0x01ffc9a7`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct SupportsInterfaceReturn(pub bool);
}
