	.global __reset
	.extern __vector_table
	.extern _start_rs
	.extern __data_start
	.extern __data_end
	.extern __rodata_start
	.extern __rodata_end
	.extern __bss_start
	.extern __bss_end
	.extern __stack_bottom

	.section .text
	.thumb
	.thumb_func
	.align 2
__reset:
	// copy rodata => data
	ldr	r0, =__data_start
	ldr	r1, =__data_end
	ldr	r2, =__rodata_start
1:	ldr	r3, [r2]
	str	r3, [r0]
	cmp	r0, r1
	blt	1b

	// init bss
	ldr	r0, =__bss_start
	ldr	r1, =__bss_end
	mov	r3, #0
2:	str	r3, [r0]
	cmp	r0, r1
	blt	2b

	ldr	r0, =__stack_bottom
	mov	sp, r0

	// disable irq
	cpsid	i

	// set VTOR
	ldr	r0, =__vector_table
	ldr	r1, =0xE000ED08
	str	r0, [r1]

	ldr	r0, =_start_rs
	bx	r0
