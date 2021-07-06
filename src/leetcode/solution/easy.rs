mod n0461 {
    pub fn hamming_distance(x: usize, y: usize) -> usize {
        let s = format!("{:b}", x ^ y);
        s.matches("1").count()
    }
}
mod n0557 {

    pub fn reverse_w_ords_in_str(s: &str) -> String {
        s.split(" ")
            .into_iter()
            .map(|s| s.chars().rev().collect::<String>())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

mod n0905 {
    use std::collections::VecDeque;

    pub fn sort_array_by_parity(arr: Vec<usize>) -> Vec<usize> {
        let list: VecDeque<usize> = VecDeque::new();
        let it = arr.iter();
        let res = it.fold(list, |mut acc, n| {
            if n % 2 == 0 {
                acc.push_front(n.to_owned());
                acc
            } else {
                acc.push_back(n.to_owned());
                acc
            }
        });
        Vec::from(res)
    }
}

mod n0977 {
    pub fn squares_of_a_sorted_array(arr: Vec<i32>) -> Vec<i32> {
        let mut v: Vec<i32> = arr.iter().map(|i| i * i).collect();
        v.sort();
        v
    }
}
mod n1047 {
    pub fn remove_all_adjacent_duplicates(s: &str) -> String {
        let stack = Vec::<char>::new();
        s.chars()
            .fold(stack, |mut acc, c| {
                if acc.is_empty() {
                    acc.push(c);
                    acc
                } else {
                    if acc.last().unwrap().eq(&c) {
                        acc.pop();
                        acc
                    } else {
                        acc.push(c);
                        acc
                    }
                }
            })
            .into_iter()
            .collect::<String>()
    }
}

#[cfg(test)]
mod tests {
    use crate::leetcode::solution::easy::{
        n0905::sort_array_by_parity, n0977::squares_of_a_sorted_array,
        n1047::remove_all_adjacent_duplicates,
    };

    use super::{n0461::hamming_distance, n0557::reverse_w_ords_in_str};

    #[test]
    fn n0461() {
        assert_eq!(hamming_distance(1, 4), 2);
        assert_eq!(hamming_distance(3, 1), 1);
    }
    #[test]
    fn n0557() {
        assert_eq!(
            reverse_w_ords_in_str("Let's take LeetCode contest"),
            "s'teL ekat edoCteeL tsetnoc"
        );
    }
    #[test]
    fn n0905() {
        assert_eq!(sort_array_by_parity(vec![3, 1, 2, 4]), vec![4, 2, 3, 1]);
    }

    #[test]
    fn n0977() {
        assert_eq!(
            squares_of_a_sorted_array(vec![-4, -1, 0, 3, 10]),
            vec![0, 1, 9, 16, 100]
        );
    }
    #[test]
    fn n1047() {
        assert_eq!(remove_all_adjacent_duplicates("abbaca"), "ca")
    }
}
