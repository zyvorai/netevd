# SPDX-License-Identifier: Apache-2.0
CLANG ?= clang
ARCH ?= $(shell uname -m | sed 's/x86_64/x86/' | sed 's/aarch64/arm64/')
# Debian/Ubuntu put asm/ under the multiarch triplet; clang -target bpf does not.
MULTIARCH_INC ?= /usr/include/$(shell dpkg-architecture -qDEB_HOST_MULTIARCH 2>/dev/null || echo x86_64-linux-gnu)
CFLAGS ?= -O2 -g -Wall -target bpf -D__TARGET_ARCH_$(ARCH) -I$(MULTIARCH_INC)

netevd-ebpf.o: netevd_observe.c
	$(CLANG) $(CFLAGS) -c $< -o $@

clean:
	rm -f netevd-ebpf.o
