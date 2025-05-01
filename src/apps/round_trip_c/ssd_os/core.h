#ifndef CORE
#define CORE

#include <stdint.h>
#include <elf.h>
#include "lring.h"

#ifndef NULL
#define NULL ((void*)0)
#endif

#define MAX_STR_SIZE 	256

#define COMPARE_AND_SWAP_INT(ptr, val, compare, status) do {	\
    int read = 0, cur = 0;					\
    status = 1;							\
    asm goto ("lr.w	%[readv], (%[ptrv])\n\t"		\
	      "bne   %[comparev], %[readv], %l[FAIL]\n\t"	\
	      "sc.w	%[statusv], %[valv], (%[ptrv])"		\
	      : [curv] "+r" (cur), [readv] "=r" (read),		\
		[ptrv] "+rm" (ptr), [statusv] "=r" (status)	\
	      : [valv] "r" (val), [comparev] "r" (compare)	\
	      :							\
	      : FAIL);						\
FAIL:								\
} while (/*CONSTCOND*/0)

static inline void atomic_store_int (int ptr, int val)
{
    int read, status;

AGAIN:
    asm goto ("lr.w	%0, (%1)\n\t"
	      "sc.w	%2, %3, (%1)\n\t"
	      "bne	%2, zero, %l[AGAIN]"
	      : "=r" (read), "+rm" (ptr), "=r" (status)
	      : "r" (val)
	      :
	      : AGAIN);
}

static inline void atomic_increment (int ptr, int val)
{
    int read, status, sum = 0;

AGAIN:
    asm goto ("lr.w	%[readv], (%[ptrv])\n\t"
	      "add	%[sumv], %[readv], %[valv]\n\t"
	      "sc.w	%[statusv], %[sumv], (%[ptrv])\n\t"
	      "bne	%[statusv], zero, %l[AGAIN]"
	      : [readv] "=r" (read), [ptrv] "+rm" (ptr),
		[statusv] "=r" (status), [sumv] "+r" (sum)
	      : [valv] "r" (val)
	      :
	      : AGAIN);
}

static inline void semaphore_init (int *sem)
{
    *sem = 0;
}

static inline void semaphore_lock (int *sem)
{
    int status, lock, locked = 1;
AGAIN:
    asm goto ("addi	%[lockedv], zero, 1\n\t"
	      "lr.w	%[lockv], (%[addrv])\n\t"
	      "bne	%[lockv], zero, %l[AGAIN]\n\t"
	      "sc.w	%[statusv], %[lockedv], (%[addrv])\n\t"
	      "bne	%[statusv], zero, %l[AGAIN]"
	      : [lockedv] "+r" (locked), [lockv] "=r" (lock),
		[addrv] "+rm" (sem), [statusv] "=r" (status)
	      :
	      :
	      : AGAIN);
}

static inline void semaphore_unlock (int *sem)
{
    *sem = 0;
}

static inline void *mem_cpy(void *dest, const void *src, uint32_t n) {
    uint32_t i;
    unsigned long *d_word = (unsigned long *)dest;
    const unsigned long *s_word = (const unsigned long *)src;

    for (i = 0; i < n / sizeof(unsigned long); i++) {
        d_word[i] = s_word[i];
    }

    unsigned char *d_byte = (unsigned char *)(d_word + i);
    const unsigned char *s_byte = (const unsigned char *)(s_word + i);

    for (i = 0; i < n % sizeof(unsigned long); i++) {
        d_byte[i] = s_byte[i];
    }

    return dest;
}

static inline int str_cpy (char *dest, char *src)
{
    int count = 0;
    char *c = src;
    while (*c != '\0' && count < MAX_STR_SIZE) {
	dest[count] = *c;
	count++;
	c = &src[count];
    }
    return count;
}

static inline int str_cmp(const char *s1, const char *s2)
{
    while (*s1 && (*s1 == *s2)) {
        s1++;
        s2++;
    }
    return *(unsigned char *)s1 - *(unsigned char *)s2;
}

static inline uint16_t cpu_to_le16 (uint16_t value)
{
#if __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
    return value;  // No conversion needed
#else
    return (value >> 8) | (value << 8);  // Swap bytes
#endif
}

static inline uint16_t le16_to_cpu(uint16_t value)
{
#if __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
    return value;  // Already in correct format
#else
    return (value >> 8) | (value << 8);  // Swap bytes
#endif
}

struct connector_data;
typedef struct pipe_data *(connector_fn) (struct lring_entry *entry);
typedef int (connector_ring_fn) (struct lring_entry *entry);
typedef int  (connector_init_fn) (struct connector_data *sdata);
typedef int (stage_init_fn) (void);
typedef void *(pipe_fn_type)(void *);

struct program_md
{
    char name[32];
    union {
	struct {
	    void *entry_point;
	    void *reserved[4];
	} ext;

	struct {
	    void *init_fn;
	    void *exit_fn;
	    void *entry_point;
	    void *data;
	    void *reserved;
	} stage;

