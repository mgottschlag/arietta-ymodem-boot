
ENTRY(start)
SECTIONS
{
	. = 0x23000000;
	
	kernel_phys_start = .;
	
	.setup : {
		*(.setup)
	}

	.text : {
		*(.text)
		*(.text.*)
	}

	.rodata : {
		*(.rodata*)
	}
	.data : {
		*(.data*)
	}
	.bss : {
		bss_start = .;
		*(.bss*)
		*(COMMON)
		bss_end = .;
	}

	__exidx_start = .;
	.ARM.exidx : {
		*(.ARM.exidx* .gnu.linkonce.armexidx.*)
	}
	__exidx_end = .;

	kernel_phys_end = .;
}

