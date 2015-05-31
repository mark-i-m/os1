# #define ASSEMBLY
#
# 	# syscallTrap()
# 	# the user implicitly passes 3 arguments in %eax, %ecx, %edx
# 	# %eax is syscall number
# 	# the rest of the arguments depend on %eax
# 	.global syscallTrap
# syscallTrap:
# 	push %ds
#
# 	push %ebp
# 	push %edi
# 	push %esi
# 	push %ebx
#
# 	push %edx
# 	push %ecx
# 	push %eax
#
# 	mov %esp,%eax                         // why?
# 	push %eax
#
# /*
#     Arguments:
#
#             context
# context->   syscall #
#             a0
#             a1
#             user %ebx
#             user %esi
#             user %edi
#             user %ebp
#             user %ds
#             user %eip
#             user %cs
#             user %eflags
#             user %esp
#             user %ss
#
# */
#
#
# 	mov kernelDataSeg,%eax
# 	movw %ax,%ds
#
# 	.extern syscallHandler
# 	call syscallHandler
#
# 	add $32,%esp
#
# 	pop %ds
# 	iret
#
#
# 	# switchToUser(pc,esp,eax)
# 	.global switchToUser
# switchToUser:
# 	mov 4(%esp),%ecx	# pc
# 	mov 8(%esp),%ebx        # esp
# 	mov 12(%esp),%eax       # eax
#
# 	mov userDataSeg,%edx    # for user SS = DS
# 	push %edx               # ring 3 SS
#
# 	push %ebx               # user ESP
#
# 	pushf                   # ring 3 flags
#         orl $0x200,(%esp)       # enable interrupts
#
# 	mov userCodeSeg,%edx
#         push %edx               # ring 3 CS
#
# 	mov userDataSeg,%edx
#
# 	push %ecx               # user EIP
#
# 	movw %dx,%ds            # no more memory refs until iret
#
#         iret
#
#
#
# 	# oldval = cs32(uint32_t* ptr, uint32_t ifval, uint32_t thenval)
# 	.global cs32
# cs32:
# 	mov 4(%esp),%ecx       # ptr
# 	mov 8(%esp),%eax       # ifval
# 	mov 12(%esp),%edx      # thenval
#         lock cmpxchgl %edx,(%ecx)
# 	ret                    # looks strange but always returns the oldval
#                                #   - if it worked, %eax had the ifval which
#                                #     is equals to the oldval
#                                #   - if it failed, %eax will be loaded with
#                                #     the oldval

    # pub fn proc_yield(q: Option<&mut Queue<Box<Process>>>);
    .extern _proc_yield
    .global proc_yield
proc_yield:
    // push all regs
    pusha

    // EDI          <= (new) ESP
    // ESI
    // EBP
    // (orig.) ESP
    // EBX
    // EDX
    // ECX
    // EAX
    //              <= (orig.) ESP

    push %esp

    call store_kcontext       // Save to context struct

    // pop the stuff we pushed
    pop %esp
    popa

    call _proc_yield

    ret 

    # pub fn save_kcontext()
    .extern store_kcontext
    .global save_kcontext
save_kcontext:
    // push all regs
    pusha

    // EDI          <= (new) ESP
    // ESI
    // EBP
    // (orig.) ESP
    // EBX
    // EDX
    // ECX
    // EAX
    //              <= (orig.) ESP

    push %esp

    call store_kcontext       // Save to context struct

    // pop the stuff we pushed
    pop %esp
    popa

    ret

    # context_switch(next_context: KContext, eflags: usize)
    .global context_switch
context_switch:

    // get args
    leal 4(%esp), %eax      // ptr to next_context
    mov 12(%esp), %ecx      // eflags

    // restore the next context
    mov   (%eax), %edi
    mov  4(%eax), %esi
    mov  8(%eax), %ebp
    mov 12(%eax), %esp       // ok to change esp here, since we are using eax as ptr
    mov 16(%eax), %ebx
    mov 20(%eax), %edx

	// get the new flags, mostly used for interrupt flag
    push %ecx
    popf

    // restore ecx and eax here, since we used them
    mov 24(%eax), %ecx
    mov 28(%eax), %eax

    ret

	# uint32_t fetchAndAdd32(uint32_t* ptr, uint32_t v)
	.global fetchAndAdd32
fetchAndAdd32:
	mov 4(%esp),%ecx		# ptr
	mov 8(%esp),%eax		# v
	lock xaddl %eax,(%ecx)
	ret

    # eflags
    .global eflags
eflags:
	pushf
	pop %eax
	ret

	# outb(int port, int val)
	.global outb
outb:
	push %edx
	mov 8(%esp),%dx
	mov 12(%esp),%al
	outb %al,%dx
	pop %edx
	ret

	# int inb(int port)
	.global inb
inb:
	push %edx
	mov 8(%esp),%dx
	inb %dx,%al
	pop %edx
	and $0xff,%eax
	ret

	# unsigned long inb(int port)
	.global inl
inl:
	push %edx
	mov 8(%esp),%dx
	inl %dx,%eax
	pop %edx
	ret

