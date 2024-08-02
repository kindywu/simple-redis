use std::hash::{Hash, Hasher};

#[derive(Debug, PartialEq)]
struct Double(f64);

impl Eq for Double {}

impl Hash for Double {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

fn main() {
    let a = Double(1.0);
    let b = Double(1.0);
    let c = Double(2.0);

    assert!(a == b); // 使用Eq特性
    assert!(a != c);

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    a.hash(&mut hasher);
    let hash_a = hasher.finish();

    hasher = std::collections::hash_map::DefaultHasher::new();
    b.hash(&mut hasher);
    let hash_b = hasher.finish();

    println!("Hash of a: {}", hash_a);
    println!("Hash of b: {}", hash_b); // 应该相同，因为a和b是相等的
}
