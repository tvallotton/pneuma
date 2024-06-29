use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        io_uring: { feature = "io-uring" },
    }
}
