#ifndef FDK_H
#define FDK_H

#define CORE_FN_ADDR 		0x810FD000 /* Holds user function pointers */
#define CORE_FN_SIZE 		12
#define CORE_FN_STATUS_OFF	4
#define CORE_FN_ARG_OFF		8

#define CORE_STACK_START	0x81100000	/* Core 0 stack start */
#define CORE_STACK_SIZE		0x20000		/* 128 KiB */

#define CORES_REG		0x810FCFFC	/* Number of cores register */

#ifndef TICKS
#define TICKS
#define TICKS_SEC	10000000
#define TICKS_MSEC	10000
#define TICKS_USEC	10
#endif /* TICKS */

#include <stdint.h>

typedef void (cpu_thread) (int cpu, void *opaque);

struct thread_data
{
    int 	  cpu;		/* CPU id */
    int		  mem_size;	/* Memory to allocate */
    void	  *opaque;	/* Opaque pointer passed to function */
    cpu_thread    *fn;		/* Program function pointer */
};

int main (int argc, char **argv);

void start (void);
void start_cpu (void);

void cpu_init (void);
void cpu_wfi_enter (void);
void cpu_wfi_exit (void);
int  cpu_get_free (unsigned int lock);
int  cpu_get_balanced (void);
int  cpu_get_exact (int cpu);
void cpu_release (int cpu);
void cpu_print (void);
int  cpu_get_load (int cpu);
int  cpu_reserve (int amount);
int  cpu_free_count (void);
struct lring *cpu_get_ring (int cpu);
struct lring *cpu_new_ring (void);

int thread_start (struct thread_data *data, uint8_t force);
int thread_next (struct thread_data *thread);
int thread_waiting (void);
int thread_schedule (void *function, void *ctx);

int is_core_busy (int cpu);

void  memory_init (int nslices);

void interrupt_setup (void);

#endif /* FDK_H */
