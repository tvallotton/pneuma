//       .global    my_function
//       .type      my_function, "function"
//       .p2align   4//
// my_function:
//       add        x0, x0, x1
//       ret//
//     


// # Calling convention
// the following pointers must be preserved: 
// * x19-x29
// * z8-z23 ?
// * v8-v15 ?

.global    _makecontext
.type      _makecontext, "function"
.p2align   4

//   pub fn _makecontext(
//       x0  ucp: *mut mcontext_t,  
//       x1  fun: extern "C" fn(),
//       x2  arg: *mut u8,
//       x3  stack: *mut u8,
//       x4  link: &mut mcontext_t,
//     );
_make_context:
    // store sp and function
    stp x3, x1, [x0, #160]
    // store FP and LN
    stp x3, x5, [x0, #80]

    // store argument
    str x2, [x0, #176]

    // get function ptr from link 
    ldr x4, [x4, #168]

    // store FP and LN
    // note here we set FP to null as it needs to be initialized by _setcontext
    stp xzr, x4, [x0, #80]
    
    ret



start: 


.global    get_context
.type      get_context, "function"
.p2align   4
get_context: 
    mov x2, sp
    ldr x3, __on_thread_exit
    stp x2, x3, [x0, #0]

    // We skip the argument pointer since 
    // the coroutine alreadys started

    // store frame pointer and link

    // General purpose registers
    stp x29, x30, [x0, #32]
    stp x27, x28, [x0, #48]
    stp x25, x26, [x0, #64]
    stp x23, x24, [x0, #80]
    stp x21, x22, [x0, #96]
    stp x19, x20, [x0, #112]
    

    // store d registers
    stp d8,  d9,  [x0, #128]
    stp d10, d11, [x0, #144]
    stp d12, d13, [x0, #160]
    stp d14, d15, [x0, #176]



__on_thread_exit:
    
    ret





.global    switch_context
.type      switch_context, "function"
.p2align   4

switch_context:

    // Store context
    // store sp and return label
    mov x2, sp
    ldr x3, __on_thread_exit
    
    stp x2, x3, [x0, #0]

    // We skip the argument pointer since 
    // the coroutine alreadys started
    

    // store frame pointer and link

    // General purpose registers
    stp x29, x30, [x0, #32]
    stp x27, x28, [x0, #48]
    stp x25, x26, [x0, #64]
    stp x23, x24, [x0, #80]
    stp x21, x22, [x0, #96]
    stp x19, x20, [x0, #112]
    

    // store d registers
    stp d8,  d9,  [x0, #128]
    stp d10, d11, [x0, #144]
    stp d12, d13, [x0, #160]
    stp d14, d15, [x0, #176]
    
    
    
    // Load context
    // General purpose registers
    ldp x29, x30, [x1, #32]
    ldp x27, x28, [x1, #48]
    ldp x25, x26, [x1, #64]
    ldp x23, x24, [x1, #80]
    ldp x21, x22, [x1, #96]
    ldp x19, x20, [x1, #112]

    // load d registers
    ldp d8,  d9,  [x1, #128]
    ldp d10, d11, [x1, #144]
    ldp d12, d13, [x1, #160]
    ldp d14, d15, [x1, #176]
    
    // load sp and function
    ldp x2, x3, [x0, #160]
    mov sp, x2
    
    // check if x29 has been initialized
    cbnz x29, jump_to_new_context    
    
    udf #0

    // initialize LN and FD to return to the parent original coroutine.
    ldp x29, x30, [x0, #16]






jump_to_new_context:
    // load argument if any
    str x0, [x0, #176]
    br x3

