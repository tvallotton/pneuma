// # Calling convention
// the following pointers must be preserved: 
// * x19-x29
// * z8-z23 ?
// * v8-v15 ?
.global    __on_coroutine_exit
.text
__on_coroutine_exit: 
    ret
    
// #[repr(C)]
// pub struct Registers 
//     pub sp: u64,
//     pub fun: u64,
//     pub arg: u64,
//     pub frame: u64,
//     pub link: u64,
//     pub general: [u64; 59],
// 

.global    _switch_context
.p2align   4
_switch_context:
    
    // # Store context
    // store sp and function
    mov x2, sp
    
    adrp x3, __on_coroutine_exit@PAGE
    add x3, x3, __on_coroutine_exit@PAGEOFF
    
    stp x2, x3, [x0, #0]

    // We skip the argument pointer since 
    // the coroutine alreadys started
    // str x

    // store frame pointer and link
    // General purpose registers
    stp x29, x30, [x0, #24]
    stp x27, x28, [x0, #40]
    stp x25, x26, [x0, #56]
    stp x23, x24, [x0, #72]
    stp x21, x22, [x0, #88]
    stp x19, x20, [x0, #104]
    

    // store d registers
    stp d8,  d9,  [x0, #120]
    stp d10, d11, [x0, #136]
    stp d12, d13, [x0, #152]
    stp d14, d15, [x0, #168]

    // # Load context
    // General purpose registers
    ldp x29, x30, [x1, #24]
    ldp x27, x28, [x1, #40]
    ldp x25, x26, [x1, #56]
    ldp x23, x24, [x1, #72]
    ldp x21, x22, [x1, #88]
    ldp x19, x20, [x1, #104]

    // load d registers
    ldp d8,  d9,  [x1, #120]
    ldp d10, d11, [x1, #136]
    ldp d12, d13, [x1, #152]
    ldp d14, d15, [x1, #168]
    
    // load sp and function
    ldp x2, x3, [x1, #0]
    mov sp, x2

    // check if x29 has been initialized
    
    cbz x29, 1f 
    
    // initialize LN and FP to return to the parent original coroutine.
    ldp x29, x30, [x0, #24]

1:
    br x3
    ret

.global    _switch_no_save
.p2align   4
_switch_no_save: 
    
    // # Load context
    // General purpose registers
    ldp x29, x30, [x0, #24]
    ldp x27, x28, [x0, #40]
    ldp x25, x26, [x0, #56]
    ldp x23, x24, [x0, #72]
    ldp x21, x22, [x0, #88]
    ldp x19, x20, [x0, #104]

    // load d registers
    ldp d8,  d9,  [x0, #120]
    ldp d10, d11, [x0, #136]
    ldp d12, d13, [x0, #152]
    ldp d14, d15, [x0, #168]
    
    // load sp and function
    ldp x2, x3, [x0, #0]
    mov sp, x2
 
    // initialize LN and FP to return to the parent original coroutine.
    ldp x29, x30, [x0, #24]

    br x3
