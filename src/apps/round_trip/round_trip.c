#include <ssd_os.h>
#include <stdint.h>
#include <lring.h>
#include <stdbool.h>

static struct lring *conn2_lring;
static uint32_t amount, count, submitted, last_count;
static struct pipeline *pipe1;
static struct pipeline *pipe2;

#define POOL_SIZE 10000
#define RING_SIZE 128

struct numbers {
    uint8_t value;
    uint8_t add;
    uint16_t id;
};

static struct numbers MESSAGE_POOL[POOL_SIZE];
static bool MSG_USAGE_BITMAP[POOL_SIZE];

static void init_message_pool(void) {
    for (int i = 0; i < POOL_SIZE; i++) {
        MESSAGE_POOL[i].value = 0;
        MESSAGE_POOL[i].add = 0;
        MESSAGE_POOL[i].id = 0;
        MSG_USAGE_BITMAP[i] = false;
    }
}

static int get_free_message_index(void) {
    for (int i = 0; i < POOL_SIZE; i++) {
        if (!MSG_USAGE_BITMAP[i]) {
            MSG_USAGE_BITMAP[i] = true;
            return i;
        }
    }
    return -1; // No free messages
}

static struct numbers* get_message_ptr(int index) {
    if (index >= 0 && index < POOL_SIZE) {
        return &MESSAGE_POOL[index];
    }
    return NULL;
}

static void release_message(int index) {
    if (index >= 0 && index < POOL_SIZE) {
        MSG_USAGE_BITMAP[index] = false;
        MESSAGE_POOL[index].value = 0;
        MESSAGE_POOL[index].add = 0;
        MESSAGE_POOL[index].id = 0;
    }
}

static int get_index_from_ptr(struct numbers* ptr) {
    if (ptr == NULL) {
        return -1;
    }
    
    int offset = ptr - &MESSAGE_POOL[0];
    if (offset >= 0 && offset < POOL_SIZE) {
        return offset;
    }
    return -1;
}

void *add(void *ctx) {
    if (ctx != NULL) {
        struct numbers *n = (struct numbers *)ctx;
        n->value = n->value + n->add;
    }
    return ctx;
}

void *stage1_1_fn (void *ctx) { ctx = (void *) add (ctx); return ctx; }
void *stage1_2_fn (void *ctx) { ctx = (void *) add (ctx); return ctx; }
void *stage1_3_fn (void *ctx) { ctx = (void *) add (ctx); return ctx; }
void *stage2_1_fn (void *ctx) { ctx = (void *) add (ctx); return ctx; }
void *stage2_2_fn (void *ctx) { ctx = (void *) add (ctx); return ctx; }
void *stage2_3_fn (void *ctx) { ctx = (void *) add (ctx); return ctx; }

int stage_init_fn (void) { return 0; }
int stage_exit_fn (void) { return 0; }

struct stage stage1_1 = {
    .magic	= MAGIC_STAGE,
    .name	= "stage1_1",
    .stage_fn	= stage1_1_fn,
    .init_fn	= stage_init_fn,
    .exit_fn	= stage_exit_fn
};

struct stage stage1_2 = {
    .magic    = MAGIC_STAGE,
    .name     = "stage1_2",
    .stage_fn = stage1_2_fn,
    .init_fn  = stage_init_fn,
    .exit_fn  = stage_exit_fn
};

struct stage stage1_3 = {
    .magic    = MAGIC_STAGE,
    .name     = "stage1_3",
    .stage_fn = stage1_3_fn,
    .init_fn  = stage_init_fn,
    .exit_fn  = stage_exit_fn
};

struct stage stage2_1 = {
    .magic    = MAGIC_STAGE,
    .name     = "stage2_1",
    .stage_fn = stage2_1_fn,
    .init_fn  = stage_init_fn,
    .exit_fn  = stage_exit_fn
};

struct stage stage2_2 = {
    .magic    = MAGIC_STAGE,
    .name     = "stage2_2",
    .stage_fn = stage2_2_fn,
    .init_fn  = stage_init_fn,
    .exit_fn  = stage_exit_fn
};

