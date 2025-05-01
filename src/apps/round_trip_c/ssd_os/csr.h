#define MTIME_REG	0x200BFF8
#define MTIMECMP_REG	0x2004000

static inline int csr_mtval_r (void)
{
    int mtval;
    asm volatile ("csrr %0, mtval" : "=r" (mtval));
    return mtval;
}

static inline int csr_mstatus_r (void)
{
    int mstatus;
    asm volatile ("csrr %0, mstatus" : "=r" (mstatus));
    return mstatus;
}

static inline void csr_mstatus_w (int mstatus)
{
    asm volatile ("csrw mstatus, %0" : : "r" (mstatus));
}

static inline int csr_misa_r (void)
{
    int mie;
    asm volatile ("csrr %0, misa" : "=r" (mie));
    return mie;
}

static inline int csr_mepc_r (void)
{
    int mepc;
    asm volatile ("csrr %0, mepc" : "=r" (mepc));
    return mepc;
}

static inline void csr_mepc_w (int mepc)
{
    asm volatile ("csrw mepc, %0" : : "r" (mepc));
}

static inline int csr_mcycle_r (void)
{
    int mcycle;
    asm volatile ("csrr %0, mcycle" : "=r" (mcycle));
    return mcycle;
}

static inline int csr_minstret_r (void)
{
    int minstret;
    asm volatile ("csrr %0, minstret" : "=r" (minstret));
    return minstret;
}

static inline int csr_mcause_r (void)
{
    int mcause;
    asm volatile ("csrr %0, mcause" : "=r" (mcause));
    return mcause;
}

static inline int csr_mtvec_r (void)
{
    int mtvec;
    asm volatile ("csrr %0, mtvec" : "=r" (mtvec));
    return mtvec;
}

static inline void csr_mtvec_w (int mtvec)
{
    asm volatile ("csrw mtvec, %0" : : "r" (mtvec));
}

static inline int csr_mie_r (void)
{
    int mie;
    asm volatile ("csrr %0, mie" : "=r" (mie));
    return mie;
}

static inline void csr_mie_w (int mie)
{
    asm volatile ("csrw mie, %0" : : "r" (mie));
}

static inline int csr_mhartid_r (void)
{
    int mhartid;
    asm volatile ("csrr %0, mhartid" : "=r" (mhartid));
    return mhartid;
}

static inline volatile int mmap_mtime_rl (void) /* Lower 32-bits */
{
    return *((int *) MTIME_REG);
}

static inline volatile int mmap_mtime_rh (void) /* Higher 32-bits */
{
    return *((int *) (MTIME_REG + 0x4));
}

static inline volatile int mmap_mtimecmp_rl (void) /* Lower 32-bits */
{
    return *((volatile int *) (MTIMECMP_REG + csr_mhartid_r() * 0x8));
}

static inline void mmap_mtimecmp_wl (int v) /* Lower 32-bits */
{
    *((int *) (MTIMECMP_REG + csr_mhartid_r() * 0x8)) = v;
}

static inline volatile int mmap_mtimecmp_rh (void) /* Higher 32-bits */
{
    return *((volatile int *) ((MTIMECMP_REG + csr_mhartid_r() * 0x8) + 0x4));
}

static inline void mmap_mtimecmp_wh (int v) /* Higher 32-bits */
{
    *((int *) ((MTIMECMP_REG + csr_mhartid_r() * 0x8) + 0x4)) = v;
}
