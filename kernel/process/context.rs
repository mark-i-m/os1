// Each process needs a kernel context and user context
//
// The user context is saved to its kernel stack when we
// switch in to kernel mode. The kernel context is saved
// to the process struct when we context switch.

#[derive(Clone, Copy)]
struct KContext {
    eax: usize,
    ecx: usize,
    edx: usize,
    ebx: usize,
    esp: usize,
    ebp: usize,
    esi: usize,
    edi: usize,
}
