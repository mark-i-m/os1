typedef void (*exec_test_t)(long);

int main() {
    exec_test_t exec_test = (exec_test_t)0x26b60;

    exec_test(23);

    return 0;
}