struct stage stage2_3 = {
    .magic    = MAGIC_STAGE,
    .name     = "stage2_3",
    .stage_fn = stage2_3_fn,
    .init_fn  = stage_init_fn,
    .exit_fn  = stage_exit_fn
};

static void timer_fn (void)
{
    uint32_t cur, diff;

    cur = count;
    diff = cur - last_count;
    last_count = cur;
   
    ssd_os_print_lock (); 
    ssd_os_print_sis ("\n op/sec      : ", diff, "\n");
    ssd_os_print_sis (" stages/sec  : ", diff * 6, "\n"); // 6 stages total
    ssd_os_print_sis (" in the rings: ", amount, "\n");
    ssd_os_print_sis (" total       : ", count, "\n");
    ssd_os_print_sis (" submitted   : ", submitted, "\n");
    ssd_os_print_sis ("", diff * 6, "\n"); // for benchmark
    ssd_os_print_unlock ();
}

int conn1_init (void)
{
    pipe1 = NULL;
    pipe2 = NULL;
    amount = 0;
    count = 0;
    last_count = 0;
    submitted = 0;
    
    init_message_pool();

    ssd_os_timer_interrupt_on (TICKS_SEC, (void *) timer_fn);

    return 0;
}

int conn2_init (void)
{
    void *mem = ssd_os_mem_get (0);

    conn2_lring = ssd_os_lring_create ("CONN2_LRING", RING_SIZE, mem, 0x0);
    
    return 0;
}

int conn1_exit (void)
{
    return 0;
}

int conn2_exit (void)
{
    return 0;
}

struct pipeline *conn1_fn (struct lring_entry *entry)
{
    void* ctx_ptr = entry->ctx;

    if (ctx_ptr == NULL && amount < RING_SIZE) {  // <- Ensure we don't exceed ring capacity
        int idx = get_free_message_index();
        if (idx >= 0) {
            struct numbers *msg_ptr = get_message_ptr(idx);
            
            msg_ptr->value = 1;
            msg_ptr->add = 1;
            msg_ptr->id = submitted;
            submitted++;
            
            entry->ctx = (void*)msg_ptr;
            amount++;
        } else {
            ssd_os_print_lock();
            ssd_os_print_s("No free messages in pool!\n");
            ssd_os_print_unlock();
        }
    }

    if (!pipe1)
        pipe1 = (struct pipeline *) ssd_os_get_connection("cpath_conn1", "cpath_pipe1");

    return pipe1;
}

struct pipeline *conn2_fn (struct lring_entry *entry)
{
    ssd_os_lring_dequeue (conn2_lring, entry);
    
    if (!pipe2)
        pipe2 = (struct pipeline *) ssd_os_get_connection("cpath_conn2", "cpath_pipe2");

    return pipe2;
}

int conn1_ring_fn (struct lring_entry *entry)
{
    struct numbers *n = (struct numbers *)entry->ctx;

    count++;
    amount--;

    if (n->value != 7) {
        ssd_os_print_lock();
        ssd_os_print_sis("conn1_ring: Value is wrong: ", n->value, "\n");
        ssd_os_print_sis("   ID: ", n->id, "\n");
        ssd_os_print_unlock();
    }
    
    // Release the message back to the pool
    int idx = get_index_from_ptr(n);
    if (idx >= 0) {
        release_message(idx);
    }

    return 0;
}

int conn2_ring_fn (struct lring_entry *entry)
{
    return ssd_os_lring_enqueue(conn2_lring, entry);
}

struct connector conn1 = {
    .magic    = MAGIC_CONNECTOR,
    .name     = "cpath_conn1",
    .nosched  = 0,
    .init_fn  = conn1_init,
    .exit_fn  = conn1_exit,
    .ring_fn  = conn1_ring_fn,
    .conn_fn  = conn1_fn
};

struct connector conn2 = {
    .magic    = MAGIC_CONNECTOR,
    .name     = "cpath_conn2",
    .nosched  = 0,
    .init_fn  = conn2_init,
    .exit_fn  = conn2_exit,
    .ring_fn  = conn2_ring_fn,
    .conn_fn  = conn2_fn
};