	struct {
	    void *init_fn;
	    void *exit_fn;
	    void *ring_fn;
	    void *conn_fn;
	    void *data;
	} conn;

	struct {
	    void *reserved[5];
	} ptr;
    };
    struct {
	struct program_md *sqe_next;
    } entry;
}; /* 54-80 bytes */

struct connector_data
{
    struct lring      *ring;
    uint32_t	       nosched;
    connector_fn      *conn_fn;
    connector_ring_fn *ring_fn;
    connector_init_fn *init_fn;
    struct program_md *conn_md;
}; /* 20 bytes */

struct pipe_stage
{
    int 		key;
    int 		stage;
    void 	       *function;
    stage_init_fn      *init_fn;
    struct program_md  *stage_md;
    struct pipe_data   *pipe;
    struct lring       *ring;
    struct {
	struct pipe_stage *sqe_next;
    } entry;
}; /* 28 bytes */

struct pipe_data
{
    int nstages;
    struct connector_data *endpoint;
    struct {
	struct pipe_stage  *sqh_first;
	struct pipe_stage **sqh_last;
	int		   sem;
    } stage_head;
}; /* 16 bytes */

struct pipe_md
{
    char  name[32];
    struct pipe_data *data;
    struct {
	struct pipe_md *sqe_next;
    } entry;
}; /* 40-48 bytes */

void print_s(const char* s);
void print_c(char c);
void print_i(unsigned long int x);
void print_h(unsigned long int x);
void print_ss (const char* s1, const char* s2); 
void print_ss_l (const char* s1, const char* s2); 
void print_sis (const char* s1, unsigned long int i, const char* s2); 
void print_sis_l (const char* s1, unsigned long int i, const char* s2); 
void print_shs (const char* s1, unsigned long int h, const char* s2); 
void print_shs_l (const char* s1, unsigned long int h, const char* s2); 
void print_lock (void);
void print_unlock (void);

int   get_ncores (void);
void *memory_get_addr (int key);
void *memory_get_program_md (void);
void *memory_get_pipe_md (void);
int   memory_get_size (int key);
int   memory_total_size (void);
int   memory_nregions (void);
int   memory_get_program_slot (unsigned int program_size);
void  memory_free_program_slot (int id);
void *memory_get_program (int id);
void  sleep (int sec);
void  msleep (int msec);
void  usleep (int usec);

struct pipe_data  *pipeline_create (char *name);
int		   pipeline_destroy (struct pipe_data *pipe);
struct pipe_stage *pipeline_add_stage (struct pipe_data *pipe,
				       struct program_md *stage_md);
int  		   pipeline_deploy (struct pipe_data *pipe);
void 		   pipeline_print  (struct pipe_data *pipe);
int 		   pipeline_send (struct pipe_data *pipe, void *ctx);
struct pipe_md 	  *pipeline_md_get (char *name);
void 		   pipeline_endpoint (struct pipe_data *pipe,
				      struct connector_data *connector);

struct connector_data *connector_create (struct program_md *md, uint32_t nosched);
int    connector_deploy (struct connector_data *data);
struct pipe_md *connector_connection_get (char *connector_name,
					  char *pipe_name);
int    connector_connection_add (struct connector_data *conn,
				 struct pipe_md *pipe);
int    connector_connection_remove (char *connector_name, char *pipe_name);
void   connector_boot (void);

uint32_t    elf_shdr_size (uint8_t *elf);
Elf32_Shdr *elf_shdr_get (uint8_t *elf);
Elf32_Shdr *elf_shdr_get_by_name (uint8_t *elf, char *name);
int 	    elf_shdr_get_id_by_name (uint8_t *elf, char *name);
char 	   *elf_shstrtab_get (uint8_t *elf);
uint8_t    *elf_section_get (uint8_t *elf, char *section);
char 	   *elf_shstrtab_str (uint8_t *elf, uint32_t index);
char 	   *elf_strtab_str (uint8_t *elf, uint32_t index);
uint32_t    elf_segments_get (uint8_t *elf, Elf32_Phdr *phdr);

int  loader_init (void);
void loader_extensions_print (void);
void loader_stages_print (void);
void loader_connectors_print (void);
struct program_md *loader_ext_get (char *name);
struct program_md *loader_stg_get (char *name);
struct program_md *loader_con_get (char *name);

int linker_relocate (uint8_t *elf, uint8_t *refe_section,
		     uint8_t *refe_segment, uint8_t *symb_segment,
		     Elf32_Rela *rel, Elf32_Sym *symbol);

int linker_relocate_lo12 (uint8_t * elf, uint8_t *refe_section,
	    uint8_t *refe_segment, uint8_t *symb_segment_hi20,
	    Elf32_Rela *rel, Elf32_Rela *rel_hi20);

int  program_load    (uint8_t *elf, uint32_t size);
int  program_pipe    (uint8_t *file);
int  program_connect (uint8_t *file);

int scheduler_init (void);
#endif /* CORE */
