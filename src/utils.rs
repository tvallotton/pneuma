/// calls borrow_mut on debug and unchecked on release
macro_rules! borrow_mut {
    ($expr: expr) => {{
        #[cfg(debug_assertions)]
        {
            &mut *($expr).borrow_mut()
        }
        #[cfg(not(debug_assertions))]
        {
            unsafe { ($expr).as_ptr().as_mut().unwrap_unchecked() }
        }
    }};
}
