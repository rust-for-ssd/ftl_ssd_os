#include "ssd_os/ssd_os.h"
#include <stdint.h>
#include "ssd_os/lring.h"

static struct lring *conn2_lring;
static uint32_t amount, count, submitted, last_count;
static struct pipeline *pipe1;
static struct pipeline *pipe2;

struct numbers {
    uint8_t value;
    uint8_t add;
    uint16_t id;
};

#define MAX_NUMBERS 256
static struct numbers numbers_list[MAX_NUMBERS];
static uint32_t numbers_index = 0;

/* Helper function to get a numbers structure from the list */
static struct numbers* get_number(uint16_t id) {
    /* Simple lookup - in a real system you might want a more efficient approach */
    for (uint32_t i = 0; i < numbers_index; i++) {
        if (numbers_list[i].id == id) {
            return &numbers_list[i];
        }
    }
    return NULL;
}

/* Add function that works with a pointer to numbers struct */
void* add(struct numbers* n) {
    if (n) {
        n->value += n->add;
    }
    return n;
}

/* Stage functions that use the numbers structure */
void *stage1_1_fn(void *ctx) { 
    struct numbers* n = (struct numbers*)ctx;
    return add(n);
}
void *stage1_2_fn(void *ctx) { 
    struct numbers* n = (struct numbers*)ctx;
    return add(n);
}
void *stage1_3_fn(void *ctx) { 
    struct numbers* n = (struct numbers*)ctx;
    return add(n);
}
void *stage1_4_fn(void *ctx) { 
    struct numbers* n = (struct numbers*)ctx;
    return add(n);
}
void *stage2_1_fn(void *ctx) { 
    struct numbers* n = (struct numbers*)ctx;
    return add(n);
}
void *stage2_2_fn(void *ctx) { 
    struct numbers* n = (struct numbers*)ctx;
    return add(n);
}
void *stage2_3_fn(void *ctx) { 
    struct numbers* n = (struct numbers*)ctx;
    return add(n);
}
void *stage2_4_fn(void *ctx) { 
    struct numbers* n = (struct numbers*)ctx;
    return add(n);
}

int stage_init_fn(void) { return 0; }
int stage_exit_fn(void) { return 0; }

struct stage stage1_1 = {
    .magic   = MAGIC_STAGE,
    .name    = "stage1_1",
    .stage_fn = stage1_1_fn,
    .init_fn = stage_init_fn,
    .exit_fn = stage_exit_fn
};

struct stage stage1_2 = {
    .magic   = MAGIC_STAGE,
    .name    = "stage1_2",
    .stage_fn = stage1_2_fn,
    .init_fn = stage_init_fn,
    .exit_fn = stage_exit_fn
};

struct stage stage1_3 = {
    .magic   = MAGIC_STAGE,
    .name    = "stage1_3",
    .stage_fn = stage1_3_fn,
    .init_fn = stage_init_fn,
    .exit_fn = stage_exit_fn
};

struct stage stage1_4 = {
    .magic   = MAGIC_STAGE,
    .name    = "stage1_4",
    .stage_fn = stage1_4_fn,
    .init_fn = stage_init_fn,
    .exit_fn = stage_exit_fn
};

struct stage stage2_1 = {
    .magic   = MAGIC_STAGE,
    .name    = "stage2_1",
    .stage_fn = stage2_1_fn,
    .init_fn = stage_init_fn,
    .exit_fn = stage_exit_fn
};

struct stage stage2_2 = {
    .magic   = MAGIC_STAGE,
    .name    = "stage2_2",
    .stage_fn = stage2_2_fn,
    .init_fn = stage_init_fn,
    .exit_fn = stage_exit_fn
};

struct stage stage2_3 = {
    .magic   = MAGIC_STAGE,
    .name    = "stage2_3",
    .stage_fn = stage2_3_fn,
    .init_fn = stage_init_fn,
    .exit_fn = stage_exit_fn
};

