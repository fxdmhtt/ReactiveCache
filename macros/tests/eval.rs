use macros::evaluate;

static mut PRINT_INVOKED: i32 = 0;

fn print(content: String) {
    unsafe { PRINT_INVOKED += 1 };

    eprint!("{content}");
}

#[evaluate(print)]
pub fn get_number() -> i32 {
    42
}

#[test]
fn evaluate_test() {
    let v1 = get_number();
    let v2 = get_number();
    assert_eq!(v1, v2);

    let _ = get_number();
    assert_eq!(unsafe { PRINT_INVOKED }, 2);
}
