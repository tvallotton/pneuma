use super::stack::Stack;
use crate::sys::switch_context;
use std::{
    cell::{Cell, UnsafeCell},
    rc::Rc,
};

// 0: stack pointer       (sp: stack to jump to)
// 1: label to branch to  (function to call)
// 2: Arg pointer         (x0: the pointer passed to the coroutine)
// 3: frame pointer       (x29: frame to return to)
// 4: Link pointer        (x30: function to return to)

struct Context {
    registeres: [u64; 64],
    link: *const Context,
    ref_count: u64,
}

impl Context {
    pub unsafe fn this() -> Context {
        todo!()
    }

    pub(crate) fn zeroed() -> Self {
        let registers = UnsafeCell::new([0u64; 64]);

        Context {
            registers,
            ref_count: 1,
        }
    }

    pub fn new(link: Rc<Context>, size: usize, f: extern "C" fn(arg: u64), arg: u64) -> Rc<Self> {
        let stack = Stack::new(0);
        let mut registers = [0; 64];

        registers[0] = stack.bottom() as u64;
        registers[1] = f as usize as u64;
        registers[2] = arg;
        let registers = UnsafeCell::new(registers);

        Rc::new(Context {
            registers,
            stack,
            link,
        })
    }

    pub fn switch(&self, from: &Context) {
        let from = from.registers.get().cast();
        let to = self.registers.get().cast();
        unsafe { switch_context(from, to) }
    }
}
