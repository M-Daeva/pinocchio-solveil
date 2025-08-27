use {
    crate::helpers::{
        extensions::counter::CounterExtension,
        suite::{
            core::App,
            types::{AppUser, TestResult},
        },
    },
    pretty_assertions::assert_eq,
};

#[test]
fn init_default() -> TestResult<()> {
    let mut app = App::new();

    app.counter_try_init(AppUser::Alice, Some(5))?;
    assert_eq!(app.counter_query_counter()?.value, 5);

    app.counter_try_set(AppUser::Alice, 7)?;
    assert_eq!(app.counter_query_counter()?.value, 7);

    Ok(())
}
