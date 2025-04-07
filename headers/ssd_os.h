#ifndef SSD_OS
#define SSD_OS

#include <stdint.h>

#ifndef NULL
#define NULL ((void*)0)
#endif

#define MAGIC_EXTENSION "ext"
#define MAGIC_STAGE     "stg"
#define MAGIC_CONNECTOR "con"

#ifndef LRE
#define LRE
struct lring_entry {
    void *function;
    void *ctx;
};
#endif /* LRE */

#ifndef TICKS
#define TICKS
#define TICKS_SEC	10000000
#define TICKS_MSEC	10000
#define TICKS_USEC	10
#endif /* TICKS */

typedef int    (ssd_os_extension_fn) (void *context);
typedef void  *(ssd_os_stage_fn)     (void *context);
typedef int    (ssd_os_ctrl_fn)      (void);
typedef int    (ssd_os_conn_ring_fn) 	  (struct lring_entry *entry);
typedef struct pipeline *(ssd_os_conn_fn) (struct lring_entry *entry);

struct extension {
    char 	         magic[4];
    char 	         name[32];
    ssd_os_extension_fn *extension_fn;
};

struct stage {
    char 	     magic[4];
    char 	     name[32];
    ssd_os_ctrl_fn  *init_fn;
    ssd_os_ctrl_fn  *exit_fn;
    ssd_os_stage_fn *stage_fn;
};

struct pipeline {
    char  name[32];
    void *internal[2];
};

struct connector {
    char 	     	 magic[4];
    char 	     	 name[32];
    ssd_os_ctrl_fn  	*init_fn;
    ssd_os_ctrl_fn  	*exit_fn;
    ssd_os_conn_fn  	*conn_fn;
    ssd_os_conn_ring_fn *ring_fn; 
};

/* DISTRIBUTION FUNCTIONS */

int  program_load    (uint8_t *elf, uint32_t size);
int  program_pipe    (uint8_t *file);
int  program_connect (uint8_t *file);

void loader_extensions_print (void);
void loader_stages_print (void);
void loader_connectors_print (void);

/* PROGRAMMING FUNCTIONS */

/* Printing */
void ssd_os_print_s     (const char* s);
void ssd_os_print_c     (char c);
void ssd_os_print_i     (unsigned long int x);
void ssd_os_print_h     (unsigned long int x);
void ssd_os_print_ss    (const char* s1, const char* s2); 
void ssd_os_print_ss_l  (const char* s1, const char* s2); 
void ssd_os_print_sis   (const char* s1, unsigned long int i, const char* s2); 
void ssd_os_print_sis_l (const char* s1, unsigned long int i, const char* s2);
void ssd_os_print_shs   (const char* s1, unsigned long int h, const char* s2); 
void ssd_os_print_shs_l (const char* s1, unsigned long int h, const char* s2);
void ssd_os_print_lock  (void);
void ssd_os_print_unlock (void);

/* Cores */
int ssd_os_ncores (void);
int ssd_os_this_cpu (char *name);

/* Memory */
void *ssd_os_mem_get      (int key);
int   ssd_os_mem_size     (int key);
int   ssd_os_mem_nregions (void);
void *ssd_os_mem_cpy (void *dest, const void *src, uint32_t n);

/* Timing */
void ssd_os_sleep  (int sec);
void ssd_os_msleep (int msec);
void ssd_os_usleep (int usec);
void ssd_os_timer_interrupt_on (int interval, void *function);
void ssd_os_timer_interrupt_off (void);

/* Lockless Ring */
struct lring *ssd_os_lring_create (char *name, int size,
				   void *mem, int flags);
int ssd_os_lring_enqueue   (struct lring *ring, struct lring_entry *entry);
int ssd_os_lring_enqueue_m (struct lring *ring, struct lring_entry *entry);
int ssd_os_lring_dequeue   (struct lring *ring, struct lring_entry *entry);
int ssd_os_lring_dequeue_m (struct lring *ring, struct lring_entry *entry);
void ssd_os_lring_print    (struct lring *ring);

/* Connections */
struct pipeline *ssd_os_get_connection (char *connector_name, char *pipe_name);

#endif /* SSD_OS */
