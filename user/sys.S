	#
	# user-side system calls
	#
	# System calls use a special convention:
        #     %eax  -  system call number
        #     %ecx  -  first argument
        #     %edx  -  second argument
        #
        # more arguments could be passed in memory
        #

	# void exit(int status)
	.global exit
exit:
	mov $0,%eax
	mov 4(%esp),%ecx
	mov $0,%edx
	int $100
	ret

	# void start_tests()
	.global start_tests
start_tests:
	mov $1,%eax
	mov $0,%ecx
	mov $0,%edx
	int $100
	ret
