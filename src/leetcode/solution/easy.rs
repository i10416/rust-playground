mod n0461 {
    pub fn hamming_distance(x: usize, y: usize) -> usize {
        let s = format!("{:b}", x ^ y);
        s.matches("1").count()
    }
}
mod n0557 {

    pub fn reverseWOrdsInStr(s: &str) -> String {
        s.split(" ")
            .into_iter()
            .map(|s| s.chars().rev().collect::<String>())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

mod n0905 {
    use std::collections::VecDeque;

    pub fn sortArrayByParity(arr: Vec<usize>) -> Vec<usize> {
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

#[cfg(test)]
mod tests {
    use crate::leetcode::solution::easy::n0905::sortArrayByParity;

    use super::{n0461::hamming_distance, n0557::reverseWOrdsInStr};

    #[test]
    fn n0461() {
        assert_eq!(hamming_distance(1, 4), 2);
        assert_eq!(hamming_distance(3, 1), 1);
    }
    #[test]
    fn n0557() {
        assert_eq!(
            reverseWOrdsInStr("Let's take LeetCode contest"),
            "s'teL ekat edoCteeL tsetnoc"
        );
    }
    #[test]
    fn n0905() {
        assert_eq!(sortArrayByParity(vec![3, 1, 2, 4]), vec![4, 2, 3, 1])
    }
}
