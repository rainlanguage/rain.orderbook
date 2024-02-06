use crate::{cynic_client::CynicClientError, utils::slice_list};
use std::num::TryFromIntError;
use thiserror::Error;

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
pub struct PaginationClient {
    page_size: u32,
}

impl PaginationClient {
    pub fn new(page_size: u32) -> Self {
        Self { page_size }
    }

    pub async fn query_paginated<
        T: Clone,
        V: PageQueryVariables + Clone,
        Q: PageQueryClient<T, V>,
    >(
        &self,
        skip: Option<u32>,
        first: Option<u32>,
        page_query_client: Q,
        page_query_variables: V,
    ) -> Result<Vec<T>, PaginationClientError> {
        let mut results = vec![];
        let mut more_pages_available = true;
        let mut page_skip = 0;
        let page_first = i32::try_from(self.page_size)?;

        let max_results_len = first.map(|f| skip.unwrap_or(0) + f);

        // Loop through fetching another page if there are more pages available AND we have not yet receive the max results length (as specified by our pagination skip & first)
        while more_pages_available && max_results_len.map_or(true, |l| results.len() < l as usize) {
            // Fetch a page
            let variables = page_query_variables.with_pagination(Some(page_skip), Some(page_first));
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
                        page_skip += page_first;
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

        Ok(slice_list(results, skip, first))
    }
}

/// Client that provides a fn to query a "page", and another to sort a list of results.
/// This allows the query to be used for client-side pagination.
/// The query_page function can potentially make multiple queries and merge the results into a single list.
pub trait PageQueryClient<T, V> {
    fn query_page(
        &self,
        variables: V,
    ) -> impl std::future::Future<Output = Result<Vec<T>, CynicClientError>> + Send;

    fn sort_results(results: Vec<T>) -> Vec<T>;
}

/// Builder fn to setup query variables with a provided skip & option
pub trait PageQueryVariables {
    fn with_pagination(&self, skip: Option<i32>, first: Option<i32>) -> Self;
}

#[cfg(test)]
mod test {
    use super::{PageQueryVariables, PaginationClient};
    use crate::{utils::slice_list, PageQueryClient};

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

    #[derive(Clone)]
    struct MockPageQueryClient {}
    impl PageQueryClient<u32, MockPageQueryVariables> for MockPageQueryClient {
        async fn query_page(
            &self,
            variables: MockPageQueryVariables,
        ) -> Result<Vec<u32>, crate::cynic_client::CynicClientError> {
            let all_vals: Vec<u32> = (0u32..1000u32).collect();

            let skip = variables.skip.map(|v| u32::try_from(v).unwrap());
            let first = variables.first.map(|v| u32::try_from(v).unwrap());
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
        let pagination_client = PaginationClient::new(50);

        let vals = pagination_client
            .query_paginated(None, None, page_query_client, page_query_variables)
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
        let pagination_client = PaginationClient::new(50);

        let vals = pagination_client
            .query_paginated(Some(100), None, page_query_client, page_query_variables)
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
        let pagination_client = PaginationClient::new(50);

        let vals = pagination_client
            .query_paginated(None, Some(500), page_query_client, page_query_variables)
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
        let pagination_client = PaginationClient::new(50);

        let vals = pagination_client
            .query_paginated(None, Some(500), page_query_client, page_query_variables)
            .await
            .unwrap();

        assert_eq!(vals, (0u32..500u32).collect::<Vec<u32>>());
    }

    #[tokio::test]
    async fn query_paginated_skip_overflow() {
        let page_query_client = MockPageQueryClient {};
        let page_query_variables = MockPageQueryVariables {
            skip: None,
            first: None,
        };
        let pagination_client = PaginationClient::new(50);

        let vals = pagination_client
            .query_paginated(Some(2000), None, page_query_client, page_query_variables)
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
        let pagination_client = PaginationClient::new(50);

        let vals = pagination_client
            .query_paginated(None, Some(2000), page_query_client, page_query_variables)
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
        let pagination_client = PaginationClient::new(50);

        let vals = pagination_client
            .query_paginated(
                Some(2000),
                Some(500),
                page_query_client,
                page_query_variables,
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
        let pagination_client = PaginationClient::new(5);
        let vals = pagination_client
            .query_paginated(
                Some(50),
                Some(500),
                page_query_client.clone(),
                page_query_variables.clone(),
            )
            .await
            .unwrap();
        assert_eq!(vals, (50u32..550u32).collect::<Vec<u32>>());

        let pagination_client = PaginationClient::new(30);
        let vals = pagination_client
            .query_paginated(
                Some(50),
                Some(500),
                page_query_client.clone(),
                page_query_variables.clone(),
            )
            .await
            .unwrap();
        assert_eq!(vals, (50u32..550u32).collect::<Vec<u32>>());

        let pagination_client = PaginationClient::new(99);
        let vals = pagination_client
            .query_paginated(
                Some(50),
                Some(500),
                page_query_client.clone(),
                page_query_variables.clone(),
            )
            .await
            .unwrap();
        assert_eq!(vals, (50u32..550u32).collect::<Vec<u32>>());

        let pagination_client = PaginationClient::new(123);
        let vals = pagination_client
            .query_paginated(
                Some(50),
                Some(500),
                page_query_client.clone(),
                page_query_variables.clone(),
            )
            .await
            .unwrap();
        assert_eq!(vals, (50u32..550u32).collect::<Vec<u32>>());

        let pagination_client = PaginationClient::new(2000);
        let vals = pagination_client
            .query_paginated(
                Some(50),
                Some(500),
                page_query_client.clone(),
                page_query_variables.clone(),
            )
            .await
            .unwrap();
        assert_eq!(vals, (50u32..550u32).collect::<Vec<u32>>());
    }
}
