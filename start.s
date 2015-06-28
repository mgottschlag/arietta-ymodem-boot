.global start
.extern main
.extern bss_start
.extern bss_end
.section .setup, "ax"

start:
	mov sp, #0x21000000
	/* clear .bss */
	mov r1, #0
	ldr r4, bss_region
	ldr r5, bss_region+4
bss_loop:
	cmp r4, r5
	bge bss_done
	str r1, [r4]
	add r4, r4, #4
	b bss_loop
bss_done:
	/* if start() returns, we just enter a loop */
	blx main
	b .
bss_region:
	.word bss_start
	.word bss_end

