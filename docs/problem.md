# Problem Definition: The Limits of User-Space Monitoring

Existing tools rely on "symptoms" (high latency, error rates). We focus on "diseases" (kernel degradation). Here are the three specific outage scenarios this platform addresses:

## Scenario 1: The "Silent" Memory Fragmentation
* **The Symptom:** A database node suddenly locks up. RAM usage was only at 70%.
* **The Reality:** High-order memory allocations failed not because memory was full, but because it was fragmented.
* **Why Tools Miss It:** Standard `free -m` reports total free memory, ignoring the lack of contiguous pages (high-order blocks).
* **Our Signal:** `mm_page_alloc_extfrag` events and compaction failures in `/proc/vmstat`.

## Scenario 2: The Uninterruptible Sleep (D-State) Death Spiral
* **The Symptom:** Load average spikes to 100+, but CPU usage is near 0%. The node becomes unresponsive to SSH.
* **The Reality:** An NFS mount or block device has stalled. Threads are entering `TASK_UNINTERRUPTIBLE` (D-state) waiting for I/O that will never come.
* **Why Tools Miss It:** CPU usage monitors see "idle" because these threads aren't using cycles; they are waiting in the kernel scheduler.
* **Our Signal:** Tracking the rate of change of `nr_uninterruptible` via scheduler tracepoints.

## Scenario 3: The IRQ Storm
* **The Symptom:** Network throughput drops to zero, packets are dropped, but the application logs show no errors.
* **The Reality:** A misconfigured NIC or driver bug is firing hardware interrupts faster than the CPU can handle (Interrupt Storm), starving the user-space application.
* **Why Tools Miss It:** Application tracing shows "slow application," misattributing the fault to code rather than hardware saturation.
* **Our Signal:** Monitoring `/proc/interrupts` deltas and `softirq` time per core.