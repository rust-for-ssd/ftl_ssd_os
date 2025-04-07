#ifndef OX_BLOCK
#define OX_BLOCK

#include <stdint.h>

#undef	MAX
#define MAX(a, b)  (((a) > (b)) ? (a) : (b))

#undef	MIN
#define MIN(a, b)  (((a) < (b)) ? (a) : (b))

#define AND64                  0xffffffffffffffff
#define ZERO_32FLAG            0x00000000
#define SEC64                  (1000000 & AND64)

#define NVM_CH_IN_USE          0x3c

#define MAX_NAME_SIZE           31
#define NVM_FTL_QUEUE_SIZE      2048

/* Timeout 2 sec */
#define NVM_QUEUE_RETRY         10000
#define NVM_QUEUE_RETRY_SLEEP   200

/* Timeout 10 sec */
#define NVM_FTL_QUEUE_TO        10 * 1000000

#define NVM_SYNCIO_TO          10
#define NVM_SYNCIO_FLAG_BUF    0x1
#define NVM_SYNCIO_FLAG_SYNC   0x2
#define NVM_SYNCIO_FLAG_DEC    0x4
#define NVM_SYNCIO_FLAG_MPL    0x8

enum {
    NVM_DMA_TO_HOST        = 0x0,
    NVM_DMA_FROM_HOST      = 0x1,
    NVM_DMA_SYNC_READ      = 0x2,
    NVM_DMA_SYNC_WRITE     = 0x3
};

enum {
    MMGR_READ_PG   = 0x1,
    MMGR_READ_OOB  = 0x2,
    MMGR_WRITE_PG  = 0x3,
    MMGR_BAD_BLK   = 0x5,
    MMGR_ERASE_BLK = 0x7,
    MMGR_READ_SGL  = 0x8,
    MMGR_WRITE_SGL = 0x9,
    MMGR_WRITE_PL_PG = 0x10,
    MMGR_READ_PL_PG = 0x11,
};

enum NVM_ERROR {
    EMAX_NAME_SIZE   = 0x1,
    EMMGR_REGISTER   = 0x2,
    EPCIE_REGISTER   = 0x3,
    EFTL_REGISTER    = 0x4,
    ENVME_REGISTER   = 0x5,
    ECH_CONFIG       = 0x6,
    EMEM             = 0x7,
    ENOMMGR          = 0x8,
    ENOFTL           = 0x9,
    EPARSER_REGISTER = 0xa,
    ENOPARSER        = 0xb,
    ENOTRANSP        = 0xc,
    ETRANSP_REGISTER = 0xd,
};

enum {
    NVM_IO_SUCCESS     = 0x1,
    NVM_IO_FAIL        = 0x2,
    NVM_IO_PROCESS     = 0x3,
    NVM_IO_NEW         = 0x4,
    NVM_IO_TIMEOUT     = 0x5
};

enum RUN_FLAGS {
    RUN_READY      = (1 << 0),
    RUN_NVME_ALLOC = (1 << 1),
    RUN_MMGR       = (1 << 2),
    RUN_FTL        = (1 << 3),
    RUN_CH         = (1 << 4),
    RUN_TRANSPORT  = (1 << 5),
    RUN_NVME       = (1 << 6),
    RUN_OXAPP      = (1 << 7),
    RUN_FABRICS    = (1 << 8),
    RUN_PARSER     = (1 << 9)
};

enum OX_MEM_TYPES {
    OX_MEM_CMD_ARG      = 0,
    OX_MEM_ADMIN        = 1,
    OX_MEM_CORE_INIT    = 2,
    OX_MEM_CORE_EXEC    = 3,
    OX_MEM_OX_MQ        = 4,
    OX_MEM_FTL_LNVM     = 5,
    OX_MEM_MMGR         = 6,
    OX_MEM_TCP_SERVER   = 7,
    OX_MEM_MMGR_VOLT    = 8,
    OX_MEM_MMGR_OCSSD   = 9,
    OX_MEM_FTL          = 10,
    OX_MEM_OXAPP        = 11,
    OX_MEM_APP_TRANS    = 12,
    OX_MEM_APP_CH       = 13,
    OX_MEM_OXBLK_LOG    = 14,
    OX_MEM_OXBLK_LBA    = 15,
    OX_MEM_OXBLK_GPR    = 16,
    OX_MEM_OXBLK_GMAP   = 17,
    OX_MEM_OXBLK_GC     = 18,
    OX_MEM_OXBLK_CPR    = 19,
    OX_MEM_OXBLK_REC    = 20,
    OX_MEM_FABRICS      = 21,
    OX_MEM_NVMEF        = 22,
    OX_MEM_MMGR_FILEBE	= 23,
    OX_MEM_ELEOS_W      = 29,
    OX_MEM_ELEOS_LBA    = 30,
    OX_MEM_APP_HMAP     = 31 /* 31-40 belong to HMAP instances */
};

