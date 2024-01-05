.text
.global start_coroutine
.type   start_coroutine, @function
.align  16
.size   start_coroutine, 8
start_coroutine:
    .cfi_startproc
    .cfi_undefined x30
    mov x30, 0
    br x2    
    .cfi_endproc

    
.global switch_context
.type   switch_context, @function
.align  16
switch_context:
    
    // Store sp
    mov x3, sp
    str x3, [x0, #0]

    // Store eneral purpose registers
    stp x30, x29, [x0, #8]
    stp x28, x27, [x0, #24]
    stp x26, x25, [x0, #40]
    stp x24, x23, [x0, #56]
    stp x22, x21, [x0, #72]
    stp x20, x19, [x0, #88]



    // Store d registers
    stp d8,  d9,  [x0, #104]
    stp d10, d11, [x0, #120]
    stp d12, d13, [x0, #136]
    stp d14, d15, [x0, #152]

    // Load sp 
    ldr x3, [x1, #0]
    mov sp, x3
    
    // Load general purpose registers
    ldp x30, x29, [x1, #8]
    ldp x28, x27, [x1, #24]
    ldp x26, x25, [x1, #40]
    ldp x24, x23, [x1, #56]
    ldp x22, x21, [x1, #72]
    ldp x20, x19, [x1, #88]

    // Load d registers
    ldp d8,  d9,  [x1, #104]
    ldp d10, d11, [x1, #120]
    ldp d12, d13, [x1, #136]
    ldp d14, d15, [x1, #152]
    
    br x30
    

.global  switch_no_save
.align   16
switch_no_save:
    // Load sp 
    ldr x3, [x0, #0]
    mov sp, x3
    
    // Load general purpose registers
    ldp x30, x29, [x0, #8]
    ldp x28, x27, [x0, #24]
    ldp x26, x25, [x0, #40]
    ldp x24, x23, [x0, #56]
    ldp x22, x21, [x0, #72]
    ldp x20, x19, [x0, #88]

    // Load d registers
    ldp d8,  d9,  [x0, #104]
    ldp d10, d11, [x0, #120]
    ldp d12, d13, [x0, #136]
    ldp d14, d15, [x0, #152]
    
    br x30
    

