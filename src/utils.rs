// pub fn sys_now() -> i64 {
//     std::time::SystemTime::now()
//         .duration_since(std::time::UNIX_EPOCH)
//         .unwrap_or_default()
//         .as_secs() as i64
// }

// pub fn rand_str(charset: &[u8], len: usize) -> String {
//     use rand::Rng;
//     let mut rng = rand::rng();
//     (0..len)
//         .map(|_| charset[rng.random_range(0..charset.len())] as char)
//         .collect()
// }
