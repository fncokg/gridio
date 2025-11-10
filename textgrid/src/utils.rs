use num_cpus;
use rayon::prelude::*;

pub fn fast_map<T, F, R>(items: &Vec<T>, func: F, min_len: usize) -> Vec<R>
where
    F: Fn(&T) -> R + Sync + Send,
    R: Send,
    T: Sync,
{
    match items.len() >= (num_cpus::get() * min_len / 2) {
        false => items.iter().map(func).collect::<Vec<R>>(),
        true => items
            .par_iter()
            .with_min_len(min_len)
            .map(func)
            .collect::<Vec<R>>(),
    }
}

pub fn fast_enumerate_map<T, F, R>(items: &Vec<T>, func: F, min_len: usize) -> Vec<R>
where
    F: Fn((usize, &T)) -> R + Sync + Send,
    R: Send,
    T: Sync,
{
    match items.len() >= (num_cpus::get() * min_len / 2) {
        false => items.iter().enumerate().map(func).collect::<Vec<R>>(),
        true => items
            .par_iter()
            .enumerate()
            .with_min_len(min_len)
            .map(func)
            .collect::<Vec<R>>(),
    }
}

pub fn fast_move_map<T, F, R>(items: Vec<T>, func: F, min_len: usize) -> Vec<R>
where
    F: Fn(T) -> R + Sync + Send,
    T: Send,
    R: Send,
{
    match items.len() >= (num_cpus::get() * min_len / 2) {
        false => items.into_iter().map(func).collect::<Vec<R>>(),
        true => items
            .into_par_iter()
            .with_min_len(min_len)
            .map(func)
            .collect::<Vec<R>>(),
    }
}
