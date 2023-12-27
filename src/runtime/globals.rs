use super::Runtime;
use std::{cell::Cell, ptr::null};

thread_local! {
    static LINK: Cell<*const Runtime> = const { Cell::new(null()) };
}
