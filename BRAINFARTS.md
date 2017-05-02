===

When do we turn on DRAM? Otherwise we only have SRAM
	- SRAM B on A80 is 256KB, 64KB on A10
	- SRAM A1 + A2 on A80 is 200KB, 32KB on A10
	- so, anywhere between 96KB and 456KB
	- 256KB might just be enough, 456KB is comfortable; but keep in mind that's got to allow for all capabilities
	- 96KB is probably enough enough tho?????

on x86 not such a big deal since BIOS/UEFI is meant to set this up for us

When do we setup PMIC and clocks?
	well, we need that to do DRAM, so.
	nope, boot0 does that for us :')

Can have some system call to provide virtual memory space to use? so that "user space" can do this stuff? and if we have no more room just fail to say we can't make any new capabilities?


===

* take control of interrupt table
	- ie register for all available interrupts
* make sure they're turn them off
* setup internal capabilities for VM and Interrupts and message passing between processes
	- maybe scheduling, but policy should not be the hypervisor's problem
* jump to "user space" with root capability
* how to control not passing memory designated for µkernel?
	- can ask for virtual addressing, or control of physical memory
	- but if it's physical memory than we do a map for them (DMA)
	- and just ensure that the requested region doesn't overlap with us
	or the interrupt table

now, this main server can:
* provide ELF loading + introspection to the .ar so it can load some needed boot programs
* process scheduling + process resource limiting (ie process #, etc)

* registers ourselves to receive the all interrupts

* loads the in-built timer server, given a capability for the timer interrupt, and some message capability
	- this relies on the assumption that the timer service will always call us back when the scheduler tick comes through

* serial port server

* DMA can be separate server iff it can validate capabilities/can have a guard for capabilities and have the kernel reject messages that don't satisfy the guard (saves on context switches)

* lightweight message passing - sets process flag in own memory, and yields (syscall)
* heavyweight message passing - syscall on endpoint? same same?
	- if we put timer in kernel we can do timeouts properly