enum ox_stats_recovery_types {
    /* Times in microseconds */
    OX_STATS_REC_CP_READ_US = 0,
    OX_STATS_REC_BBT_US,
    OX_STATS_REC_BLK_US,
    OX_STATS_REC_CH_MAP_US,
    OX_STATS_REC_GL_MAP_US,
    OX_STATS_REC_REPLAY_US,
    OX_STATS_REC_CP_WRITE_US,
    OX_STATS_REC_START1_US,
    OX_STATS_REC_START2_US,

    /* Log replay information */
    OX_STATS_REC_LOG_PGS,
    OX_STATS_REC_LOG_SZ,
    OX_STATS_REC_DROPPED_LOGS,
    OX_STATS_REC_TR_COMMIT,
    OX_STATS_REC_TR_ABORT /* 14 */
};

enum ox_stats_cp_types {
    OX_STATS_CP_LOAD_ADDR = 0,
    OX_STATS_CP_MAP_EVICT,
    OX_STATS_CP_MAPMD_EVICT,
    OX_STATS_CP_BLK_EVICT,
    OX_STATS_CP_MAP_BIG_SZ,
    OX_STATS_CP_MAP_SMALL_SZ,
    OX_STATS_CP_MAP_TINY_SZ,
    OX_STATS_CP_BLK_SMALL_SZ,
    OX_STATS_CP_BLK_TINY_SZ,
    OX_STATS_CP_SZ           /* 9 */
};

struct nvm_ppa_addr {
    /* Generic structure for all addresses */
    union {
        struct {
            uint64_t sec   : 3;
            uint64_t pl    : 2;
            uint64_t ch    : 12;
            uint64_t lun   : 6;
            uint64_t pg    : 12;
            uint64_t blk   : 15;
            uint64_t rsv   : 14;
        } g;

        uint64_t ppa;
    };
};


struct nvm_memory_region {
    uint64_t     addr;
    uint64_t     paddr;
    uint64_t     size;
    uint8_t      is_valid;
};

typedef void (nvm_callback_fn) (void *arg);

struct nvm_callback {
    nvm_callback_fn *cb_fn;
    void            *opaque;
    uint64_t         ts;
};

struct nvm_io_status {
    uint8_t     status;       /* global status for the cmd */
    uint16_t    nvme_status;  /* status to send to host */
    uint32_t    pg_errors;    /* n of errors */
    uint32_t    total_pgs;
    uint16_t    pgs_p;        /* pages processed within iteration */
    uint16_t    pgs_s;        /* pages success in request */
    uint16_t    ret_t;        /* retried times */
    uint8_t     pg_map[8];    /* pgs to retry */
};

struct nvm_mmgr_io_cmd {
    struct nvm_io_cmd       *nvm_io;
    struct nvm_ppa_addr     ppa;
    struct nvm_channel      *ch;
    struct nvm_callback     callback;
    uint64_t                prp[32]; /* max of 32 sectors */
    uint64_t                md_prp;
    uint8_t                 status;
    uint8_t                 cmdtype;
    uint32_t                pg_index; /* pg index inside nvm_io_cmd */
    uint32_t                pg_sz;
    uint16_t                n_sectors;
    uint32_t                sec_sz;
    uint32_t                md_sz;
    uint16_t                sec_offset; /* first sector in the ppa vector */
    uint8_t                 force_sync_md;
    uint8_t                 force_sync_data[32];
    uint32_t                sync_count;
    //pthread_mutex_t         *sync_mutex;
    //struct timeval          tstart;
    //struct timeval          tend;

    /* MMGR specific */
    //uint8_t                 rsvd[170];    /* Obsolete FPGA */
    uint8_t                 rsvd[128];       /* Volt + ELEOS */
};

struct nvm_io_cmd {
    uint64_t                    cid;
    struct nvm_channel          *channel[64];
    struct nvm_ppa_addr         ppalist[256];
    struct nvm_io_status        status;
    struct nvm_mmgr_io_cmd      mmgr_io[64];
    struct nvm_callback         callback;
    void                        *req;
    void                        *mq_req;
    void                        *opaque;
    uint64_t                    prp[256]; /* maximum 1 MB for block I/O */
    uint64_t                    md_prp[256];
    uint32_t                    sec_sz;
    uint32_t                    md_sz;
    uint32_t                    n_sec;
    uint64_t                    slba;
    uint8_t                     cmdtype;
    //pthread_mutex_t             mutex;
};
struct nvm_mmgr_geometry {
    uint8_t     n_of_ch;
    uint8_t     lun_per_ch;
    uint16_t    blk_per_lun;
    uint16_t    pg_per_blk;
    uint16_t    sec_per_pg;
    uint8_t     n_of_planes;
    uint32_t    pg_size;
    uint32_t    sec_oob_sz;

