#[derive(Copy, Clone)]
pub struct AllocatorStatistics {
    pub allocated_size: usize,
    pub allocation_count: usize,
    pub total_memory: usize,
}