struct stage stage2_4 = {
    .magic   = MAGIC_STAGE,
    .name    = "stage2_4",
    .stage_fn = stage2_4_fn,
    .init_fn = stage_init_fn,
    .exit_fn = stage_exit_fn
};

static void timer_fn(void)
{
    uint32_t cur, diff;

    cur = count;
    diff = cur - last_count;
    last_count = cur;
   
    ssd_os_print_lock(); 
    ssd_os_print_sis("\n op/sec      : ", diff, "\n");
    ssd_os_print_sis(" stages/sec  : ", diff * 20, "\n");
    ssd_os_print_sis(" in the rings: ", amount, "\n");
    ssd_os_print_sis(" total       : ", count, "\n");
    ssd_os_print_unlock();
}

int conn1_init(void)
{
    pipe1 = NULL;
    pipe2 = NULL;
    amount = 0;
    count = 0;
    last_count = 0;
    submitted = 0;
    numbers_index = 0;

    ssd_os_timer_interrupt_on(TICKS_SEC, (void *)timer_fn);

    return 0;
}

int conn2_init(void)
{
    void *mem = ssd_os_mem_get(0);

    conn2_lring = ssd_os_lring_create("CONN2_LRING", 128, mem, 0x0);
    
    return 0;
}

int conn1_exit(void)
{
    return 0;
}

int conn2_exit(void)
{
    return 0;
}

struct pipeline *conn1_fn(struct lring_entry *entry)
{
    /* Store the id in the entry context so we can retrieve it later */
    uint16_t *id_ptr = (uint16_t *)&entry->ctx;
    
    /* Allow a finite amount of entries in the rings */
    if (amount < 128 && numbers_index < MAX_NUMBERS) {
        /* Create a new numbers entry in our list */
        struct numbers *n = &numbers_list[numbers_index++];
        n->value = 1;
        n->add = 1;
        n->id = submitted;
        
        /* Store the ID in the lring entry's context */
        *id_ptr = n->id;
        
        amount++;
        submitted++;
        
        /* Pass the pointer to the numbers structure as context */
        entry->ctx = (void *)n;
    }

    if (!pipe1)
        pipe1 = (struct pipeline *)ssd_os_get_connection("cpath_conn1", "cpath_pipe1");

    return pipe1;
}

struct pipeline *conn2_fn(struct lring_entry *entry)
{
    ssd_os_lring_dequeue(conn2_lring, entry);
    
    /* Retrieve the numbers structure based on the ID in the context */
    uint16_t *id_ptr = (uint16_t *)&entry->ctx;
    struct numbers *n = get_number(*id_ptr);
    if (n) {
        entry->ctx = (void *)n;
    }
    
    if (!pipe2)
        pipe2 = (struct pipeline *)ssd_os_get_connection("cpath_conn2", "cpath_pipe2");

    return pipe2;
}

int conn1_ring_fn(struct lring_entry *entry)
{
    /* Retrieve the numbers structure pointer from context_ptr */
    struct numbers *n = (struct numbers *)entry->ctx;
    
    if (n) {
        count++;
        amount--;

        if (n->value != 21) {
            ssd_os_print_lock();
            ssd_os_print_sis("conn1_ring: Number is wrong: ", n->value, "\n");
            ssd_os_print_sis("   ID: ", n->id, "\n");
            ssd_os_print_unlock();
        }
    }

    return 0;
}

int conn2_ring_fn(struct lring_entry *entry)
{
    return ssd_os_lring_enqueue(conn2_lring, entry);
}

struct connector conn1 = {
    .magic   = MAGIC_CONNECTOR,
    .name    = "cpath_conn1",
    .nosched = 0,
    .init_fn = conn1_init,
    .exit_fn = conn1_exit,
    .ring_fn = conn1_ring_fn,
    .conn_fn = conn1_fn
};

struct connector conn2 = {
    .magic   = MAGIC_CONNECTOR,
    .name    = "cpath_conn2",
    .nosched = 0,
    .init_fn = conn2_init,
    .exit_fn = conn2_exit,
    .ring_fn = conn2_ring_fn,
    .conn_fn = conn2_fn
};