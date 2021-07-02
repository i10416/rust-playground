mod stable {}

mod unstable {

    pub fn bubble_sort<T: Ord + Clone>(vec: Vec<T>) -> Vec<T> {
        let vec = swap(vec, 0);
        let res: Vec<T> = if let Some((largest, remains)) = vec[..].split_last() {
            let mut remains = bubble_sort(remains.to_vec());
            remains.push(largest.to_owned());
            remains
        } else {
            vec
        };
        res
    }
    fn swap<T: Ord + Clone>(mut vec: Vec<T>, focus: usize) -> Vec<T> {
        if focus + 1 >= vec.len() {
            vec
        } else {
            if vec[focus] > vec[focus + 1] {
                let tmp = vec[focus + 1].clone();
                vec[focus + 1] = vec[focus].clone();
                vec[focus] = tmp;
                swap(vec, focus + 1)
            } else {
                swap(vec, focus + 1)
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::algo::sorting::unstable::bubble_sort;

    #[test]
    fn test() {
        let v: Vec<i8> = vec![9, 5, 2, 1];
        assert_eq!(bubble_sort::<i8>(vec![]),vec![]);
        assert_eq!(bubble_sort(vec![2,1]),vec![1,2]);
        assert_eq!(bubble_sort(v), vec![1, 2, 5, 9]);
    }
}
