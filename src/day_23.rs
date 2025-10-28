use std::collections::VecDeque;

use index_list::{IndexList, ListIndex};

#[aoc(day23, part1)]
fn part_1(input: &[u8]) -> String {
    // VecDeque is faster for part 1
    let mut cups: VecDeque<_> = input.to_vec().into();
    for _ in 0..100 {
        let front = cups.front().copied().unwrap();
        cups.rotate_left(1);
        let a = cups.pop_front().unwrap();
        let b = cups.pop_front().unwrap();
        let c = cups.pop_front().unwrap();
        let mut before = if front == b'1' { b'9' } else { front - 1 };
        while before == a || before == b || before == c {
            before = if before == b'1' { b'9' } else { before - 1 };
        }
        let index = cups.iter().position(|&x| x == before).unwrap();
        cups.rotate_left(index + 1);
        cups.push_front(c);
        cups.push_front(b);
        cups.push_front(a);
        cups.rotate_right(index + 1);
    }
    let index = cups.iter().position(|&x| x == b'1').unwrap();
    cups.rotate_left(index);
    cups.pop_front();
    String::from_utf8(cups.into()).unwrap()
}

#[aoc(day23, part2)]
fn part_2(input: &[u8]) -> u64 {
    let list = crab_cups(input, 1_000_000, 10_000_000);
    let one = ListIndex::from(1_usize);
    let first = {
        let x = list.next_index(one);
        if x.is_none() { list.first_index() } else { x }
    };
    let second = {
        let x = list.next_index(first);
        if x.is_none() { list.first_index() } else { x }
    };
    let first_value = list.get(first).copied().unwrap();
    let second_value = list.get(second).copied().unwrap();
    u64::from(first_value) * u64::from(second_value)
}

fn crab_cups(input: &[u8], total_cups: u32, turns: usize) -> IndexList<u32> {
    // IndexList stores each value at an index, with links based on those indexes.
    // This allows a different iteration order from it's memory order, and still be
    // addressible by index. Storing each value at the index equal to the value
    // allows querying the next and previous item from it's value.

    // Each value is placed at index equal to the value
    let mut cups = IndexList::with_capacity(usize::try_from(total_cups + 1).unwrap());
    cups.extend(0_u32..=total_cups);
    // 0 is removed, leaving index 0 as `None`
    cups.remove_first();
    // Change iteration order to start with values from input. This does not
    // move the index of the values, only how they are linked.
    for &ch in input.iter().rev() {
        cups.shift_index_to_front(ListIndex::from(usize::from(ch - b'0')));
    }
    macro_rules! next {
        ($ix:expr) => {{
            let x = cups.next_index($ix);
            if x.is_none() { cups.first_index() } else { x }
        }};
    }
    let mut current = ListIndex::from(usize::from(input[0] - b'0'));
    for _ in 0..turns {
        let a = next!(current);
        let b = next!(a);
        let c = next!(b);
        let a_value = cups.get(a).copied().unwrap();
        let b_value = cups.get(b).copied().unwrap();
        let c_value = cups.get(c).copied().unwrap();

        // Find the value 1 smaller than current, skipping a, b and c, and possible wrapping around
        let mut t_value = cups.get(current).copied().unwrap();
        t_value = if t_value == 1 {
            total_cups
        } else {
            t_value - 1
        };
        while t_value == a_value || t_value == b_value || t_value == c_value {
            t_value = if t_value == 1 {
                total_cups
            } else {
                t_value - 1
            };
        }
        let t = ListIndex::from(t_value);
        cups.shift_index_after(a, t);
        cups.shift_index_after(b, a);
        cups.shift_index_after(c, b);
        current = next!(current);
    }
    cups
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(b"389125467", 9, 10 => "92658374")]
    #[test_case(b"389125467", 9, 100 => "67384529")]
    fn test_crab(input: &[u8], total_cups: u32, turns: usize) -> String {
        let list = crab_cups(input, total_cups, turns);
        let mut res = Vec::new();
        for &x in list
            .iter()
            .chain(&list)
            .skip_while(|&&x| x != 1)
            .skip(1)
            .take_while(|&&x| x != 1)
        {
            res.push(u8::try_from(x).unwrap() + b'0');
        }
        String::from_utf8(res).unwrap()
    }

    #[test]
    fn test_part_1() {
        let result = part_1(b"389125467");
        assert_eq!(result, "67384529");
    }

    #[test]
    fn test_part_2() {
        let result = part_2(b"389125467");
        assert_eq!(result, 149_245_887_792);
    }
}
