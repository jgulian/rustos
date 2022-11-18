
int my_strlen(char *str) {
    int i = 0;
    while (str[i]) i++;
    return i;
}

void my_puts(char *string) {
    unsigned long ptr = (unsigned long) string;
    unsigned long len = my_strlen(string);
    asm("mov x0, #0");
    asm("mov x1, %0" : :"r"(ptr));

    asm("mov x0, #0 \n\t"
        "mov x1, %0\n\t"
        "mov x2, %1\n\t"
        "svc 5"
        :: "r" (ptr), "r" (len)
        : "x0", "x1", "x2"
        );

}

int _start() {
    char *message = "Hello World";
    my_puts(message);
    return 0;
}