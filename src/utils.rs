// Approximate sum with optimized algorithm
pub fn approximate_sum_optimized(a: i32, b: i32, c: i32) -> (i32, i32, i32) {
    if a == 0 && b == 0 {
        return (0, 0, c.abs());
    }

    let mut best_x = 0;
    let mut best_y = 0;
    let mut best_diff = c.abs();

    if a == 0 {
        if b != 0 {
            let y = (c as f64 / b as f64).round() as i32;
            return (0, y, (y * b - c).abs());
        }
    } else if b == 0 {
        let x = (c as f64 / a as f64).round() as i32;
        return (x, 0, (x * a - c).abs());
    }

    let max_range = std::cmp::max(
        if a != 0 { (c / a).abs() + 1 } else { 0 },
        if b != 0 { (c / b).abs() + 1 } else { 0 },
    ).max(100);

    for x in -max_range..=max_range {
        if b != 0 {
            let ideal_y = (c - x * a) as f64 / b as f64;
            for y in [ideal_y.floor() as i32, ideal_y.ceil() as i32, ideal_y.round() as i32] {
                let sum_val = x * a + y * b;
                let diff = (sum_val - c).abs();

                if diff < best_diff {
                    best_diff = diff;
                    best_x = x;
                    best_y = y;
                }

                if diff == 0 {
                    return (best_x, best_y, best_diff);
                }
            }
        } else {
            let sum_val = x * a;
            let diff = (sum_val - c).abs();

            if diff < best_diff {
                best_diff = diff;
                best_x = x;
                best_y = 0;
            }

            if diff == 0 {
                return (best_x, best_y, best_diff);
            }
        }
    }

    (best_x, best_y, best_diff)
}

// Find closest solution to ax + by = c
pub fn find_closest(a: i32, b: i32, c: i32) -> (Vec<(i32, i32, i32)>, f64) {
    if a <= 0 || b <= 0 {
        panic!("a and b must be greater than 0");
    }
    
    let mut min_diff = f64::INFINITY;
    let mut final_diff = min_diff;
    let mut solutions = Vec::new();

    let max_ab = std::cmp::max(a, b);
    let x_max = ((c + max_ab) / a) + 2;

    let mut x = 1;
    while x <= x_max {
        let x_a = x * a;
        let y_ideal = (c - x_a) as f64 / b as f64;

        let y_floor = (y_ideal.floor() as i32).max(0);
        let y_ceil = (y_ideal.ceil() as i32).max(0);
        let y_round = (y_ideal.round() as i32).max(0);
        let y_candidates = vec![0, y_floor, y_ceil, y_round];

        for y in y_candidates {
            let total = x_a + y * b;
            let current_diff = (total - c).abs() as f64;

            if current_diff < min_diff {
                min_diff = current_diff;
                final_diff = total as f64 - c as f64;
                solutions = vec![(x, y, total)];
            } else if (current_diff - min_diff).abs() < f64::EPSILON {
                solutions.push((x, y, total));
            }
        }

        x += 1;
    }

    (solutions, final_diff)
}

// Solve for x,y in ax + by = c where a=s/2, b=b/2
pub fn solve_xy(s: i32, b: i32, c: i32) -> Option<(i32, i32)> {
    if c % 2 != 0 {
        return None;
    }

    let m = c / 2;
    let a = s / 2;
    let b_half = b / 2;
    let t_min = (m + a) / b_half;
    let t_max = m / a;
    
    if t_min <= t_max {
        let t = t_min;
        let x = -m + b_half * t;
        let y = m - a * t;
        Some((x, y))
    } else {
        None
    }
}