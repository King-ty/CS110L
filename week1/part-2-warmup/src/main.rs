/* The following exercises were borrowed from Will Crichton's CS 242 Rust lab. */

use std::collections::HashSet;

fn main() {
    println!("Hi! Try running \"cargo test\" to run tests.");
}

fn add_n(v: Vec<i32>, n: i32) -> Vec<i32> {
    // unimplemented!()
    let mut v = v;
    for i in &mut v {
        *i += n;
    }
    v
}

fn add_n_inplace(v: &mut Vec<i32>, n: i32) {
    for i in v {
        *i += n;
    }
}

fn dedup(v: &mut Vec<i32>) {
    // unimplemented!()
    let mut hset = HashSet::new();
    let mut retain_vec = Vec::new();
    for n in v.iter() {
        if hset.insert(*n) {
            retain_vec.push(*n);
        }
    }
    v.clear();
    for i in retain_vec {
        v.push(i);
    }

    // 失败尝试3
    // let mut retain_vec = Vec::new();
    // for n in v {
    //     if hset.insert(n) {
    //         retain_vec.push(n);
    //     }
    // }
    // v.retain(|x| retain_vec.contains(x));

    // 失败尝试2
    // v.retain(|x| hset.insert(x));

    // 失败尝试1
    // for (i, n) in v.iter().enumerate() {
    //     if !hset.insert(n) {
    //         v.remove(i);
    //     }
    // }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_n() {
        assert_eq!(add_n(vec![1, 100, 5], 2), vec![3, 102, 7]);
    }

    #[test]
    fn test_add_n_inplace() {
        let mut v = vec![1, 100, 5];
        add_n_inplace(&mut v, 2);
        assert_eq!(v, vec![3, 102, 7]);
    }

    #[test]
    fn test_dedup() {
        let mut v = vec![3, 1, 0, 1, 4, 4];
        dedup(&mut v);
        assert_eq!(v, vec![3, 1, 0, 4]);
    }
}
