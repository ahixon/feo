# use ld -v to find arches to use for clang or ld or whatever

.global reset_handler
interrupt_table:
	b jmp_reset_handler

jmp_reset_handler:
	ldr sp, =stack_top
	bl reset_handler
	b .