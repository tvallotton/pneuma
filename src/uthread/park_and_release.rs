use std::io;
use std::mem::transmute;

pub fn park_and_release(release: impl FnOnce()) -> io::Result<()> {
    let mut impl_fn_mut = into_fn_mut(release);

    let dyn_fn_mut: *mut dyn FnMut() = &mut impl_fn_mut;
    // Safety:
    // It is fine to transmute the lifetime since the closure will be called
    //
    let dyn_fn_mut: *mut dyn FnMut() = unsafe { transmute(dyn_fn_mut) };

    unsafe {
        pneuma::uthread::current()
            .cx
            .set_release_closure(Some(dyn_fn_mut));
    }

    pneuma::uthread::park()
}

fn into_fn_mut(f: impl FnOnce()) -> impl FnMut() {
    let mut option = Some(f);

    move || {
        option.take().map(|f| f());
    }
}
