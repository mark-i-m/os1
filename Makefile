default : all;

run: all
	qemu-system-x86_64 -enable-kvm -nographic --serial mon:stdio -hdc kernel/kernel.img

rungraphic: all
	qemu-system-x86_64 -enable-kvm --serial mon:stdio -hdc kernel/kernel.img

% :
	(make -C kernel $@)
	#(make -C user $@)
