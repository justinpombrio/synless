/// Divvy `cookies` up among children as fairly as possible, where the `i`th
/// child has `child_hungers[i]` hunger. Children should receive cookies in proportion
/// to their hunger, with the difficulty that cookies cannot be split into
/// pieces. Exact ties go to the leftmost tied child.
pub fn proportionally_divide(cookies: usize, child_hungers: &[usize]) -> Vec<usize> {
    let total_hunger: usize = child_hungers.iter().sum();
    // Start by allocating each child a guaranteed minimum number of cookies,
    // found as the floor of the real number of cookies they deserve.
    let mut cookie_allocation: Vec<usize> = child_hungers
        .iter()
        .map(|hunger| cookies * hunger / total_hunger)
        .collect();
    // Compute the number of cookies still remaining.
    let allocated_cookies: usize = cookie_allocation.iter().sum();
    let leftover: usize = cookies - allocated_cookies;
    // Determine what fraction of a cookie each child still deserves, found as
    // the remainder of the above division. Then hand out the remaining cookies
    // to the children with the largest remainders.
    let mut remainders: Vec<(usize, usize)> = child_hungers
        .iter()
        .map(|hunger| cookies * hunger % total_hunger)
        .enumerate()
        .collect();
    remainders.sort_by(|(_, r1), (_, r2)| r2.cmp(r1));
    remainders
        .into_iter()
        .take(leftover)
        .for_each(|(i, _)| cookie_allocation[i] += 1);
    // Return the maximally-fair cookie allocation.
    cookie_allocation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proportional_division() {
        assert_eq!(proportionally_divide(0, &vec!(1, 1)), vec!(0, 0));
        assert_eq!(proportionally_divide(1, &vec!(1, 1)), vec!(1, 0));
        assert_eq!(proportionally_divide(2, &vec!(1, 1)), vec!(1, 1));
        assert_eq!(proportionally_divide(3, &vec!(1, 1)), vec!(2, 1));
        assert_eq!(proportionally_divide(4, &vec!(10, 11, 12)), vec!(1, 1, 2));
        assert_eq!(proportionally_divide(5, &vec!(17)), vec!(5));
        assert_eq!(proportionally_divide(5, &vec!(12, 10, 11)), vec!(2, 1, 2));
        assert_eq!(proportionally_divide(5, &vec!(10, 10, 11)), vec!(2, 1, 2));
        assert_eq!(proportionally_divide(5, &vec!(2, 0, 1)), vec!(3, 0, 2));
        assert_eq!(proportionally_divide(61, &vec!(1, 2, 3)), vec!(10, 20, 31));
        assert_eq!(
            proportionally_divide(34583, &vec!(55, 98, 55, 7, 12, 200)),
            vec!(4455, 7937, 4454, 567, 972, 16198)
        );
    }
}
