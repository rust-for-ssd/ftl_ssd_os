#ifndef LRING_H
#define LRING_H

#define RING_NAME_SIZE  32
#define CACHE_LINE_SIZE 64
#define __cache_aligned __attribute__((__aligned__(CACHE_LINE_SIZE)))

#define RING_FLAG_SINGLE_CONS (1 << 1)
#define RING_FLAG_SINGLE_PROD (1 << 2)
#define RING_FLAG_RUNNING	0x1

#define LOCKLESS_RING	1

static inline void memory_barrier_release(void)
{
    asm volatile ("fence rw,w" ::: "memory");
}

static inline void memory_barrier_acquire(void)
{
    asm volatile ("fence r,rw" ::: "memory");
}

static inline void cpu_relax_dynamic(int delay)
{
    for (volatile int i = 0; i < delay; i++) {
        asm volatile ("nop");
    }
}

static inline int compare_and_swap_ptr(volatile void **ptr,
				       volatile void *val,
				       volatile void *compare) {
    void *read = (void *) 0x0;
    int status = 0;

    asm volatile (
        "lr.w %[readv], (%[ptrv])\n"  // Load value from ptr
        : [readv] "=r" (read)         // Output
        : [ptrv] "r" (ptr)            // Input
    );

    if (compare != read)
        return 1; // Values don't match, return failure

    asm volatile (
        "sc.w %[statusv], %[valv], (%[ptrv])\n" // Store-conditional
        : [statusv] "=r" (status)               // Output
        : [valv] "r" (val), [ptrv] "r" (ptr)    // Inputs
    );

    return status; // 0 if successful, 1 if failed
}

struct lring {
    char  name[RING_NAME_SIZE]; /* Ring name */
    int	  alloc_mem;		/* Total memory allocated */
    void *ring_start;		/* Ring start address */
    void *last_entry;		/* Addr of last entry */
    int   ring_size;		/* Number of entries */
    int   entry_size;		/* Entry size */
    int   flags;
    int	  id;
    int sem;

    struct prod {
	int	     sp_enqueue; /* True if single producer */
	volatile void *head;
	volatile void *tail;
    } prod __cache_aligned;

    struct cons {
	int	     sc_dequeue; /* True if single consumer */
	volatile void *head;
	volatile void *tail;
    } cons __cache_aligned;
};

#ifndef LRE
#define LRE
struct lring_entry {
    void *function;
    void *ctx;
};
#endif /* LRE */

void lring_flag_on (struct lring *ring, int flag);
void lring_flag_off (struct lring *ring, int flag);
int lring_flag_check (struct lring *ring, int flag);

struct lring * lring_create (char *name, int size, void *mem, int flags);

static inline int lring_enqueue (struct lring *ring, struct lring_entry *entry)
{
    struct lring_entry *rentry;
    volatile void *prod_head;
    volatile void *cons_tail;
    volatile void *prod_next;

    prod_head = ring->prod.head;
    cons_tail = ring->cons.tail;

    prod_next = (prod_head == ring->last_entry) ?
		 ring->ring_start :
		 prod_head + sizeof (struct lring_entry);

    /* Check if ring is full */
    if (prod_next == cons_tail)
	return -1;

    /* This should be atomic if multiple producers */
    ring->prod.head = prod_next;

    /* Add object in the ring */
    rentry = (struct lring_entry *) ring->prod.tail;
    rentry->function = entry->function;
    rentry->ctx = entry->ctx;

    /* This should wait others if multiple producers */
    ring->prod.tail = prod_next;

    return 0;
}

/* Multiple producers */
static inline int lring_enqueue_m (struct lring *ring, struct lring_entry *entry)
{
    struct lring_entry *rentry;
    volatile void *prod_head;
    volatile void *cons_tail;
    volatile void *prod_next;

AGAIN:
    prod_head = ring->prod.head;
    cons_tail = ring->cons.tail;

    prod_next = (prod_head == ring->last_entry) ?
		 ring->ring_start :
		 prod_head + sizeof (struct lring_entry);

    /* Check if ring is full */
    if (prod_next == cons_tail)
	return -1;

    /* Atomic instruction */
    int status = compare_and_swap_ptr (&ring->prod.head,
				       prod_next,
				       prod_head);
    if (status)
	goto AGAIN;

    /* Add object in the ring */
    rentry = (struct lring_entry *) prod_head;
    rentry->function = entry->function;
    rentry->ctx = entry->ctx;

    memory_barrier_release();

    /* Wait for concurrent tasks */
    int delay = 1;
    while (ring->prod.tail != prod_head) {
	cpu_relax_dynamic(delay);
	if (delay < 1024)
	    delay *= 2;
    }

    ring->prod.tail = prod_next;

    return 0;
}

static inline int lring_dequeue (struct lring *ring, struct lring_entry *entry)
{
    struct lring_entry *rentry;
    volatile void *prod_tail;
    volatile void *cons_head;
    volatile void *cons_next;

    cons_head = ring->cons.head;
    prod_tail = ring->prod.tail;

    cons_next = (cons_head == ring->last_entry) ?
		 ring->ring_start :
		 cons_head + sizeof (struct lring_entry);

    /* Check if ring is empty */
    if (prod_tail == ring->last_entry) {
	if (cons_next == ring->ring_start)
	    return -1;
    } else {
	if (cons_next == prod_tail + sizeof (struct lring_entry))
	    return -1;
    }

    /* This should be atomic if multiple consumers */
    ring->cons.head = cons_next;

    /* Add object in the ring */
    rentry = (struct lring_entry *) ring->cons.tail;
    entry->function = rentry->function;
    entry->ctx = rentry->ctx;

    /* This should wait others if multiple consumers */
    ring->cons.tail = cons_next;

    return 0;
}

static inline int lring_dequeue_m (struct lring *ring, struct lring_entry *entry)
{
    struct lring_entry *rentry;
    volatile void *prod_tail;
    volatile void *cons_head;
    volatile void *cons_next;

AGAIN:
    cons_head = ring->cons.head;
    prod_tail = ring->prod.tail;

    cons_next = (cons_head == ring->last_entry) ?
		 ring->ring_start :
		 cons_head + sizeof (struct lring_entry);

    /* Check if ring is empty */
    if (cons_head == prod_tail)
	return -1;

    /* Atomic instruction */
    int status = compare_and_swap_ptr (&ring->cons.head,
				       cons_next,
				       cons_head);
    if (status)
	goto AGAIN;

    memory_barrier_acquire();

    /* Add object in the ring */
    rentry = (struct lring_entry *) cons_head;
    entry->function = rentry->function;
    entry->ctx = rentry->ctx;

    /* Wait for concurrent tasks */
    int delay = 1;
    while (ring->cons.tail != cons_head) {
	cpu_relax_dynamic(delay);
	if (delay < 1024)
	    delay *= 2;
    }

    ring->cons.tail = cons_next;

    return 0;
}

void lring_print (struct lring *ring);

#endif /* LRING_H */
