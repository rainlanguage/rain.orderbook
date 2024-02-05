/// Slice a subset of a list, by optional skip (i.e. slice start) and first (i.e. slice length)
pub fn slice_list<T: Clone>(list: Vec<T>, skip: Option<u16>, first: Option<u16>) -> Vec<T> {
    let mut list_sliced = list.clone();
    if let Some(s) = skip {
        if list_sliced.len() > s as usize {
            list_sliced = list_sliced[s as usize..].to_vec();
        } else {
            list_sliced = vec![];
        }
    }
    if let Some(f) = first {
        if list_sliced.len() > f as usize {
            list_sliced = list_sliced[..f as usize].to_vec();
        }
    }

    list_sliced
}

#[cfg(test)]
mod test {
    use super::slice_list;

    #[test]
    fn slice_list_default() {
        let list = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let slice = slice_list(list, None, None);

        assert_eq!(slice, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn slice_list_skip() {
        let list = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let slice = slice_list(list, Some(3), None);

        assert_eq!(slice, vec![4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn slice_list_first() {
        let list = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let slice = slice_list(list, None, Some(3));

        assert_eq!(slice, vec![1, 2, 3]);
    }

    #[test]
    fn slice_list_skip_and_first() {
        let list = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let slice = slice_list(list, Some(3), Some(3));

        assert_eq!(slice, vec![4, 5, 6]);
    }

    #[test]
    fn slice_list_skip_overflow() {
        let list = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let slice = slice_list(list, Some(12), None);

        assert_eq!(slice, Vec::<u32>::new());
    }

    #[test]
    fn slice_list_limit_overflow() {
        let list = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let slice = slice_list(list, None, Some(12));

        assert_eq!(slice, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn slice_list_skip_overflow_and_limit_overflow() {
        let list = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let slice = slice_list(list, Some(12), Some(10));

        assert_eq!(slice, Vec::<u32>::new());
    }
}
