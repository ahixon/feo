ENTRY(_reset);

SECTIONS {
  . = 0x10000;

  .text : {
    *(.text._reset)
    *(.text)
  }

  .data : {
    *(.data)
  }

  .bss : {
    *(.bss)
  }

  . = ALIGN(8);
  . = . + 0x4000; /* 16kB of stack memory */

  stack_top = .;

  /DISCARD/ : {
    *(.ARM.*)
    *(.note.gnu.build-id*)
  }
}