#ifndef PIPELINE_H
#define PIPELINE_H

void process_ring  (int cpu, void *opaque);
void pipeline_init (void);
void service_boot  (void);

#endif /* PIPELINE_H */
