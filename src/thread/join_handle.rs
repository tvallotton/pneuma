use std::marker::PhantomData;

pub struct JoinHandle<T> {
    _marker: PhantomData<T>,
}
