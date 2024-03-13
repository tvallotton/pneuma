use std::ptr::null;

static KEY: libc::pthread_key_t = 0;
static ONCE: libc::pthread_once_t = libc::PTHREAD_ONCE_INIT;

fn make_key() {}

fn get() {}
