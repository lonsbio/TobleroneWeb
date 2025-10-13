// test.c
#include <pthread.h>
#include <stdio.h>
void* work(void* x){ puts("hi from thread"); return NULL; }
int main(){ pthread_t t; pthread_create(&t,0,work,0); pthread_join(t,0); puts("done"); }