    /* calculated values */
    uint32_t    sec_per_pl_pg;
    uint32_t    sec_per_blk;
    uint32_t    sec_per_lun;
    uint32_t    sec_per_ch;
    uint32_t    pg_per_lun;
    uint32_t    pg_per_ch;
    uint32_t    blk_per_ch;
    uint64_t    tot_sec;
    uint64_t    tot_pg;
    uint32_t    tot_blk;
    uint32_t    tot_lun;
    uint32_t    sec_size;
    uint32_t    pl_pg_size;
    uint32_t    blk_size;
    uint64_t    lun_size;
    uint64_t    ch_size;
    uint64_t    tot_size;
    uint32_t    pg_oob_sz;
    uint32_t    pl_pg_oob_sz;
    uint32_t    blk_oob_sz;
    uint32_t    lun_oob_sz;
    uint64_t    ch_oob_sz;
    uint64_t    tot_oob_sz;
};

enum mmgr_flags {
    MMGR_FLAG_PL_CMD        = (1 << 0), /* Accept multi plane page commands */
    MMGR_FLAG_MIN_CP_TIME   = (1 << 2)  /* Use the minimum checkpoint interval*/
};
#define NVM_MAGIC           0x3c
#define NVM_TRANS_TO_NVM    0
#define NVM_TRANS_FROM_NVM  1
#define NVM_IO_NORMAL       0
#define NVM_IO_RESERVED     1 /* Used for FTL reserved blocks */

struct nvm_magic {
    uint32_t rsvd;
    uint32_t magic;
} __attribute__((packed));

struct nvm_io_data {
    struct nvm_channel *ch;
    uint8_t             n_pl;
    uint32_t            pg_sz;
    uint8_t            *buf;
    uint8_t           **pl_vec;  /* Array of plane data (first sector) */
    uint8_t           **oob_vec; /* Array of OOB area (per sector) */
    uint8_t          ***sec_vec; /* Array of sectors + OOB [plane][sector] */
    uint32_t            meta_sz;
    uint32_t            buf_sz;
    uint8_t            *mod_oob; /* OOB as SGL can be buffered here */
};

struct nvm_mmgr;
typedef int     (nvm_mmgr_read_pg)(struct nvm_mmgr_io_cmd *);
typedef int     (nvm_mmgr_write_pg)(struct nvm_mmgr_io_cmd *);
typedef int     (nvm_mmgr_erase_blk)(struct nvm_mmgr_io_cmd *);
typedef int     (nvm_mmgr_get_ch_info)(struct nvm_channel *, uint16_t);
typedef int     (nvm_mmgr_set_ch_info)(struct nvm_channel *, uint16_t);
typedef void    (nvm_mmgr_exit)(struct nvm_mmgr *);

struct nvm_mmgr_ops {
    nvm_mmgr_read_pg       *read_pg;
    nvm_mmgr_write_pg      *write_pg;
    nvm_mmgr_erase_blk     *erase_blk;
    nvm_mmgr_exit          *exit;
    nvm_mmgr_get_ch_info   *get_ch_info;
    nvm_mmgr_set_ch_info   *set_ch_info;
};

struct nvm_channel;
struct nvm_mmgr {
    const char                  *name;
    struct nvm_mmgr_ops         *ops;
    struct nvm_mmgr_geometry    *geometry;
    struct nvm_channel          *ch_info;
    uint8_t                     flags;
};

struct nvm_channel {
    uint16_t                    ch_id;
    uint16_t                    ch_mmgr_id;
    uint64_t                    ns_pgs;
    uint64_t                    slba;
    uint64_t                    elba;
    uint64_t                    tot_bytes;
    uint16_t                    mmgr_rsv; /* number of blks reserved by mmgr */
    uint16_t                    ftl_rsv;  /* number of blks reserved by ftl */
    struct nvm_mmgr             *mmgr;
    struct nvm_ftl              *ftl;
    struct nvm_mmgr_geometry    *geometry;
    struct nvm_ppa_addr         mmgr_rsv_list[16]; /* list of mmgr rsvd blks */
    struct nvm_ppa_addr         ftl_rsv_list[16];  /* list of ftl rsvd blks */
    //LIST_ENTRY(nvm_channel)     entry;
    union {
        struct {
            uint64_t   ns_id         :16;
            uint64_t   ns_part       :32;
            uint64_t   ftl_id        :8;
            uint64_t   in_use        :8;
        } i;
        uint64_t       nvm_info;
    };
};

#define NVM_CMD_ADMIN   0x0
#define NVM_CMD_IO      0x1

//typedef int (ox_nvm_parser_fn)(NvmeRequest *req, NvmeCmd *cmd);

struct nvm_parser_cmd {
    const char           name[MAX_NAME_SIZE];
    uint8_t              opcode;
    uint8_t              queue_type;
    //ox_nvm_parser_fn    *opcode_fn;
};

/* ssd_os64: Change dma->prp to uint64_t */
struct nvm_dma {
    void    *ptr;
    uint32_t prp;
    uint64_t size;
    uint8_t  direction;
};

/* OX-Block Extension Functions */
int ox_dma (struct nvm_dma *dma);

#endif /* OX_BLOCK */
