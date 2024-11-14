use crate::{cynic_client::CynicClientError, utils::slice_list};
use serde::{Deserialize, Serialize};
use std::num::TryFromIntError;
use thiserror::Error;
#[cfg(target_family = "wasm")]
use tsify::Tsify;
use typeshare::typeshare;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct PaginationArgs {
    pub page: u16,
    pub page_size: u16,
}

#[cfg(target_family = "wasm")]
mod wasm_impls {
    use super::*;
    use rain_orderbook_bindings::impl_all_wasm_traits;

    impl_all_wasm_traits!(PaginationArgs);
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct QueryPaginationVariables {
    pub skip: Option<i32>,
    pub first: Option<i32>,
}

#[derive(Error, Debug)]
pub enum PaginationClientError {
    #[error(transparent)]
    TryFromIntError(#[from] TryFromIntError),
    #[error(transparent)]
    CynicClientError(#[from] CynicClientError),
}

/// Utiltity for client-side pagination of arbitrary queries
///
/// Takes a PageQueryClient and PageQueryVariables so that any
/// arbitrary query results can be specified outside
/// of this generalized pagination logic.
///
/// After fetching the required pages, it retuns the desired page based
/// the given 'skip' + 'first'.
pub trait PaginationClient {
    fn parse_pagination_args(pagination_args: PaginationArgs) -> QueryPaginationVariables {
        let first: i32 = pagination_args.page_size.into();
        let skip: i32 = ((pagination_args.page - 1) * pagination_args.page_size).into();

        QueryPaginationVariables {
            first: Some(first),
            skip: Some(skip),
        }
    }

    async fn query_paginated<T: Clone, V: PageQueryVariables + Clone, Q: PageQueryClient<T, V>>(
        &self,
        pagination_variables: QueryPaginationVariables,
        page_query_client: Q,
        page_query_variables: V,
        page_query_limit: i32,
    ) -> Result<Vec<T>, PaginationClientError> {
        let mut results = vec![];
        let mut more_pages_available = true;
        let mut page_skip = 0;
        let max_results_len = pagination_variables
            .first
            .map(|f| pagination_variables.skip.unwrap_or(0) + f);

        // Loop through fetching another page if there are more pages available AND we have not yet receive the max results length (as specified by our pagination skip & first)
        while more_pages_available && max_results_len.map_or(true, |l| results.len() < l as usize) {
            // Fetch a page
            let variables =
                page_query_variables.with_pagination(Some(page_skip), Some(page_query_limit));
            let res = page_query_client.query_page(variables.clone()).await;

            match res {
                Ok(page_results) => {
                    // No results
                    if page_results.is_empty() {
                        more_pages_available = false;
                        Ok(())
                    }
                    // Results received, append to merged vec and re-sort
                    else {
                        results.extend(page_results);
                        results = Q::sort_results(results);
                        page_skip += page_query_limit;
                        Ok(())
                    }
                }
                // No results
                Err(CynicClientError::Empty) => {
                    more_pages_available = false;
                    Ok(())
                }
                Err(e) => Err(e),
            }?;
        }

        let skip_u16 = pagination_variables.skip.map(u16::try_from).transpose()?;
        let first_u16 = pagination_variables.first.map(u16::try_from).transpose()?;

        Ok(slice_list(results, skip_u16, first_u16))
    }
}

/// Client that provides a fn to query a "page", and another to sort a list of results.
/// This allows the query to be used for client-side pagination.
/// The query_page function can potentially make multiple queries and merge the results into a single list.
#[cfg(not(target_family = "wasm"))]
pub trait PageQueryClient<T, V> {
    fn query_page(
        &self,
        variables: V,
    ) -> impl std::future::Future<Output = Result<Vec<T>, CynicClientError>> + Send;

    fn sort_results(results: Vec<T>) -> Vec<T>;
}

/// Client that provides a fn to query a "page", and another to sort a list of results.
/// This allows the query to be used for client-side pagination.
/// The query_page function can potentially make multiple queries and merge the results into a single list.
#[cfg(target_family = "wasm")]
pub trait PageQueryClient<T, V> {
    fn query_page(
        &self,
        variables: V,
    ) -> impl std::future::Future<Output = Result<Vec<T>, CynicClientError>>;

    fn sort_results(results: Vec<T>) -> Vec<T>;
}

/// Builder fn to setup query variables with a provided skip & option
pub trait PageQueryVariables {
    fn with_pagination(&self, skip: Option<i32>, first: Option<i32>) -> Self;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{pagination::QueryPaginationVariables, utils::slice_list, PageQueryClient};

    #[derive(Clone)]
    struct MockPageQueryVariables {
        skip: Option<i32>,
        first: Option<i32>,
    }
    impl PageQueryVariables for MockPageQueryVariables {
        fn with_pagination(&self, skip: Option<i32>, first: Option<i32>) -> Self {
            Self { skip, first }
        }
    }

    struct MockPaginationClient {}
    impl PaginationClient for MockPaginationClient {}

    #[derive(Clone)]
    struct MockPageQueryClient {}
    impl PageQueryClient<u32, MockPageQueryVariables> for MockPageQueryClient {
        async fn query_page(
            &self,
            variables: MockPageQueryVariables,
        ) -> Result<Vec<u32>, crate::cynic_client::CynicClientError> {
            let all_vals: Vec<u32> = (0..1000).collect();

            let skip = variables.skip.map(|v| u16::try_from(v).unwrap());
            let first = variables.first.map(|v| u16::try_from(v).unwrap());
            let page_vals = slice_list(all_vals, skip, first);

            Ok(page_vals)
        }
        fn sort_results(results: Vec<u32>) -> Vec<u32> {
            let mut new_results = results.clone();
            new_results.sort();
            new_results
        }
    }

    #[tokio::test]
    async fn query_paginated_default() {
        let page_query_client = MockPageQueryClient {};
        let page_query_variables = MockPageQueryVariables {
            skip: None,
            first: None,
        };
        let pagination_client = MockPaginationClient {};
        let vals = pagination_client
            .query_paginated(
                QueryPaginationVariables {
                    skip: None,
                    first: None,
                },
                page_query_client,
                page_query_variables,
                50,
            )
            .await
            .unwrap();

        assert_eq!(vals, (0u32..1000u32).collect::<Vec<u32>>());
    }

    #[tokio::test]
    async fn query_paginated_skip() {
        let page_query_client = MockPageQueryClient {};
        let page_query_variables = MockPageQueryVariables {
            skip: None,
            first: None,
        };
        let pagination_client = MockPaginationClient {};
        let vals = pagination_client
            .query_paginated(
                QueryPaginationVariables {
                    skip: Some(100),
                    first: None,
                },
                page_query_client,
                page_query_variables,
                50,
            )
            .await
            .unwrap();

        assert_eq!(vals, (100u32..1000u32).collect::<Vec<u32>>());
    }

    #[tokio::test]

    async fn query_paginated_first() {
        let page_query_client = MockPageQueryClient {};
        let page_query_variables = MockPageQueryVariables {
            skip: None,
            first: None,
        };
        let pagination_client = MockPaginationClient {};
        let vals = pagination_client
            .query_paginated(
                QueryPaginationVariables {
                    skip: None,
                    first: Some(500),
                },
                page_query_client,
                page_query_variables,
                50,
            )
            .await
            .unwrap();

        assert_eq!(vals, (0u32..500u32).collect::<Vec<u32>>());
    }

    #[tokio::test]
    async fn query_paginated_skip_and_first() {
        let page_query_client = MockPageQueryClient {};
        let page_query_variables = MockPageQueryVariables {
            skip: None,
            first: None,
        };
        let pagination_client = MockPaginationClient {};
        let vals: Vec<_> = pagination_client
            .query_paginated(
                QueryPaginationVariables {
                    skip: Some(50),
                    first: Some(500),
                },
                page_query_client,
                page_query_variables,
                50,
            )
            .await
            .unwrap();

        assert_eq!(vals, (50u32..550u32).collect::<Vec<u32>>());
    }

    #[tokio::test]
    async fn query_paginated_skip_overflow() {
        let page_query_client = MockPageQueryClient {};
        let page_query_variables = MockPageQueryVariables {
            skip: None,
            first: None,
        };
        let pagination_client = MockPaginationClient {};
        let vals = pagination_client
            .query_paginated(
                QueryPaginationVariables {
                    skip: Some(2000),
                    first: None,
                },
                page_query_client,
                page_query_variables,
                50,
            )
            .await
            .unwrap();

        assert_eq!(vals, Vec::<u32>::new());
    }

    #[tokio::test]
    async fn query_paginated_first_overflow() {
        let page_query_client = MockPageQueryClient {};
        let page_query_variables = MockPageQueryVariables {
            skip: None,
            first: None,
        };
        let pagination_client = MockPaginationClient {};
        let vals = pagination_client
            .query_paginated(
                QueryPaginationVariables {
                    skip: None,
                    first: Some(2000),
                },
                page_query_client,
                page_query_variables,
                200,
            )
            .await
            .unwrap();

        assert_eq!(vals, (0u32..1000u32).collect::<Vec<u32>>());
    }

    #[tokio::test]
    async fn query_paginated_skip_overflow_and_first_overflow() {
        let page_query_client = MockPageQueryClient {};
        let page_query_variables = MockPageQueryVariables {
            skip: None,
            first: None,
        };
        let pagination_client = MockPaginationClient {};
        let vals = pagination_client
            .query_paginated(
                QueryPaginationVariables {
                    skip: Some(2000),
                    first: Some(500),
                },
                page_query_client,
                page_query_variables,
                50,
            )
            .await
            .unwrap();

        assert_eq!(vals, Vec::<u32>::new());
    }

    #[tokio::test]
    async fn query_paginated_supports_arbitrary_page_size() {
        let page_query_client = MockPageQueryClient {};
        let page_query_variables = MockPageQueryVariables {
            skip: None,
            first: None,
        };
        let pagination_client = MockPaginationClient {};
        let vals = pagination_client
            .query_paginated(
                QueryPaginationVariables {
                    skip: Some(50),
                    first: Some(500),
                },
                page_query_client.clone(),
                page_query_variables.clone(),
                5,
            )
            .await
            .unwrap();
        assert_eq!(vals, (50u32..550u32).collect::<Vec<u32>>());

        let pagination_client = MockPaginationClient {};
        let vals = pagination_client
            .query_paginated(
                QueryPaginationVariables {
                    skip: Some(50),
                    first: Some(500),
                },
                page_query_client.clone(),
                page_query_variables.clone(),
                30,
            )
            .await
            .unwrap();
        assert_eq!(vals, (50u32..550u32).collect::<Vec<u32>>());

        let pagination_client = MockPaginationClient {};
        let vals = pagination_client
            .query_paginated(
                QueryPaginationVariables {
                    skip: Some(50),
                    first: Some(500),
                },
                page_query_client.clone(),
                page_query_variables.clone(),
                99,
            )
            .await
            .unwrap();
        assert_eq!(vals, (50u32..550u32).collect::<Vec<u32>>());

        let pagination_client = MockPaginationClient {};
        let vals = pagination_client
            .query_paginated(
                QueryPaginationVariables {
                    skip: Some(50),
                    first: Some(500),
                },
                page_query_client.clone(),
                page_query_variables.clone(),
                123,
            )
            .await
            .unwrap();
        assert_eq!(vals, (50u32..550u32).collect::<Vec<u32>>());

        let pagination_client = MockPaginationClient {};
        let vals = pagination_client
            .query_paginated(
                QueryPaginationVariables {
                    skip: Some(50),
                    first: Some(500),
                },
                page_query_client.clone(),
                page_query_variables.clone(),
                2000,
            )
            .await
            .unwrap();
        assert_eq!(vals, (50u32..550u32).collect::<Vec<u32>>());
    }

    #[test]
    fn parse_pagination_args() {
        let query_pagination_vars = MockPaginationClient::parse_pagination_args(PaginationArgs {
            page: 1,
            page_size: 25,
        });
        assert_eq!(query_pagination_vars.skip, Some(0));
        assert_eq!(query_pagination_vars.first, Some(25));

        let query_pagination_vars = MockPaginationClient::parse_pagination_args(PaginationArgs {
            page: 2,
            page_size: 25,
        });
        assert_eq!(query_pagination_vars.skip, Some(25));
        assert_eq!(query_pagination_vars.first, Some(25));

        let query_pagination_vars = MockPaginationClient::parse_pagination_args(PaginationArgs {
            page: 3,
            page_size: 25,
        });
        assert_eq!(query_pagination_vars.skip, Some(50));
        assert_eq!(query_pagination_vars.first, Some(25));

        let query_pagination_vars = MockPaginationClient::parse_pagination_args(PaginationArgs {
            page: 1,
            page_size: 5,
        });
        assert_eq!(query_pagination_vars.skip, Some(0));
        assert_eq!(query_pagination_vars.first, Some(5));

        let query_pagination_vars = MockPaginationClient::parse_pagination_args(PaginationArgs {
            page: 1,
            page_size: 10,
        });
        assert_eq!(query_pagination_vars.skip, Some(0));
        assert_eq!(query_pagination_vars.first, Some(10));
    }
}