#
# 	#
# 	# void ltr(uint32_t tr)
# 	#
# 	.global ltr
# ltr:
# 	mov 4(%esp),%eax
# 	ltr %ax
# 	ret
#
# 	/* vmm_on(pd) */
# 	.global vmm_on
# vmm_on:
# 	mov 4(%esp),%eax
# 	mov %eax,%cr3
#
# 	mov %cr0,%eax
# 	or $0x80000000,%eax
# 	mov %eax,%cr0
# 	ret
#
# 	.global getcr0
# getcr0:
# 	mov %cr0,%eax
# 	ret
#
# 	.global getcr3
# getcr3:
# 	mov %cr3,%eax
# 	ret
#
# 	.global invlpg
# invlpg:
# 	mov 4(%esp),%eax
# 	invlpg (%eax)
# 	ret
#
# 	.global cli
# cli:
# 	cli
# 	ret
#
# 	.global sti
# sti:
# 	sti
# 	ret
#
# 	.global irq0
# irq0:
# 	push %eax
# 	mov $0, %eax
# 	jmp irq_common
#
# 	.global irq1
# irq1:
# 	push %eax
# 	mov $1, %eax
# 	jmp irq_common
#
# 	.global irq2
# irq2:
# 	push %eax
# 	mov $2, %eax
# 	jmp irq_common
#
# 	.global irq3
# irq3:
# 	push %eax
# 	mov $3, %eax
# 	jmp irq_common
#
# 	.global irq4
# irq4:
# 	push %eax
# 	mov $4, %eax
# 	jmp irq_common
#
# 	.global irq5
# irq5:
# 	push %eax
# 	mov $5, %eax
# 	jmp irq_common
#
# 	.global irq6
# irq6:
# 	push %eax
# 	mov $6, %eax
# 	jmp irq_common
#
# 	.global irq7
# irq7:
# 	push %eax
# 	mov $7, %eax
# 	jmp irq_common
#
# 	.global irq8
# irq8:
# 	push %eax
# 	mov $8, %eax
# 	jmp irq_common
#
# 	.global irq9
# irq9:
# 	push %eax
# 	mov $9, %eax
# 	jmp irq_common
#
# 	.global irq10
# irq10:
# 	push %eax
# 	mov $10, %eax
# 	jmp irq_common
#
# 	.global irq11
# irq11:
# 	push %eax
# 	mov $11, %eax
# 	jmp irq_common
#
# 	.global irq12
# irq12:
# 	push %eax
# 	mov $12, %eax
# 	jmp irq_common
#
# 	.global irq13
# irq13:
# 	push %eax
# 	mov $13, %eax
# 	jmp irq_common
#
# 	.global irq14
# irq14:
# 	push %eax
# 	mov $14, %eax
# 	jmp irq_common
#
# 	.global irq15
# irq15:
# 	push %eax
# 	mov $15, %eax
# 	jmp irq_common
#
# irq_common:
# 	push %ebx
# 	push %ecx
# 	push %edx
# 	push %esi
# 	push %edi
# 	push %ebp
#
# 	push %ds
#
# 	mov kernelDataSeg,%ecx
# 	mov %cx,%ds
#
# 	mov %cr2,%ebp
# 	push %ebp
#
#     mov %esp, %edx
#
#     # edx points to the saved user context
#     push %edx
# 	push %eax
#
# 	.extern pic_irq
# 	call pic_irq
#
# 	add $8,%esp    /* pop arguments */
#
# 	pop %ebp
# 	mov %ebp,%cr2
#
# 	pop %ds
#
# 	pop %ebp
# 	pop %edi
# 	pop %esi
# 	pop %edx
# 	pop %ecx
# 	pop %ebx
# 	pop %eax
# 	iret
#
# # PIT
#
# 	/* pit_init(divide) */
# 	.global pit_do_init
# pit_do_init:
# 	pushf			# push IF
# 	cli			# disable interrupts
# 	movb $0b00110100,%al	# 00 (channel 0)
# 				# 110 (lobyte/hibyte)
# 				# 100 (rate generator)
# 	outb %al,$0x43		# write command
# 	movb 8(%esp),%al	# divide
#         outb %al,$0x40
# 	movb 9(%esp),%al
# 	outb %al,$0x40
# 	popf			# pop IF
# 	ret
#
#
# 	.global pageFaultHandler
# pageFaultHandler:
#         push %eax
#         push %ebx
#         push %ecx
#         push %edx
#         push %esi
#         push %edi
#         push %ebp
#
# 	/* switch to kernel DS */
# 	push %ds
# 	mov kernelDataSeg,%eax
# 	mov %ax,%ds
#
#
#         mov %cr2,%eax   /* address */
#         push %eax
#
# 	mov %esp, %eax
# 	push %eax
#
#         .extern vmm_pageFault
#         call vmm_pageFault
#
#         add $8,%esp    /* pop argument */
#
#         /* restore DS */
#         pop %ds
#
#         pop %ebp
#         pop %edi
#         pop %esi
#         pop %edx
#         pop %ecx
#         pop %ebx
#         pop %eax
#         add $4,%esp   /* pop error */
#         iret
#
#     # void sys_sigret(sigcontext context)
#     .global sys_sigret
# sys_sigret:
#     mov 4(%esp), %eax   # load the frame pointer
#     mov %eax, %esp
#
# 	pop %ebp
# 	mov %ebp,%cr2
#
# 	pop %ds
#
# 	pop %ebp
# 	pop %edi
# 	pop %esi
# 	pop %edx
# 	pop %ecx
# 	pop %ebx
# 	pop %eax
# 	iret
#
#     #    cr2 <= context
# 	#    ds;
# 	#    ebp;
# 	#    edi;
# 	#    esi;
# 	#    edx;
# 	#    ecx;
# 	#    ebx;
# 	#    eax;
# 	#    eip;
# 	#    cs;
# 	#    flags;
# 	#    esp;
# 	#    ss;
