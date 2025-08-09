use macros::memo;

#[memo]
pub fn get_number() -> i32 {
    println!("Calculate i32");
    42
}

#[memo]
pub fn get_text() -> String {
    println!("Calculate String");
    "hello".to_string()
}

#[test]
fn basic_memoization_works() {
    let v1 = get_number();
    let v2 = get_number();
    assert_eq!(v1, v2);

    let s1 = get_text();
    let s2 = get_text();
    assert_eq!(s1, s2);
}
