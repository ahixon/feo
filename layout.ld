ENTRY(main);

SECTIONS {
  . = 0x200000;

  .text : ALIGN(4) {
    *(.text .text.*);
  }

  .rodata : ALIGN(4)
  {
    *(.rodata .rodata.*);
  }

  .data : ALIGN(4) {
    *(.data .data.*);
  }

  .bss : ALIGN(4) {
    *(.bss .bss.*);
  }

  /*. = ALIGN(8);
  . = . + 0x4000;

  stack_top = .;*/

  debug_gdb_scripts 0 (INFO) : {
    KEEP(*(.debug_gdb_scripts))
  }

  /DISCARD/ : {
    *(.ARM.*)
    *(.note.gnu.build-id*)
  }
}