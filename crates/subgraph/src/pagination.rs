use crate::{cynic_client::CynicClientError, utils::slice_list};

/// Utiltity for client-side pagination of arbitrary queries
///
/// Takes a PageQueryClient and PageQueryVariables so that any
/// arbitrary query results can be specified outside
/// of this generalized pagination logic.
///
/// After fetching the required pages, it retuns the desired page based
/// the given 'skip' + 'first'.
pub trait PaginationClient {
    async fn query_paginated<T: Clone, V: PageQueryVariables + Clone, Q: PageQueryClient<T, V>>(
        &self,
        skip: Option<u32>,
        first: Option<u32>,
        page_query_client: Q,
        page_query_variables: V,
    ) -> Result<Vec<T>, CynicClientError> {
        let mut results = vec![];
        let mut more_pages_available = true;
        let mut page_skip = 0;
        let page_first = 200;

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
