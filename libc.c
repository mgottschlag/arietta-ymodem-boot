
typedef unsigned int size_t;

void *memset(void *ptr, int c, size_t num) {
	size_t i;
	for (i = 0; i < num; i++) {
		((char*)ptr)[i] = c;
	}
	return ptr;
}

void __aeabi_memset(void *ptr, size_t num, int c) {
	memset(ptr, c, num);
}

void __aeabi_memclr4(void *ptr, size_t num) {
	memset(ptr, 0, num);
}

void *memmove(void *dest, const void *src, size_t num) {
	char *c_dest = dest;
	char *c_src = src;
	if ((unsigned int)dest < (unsigned int)src) {
		while (num > 0) {
			*c_dest++ = *c_src++;
			num--;
		}
	} else if ((unsigned int)dest > (unsigned int)src) {
		c_dest += num;
		c_src += num;
		while (num > 0) {
			*--c_dest = *--c_src;
			num--;
		}
	}
}

void __aeabi_memmove(void *dest, const void *src, size_t num) {
	memmove(dest, src, num);
}

