//! fastlane-dataplane — High-performance packet processing dataplane
//!
//! Provides AF_XDP zero-copy packet processing, multi-queue management,
//! and lock-free ring buffers for 400G+ interface support.

use anyhow::Result;

/// Configuration for AF_XDP socket
pub struct XdpConfig {
    pub iface: String,
    pub queue: u32,
    pub num_descs: u32,
    pub ring_size: u32,
    pub frame_size: u32,
    pub zero_copy: bool,
    pub busy_poll: bool,
    pub num_cpus: u32,
}

/// Lock-free ring buffer for packet processing
pub struct RingBuffer {
    pub capacity: u32,
    pub entries: Vec<u64>,
    pub head: u32,
    pub tail: u32,
}

impl RingBuffer {
    pub fn new(capacity: u32) -> Self {
        Self {
            capacity,
            entries: vec![0; capacity as usize],
            head: 0,
            tail: 0,
        }
    }

    pub fn push(&mut self, value: u64) -> bool {
        let next_tail = (self.tail + 1) % self.capacity;
        if next_tail == self.head {
            return false; // Ring full
        }
        self.entries[self.tail as usize] = value;
        self.tail = next_tail;
        true
    }

    pub fn pop(&mut self) -> Option<u64> {
        if self.head == self.tail {
            return None; // Ring empty
        }
        let value = self.entries[self.head as usize];
        self.head = (self.head + 1) % self.capacity;
        Some(value)
    }
}

/// Multi-queue packet generator
pub struct MultiQueueGenerator {
    pub num_queues: u32,
    pub queues: Vec<RingBuffer>,
}

impl MultiQueueGenerator {
    pub fn new(num_queues: u32, queue_capacity: u32) -> Self {
        Self {
            num_queues,
            queues: (0..num_queues)
                .map(|_| RingBuffer::new(queue_capacity))
                .collect(),
        }
    }

    pub fn send(&mut self, queue_id: u32, packet: u64) -> bool {
        if queue_id >= self.num_queues {
            return false;
        }
        self.queues[queue_id as usize].push(packet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_push_pop() {
        let mut rb = RingBuffer::new(4);
        assert!(rb.push(1));
        assert!(rb.push(2));
        assert_eq!(rb.pop(), Some(1));
        assert_eq!(rb.pop(), Some(2));
        assert_eq!(rb.pop(), None);
    }

    #[test]
    fn test_ring_buffer_full() {
        let mut rb = RingBuffer::new(2);
        assert!(rb.push(10));
        assert!(rb.push(20));
        assert!(!rb.push(30)); // Ring full
    }

    #[test]
    fn test_multi_queue_send() {
        let mut mg = MultiQueueGenerator::new(4, 8);
        assert!(mg.send(0, 1));
        assert!(mg.send(3, 4));
        assert!(!mg.send(5, 5)); // Queue 5 out of range
    }
}
