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


_switch_context:
    // Store context
    // General purpose registers
    stp x19, x20, [x0, #0]
    stp x21, x22, [x0, #16]
    stp x23, x24, [x0, #32]
    stp x25, x26, [x0, #48]
    stp x27, x28, [x0, #64]
    stp x29, x30, [x0, #80]

    // store d registers
    stp d8,  d9,  [x0, #96]
    stp d10, d11, [x0, #112]
    stp d12, d13, [x0, #128]
    stp d14, d15, [x0, #144]
    
    // save sp and return function
    mov x1, sp
    adr x2, __on_thread_exit
    stp x1, x2, [x0, #160]
    
    // Load context
    // General purpose registers
    ldp x19, x20, [x1, #0]
    ldp x21, x22, [x1, #16]
    ldp x23, x24, [x1, #32]
    ldp x25, x26, [x1, #48]
    ldp x27, x28, [x1, #64]
    ldp x29, x30, [x1, #80]
    

    // store d registers
    ldp d8,  d9,  [x1, #96]
    ldp d10, d11, [x1, #112]
    ldp d12, d13, [x1, #128]
    ldp d14, d15, [x1, #144]

    // load sp and function
    ldp x2, x3, [x0, #160]
    mov sp, x2
    
    cbz x29, jump_to_new_context    
    
    // set return to link instead of stored registers
    ldp x29, x30, [x1, #160]


jump_to_new_context:
    // load argument if any
    str x0, [x0, #176]
    br x3



__on_thread_exit:

    ret



.global    _setcontext
.type      _setcontext, "function"
.p2align   4
_setcontext: 
    // General purpose registers
    ldp x19, x20, [x0, #0]
    ldp x21, x22, [x0, #16]
    ldp x23, x24, [x0, #32]
    ldp x25, x26, [x0, #48]
    ldp x27, x28, [x0, #64]
    ldp x29, x30, [x0, #80]
    

    // store d registers
    ldp d8,  d9,  [x0, #96]
    ldp d10, d11, [x0, #112]
    ldp d12, d13, [x0, #128]
    ldp d14, d15, [x0, #144]

    // load sp and jump to function
    ldp x2, x3, [x0, #160]
    mov sp, x2
    
    cbz x29, init_fp
    
continue:

    // load argument if any
    str x0, [x0, #176]
    
    br x3

init_fp:
    // set return to link instead of stored registers
    ldp x29, x30, [x1, #160]
    
    b continue


    
    


.global    _getcontext
.type      _getcontext, "function"
.p2align   4
_getcontext:
    // General purpose registers
    stp x19, x20, [x0, #0]
    stp x21, x22, [x0, #16]
    stp x23, x24, [x0, #32]
    stp x25, x26, [x0, #48]
    stp x27, x28, [x0, #64]
    stp x29, x30, [x0, #80]

    // store d registers
    stp d8,  d9,  [x0, #96]
    stp d10, d11, [x0, #112]
    stp d12, d13, [x0, #128]
    stp d14, d15, [x0, #144]
    
    // save sp and return function
    mov x1, sp
    adr x2, __return
    stp x1, x2, [x0, #160]
    ret
__return:
    udf #0
    ret

