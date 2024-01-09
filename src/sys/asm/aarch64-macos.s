.global _start_coroutine
.text
_start_coroutine:     
    mov x30, #0
    mov x0, x1 
    br x19
    ret


.global    _switch_context
.p2align   4
_switch_context:
    
    // # Store context
    // store sp 
    mov x2, sp
    str x2, [x0, #0]

    // General purpose registers
    stp x29, x30, [x0, #8]
    stp x27, x28, [x0, #24]
    stp x25, x26, [x0, #40]
    stp x23, x24, [x0, #56]
    stp x21, x22, [x0, #72]
    stp x19, x20, [x0, #88]
   

    // store d registers
    stp d8,  d9,  [x0, #104]
    stp d10, d11, [x0, #120]
    stp d12, d13, [x0, #136]
    stp d14, d15, [x0, #152]

    // # Load context
    
    // load sp 
    ldr x2, [x1, #0]
    mov sp, x2

    // General purpose registers
    ldp x29, x30, [x1,  #8]
    ldp x27, x28, [x1,  #24]
    ldp x25, x26, [x1,  #40]
    ldp x23, x24, [x1,  #56]
    ldp x21, x22, [x1,  #72]
    ldp x19, x20, [x1,  #88]

    // load d registers
    ldp d8,  d9,  [x1, #104]
    ldp d10, d11, [x1, #120]
    ldp d12, d13, [x1, #136]
    ldp d14, d15, [x1, #152]

    

    br x30
    
