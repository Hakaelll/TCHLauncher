//! Session-scoped process counters with bounded retention across server restarts.
use std::collections::HashMap;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

const MAX_SERVERS: usize = 32;
struct Entry { session: String, pid: u32, system: System, sampled: bool, touched: u64 }
pub struct Collector { entries: HashMap<String, Entry>, sequence: u64, cores: f32 }
impl Default for Collector {
    fn default() -> Self {
        Self { entries: HashMap::new(), sequence: 0,
            cores: std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1) as f32 }
    }
}
pub struct Reading {
    pub memory: Option<u64>, pub cpu: Option<f32>,
    pub disk_read: Option<u64>, pub disk_written: Option<u64>,
}
impl Collector {
    fn entry(&mut self, id: &str, session: &str, pid: u32) -> &mut Entry {
        if self.entries.get(id).is_some_and(|e| e.session != session || e.pid != pid) {
            self.entries.remove(id);
        }
        if !self.entries.contains_key(id) && self.entries.len() >= MAX_SERVERS {
            if let Some(oldest) = self.entries.iter().min_by_key(|(_,e)| e.touched).map(|(id,_)| id.clone()) {
                self.entries.remove(&oldest);
            }
        }
        self.sequence = self.sequence.saturating_add(1);
        let touched = self.sequence;
        let entry = self.entries.entry(id.into()).or_insert_with(|| Entry {
            session: session.into(), pid, system: System::new(), sampled: false, touched,
        });
        entry.touched = touched;
        entry
    }
    pub fn forget(&mut self, id: &str) { self.entries.remove(id); }
    pub fn sample(&mut self, id: &str, session: &str, pid: u32) -> Reading {
        let cores = self.cores;
        let entry = self.entry(id,session,pid);
        let pid = Pid::from_u32(pid);
        entry.system.refresh_processes_specifics(ProcessesToUpdate::Some(&[pid]),true,
            ProcessRefreshKind::nothing().with_cpu().with_memory().with_disk_usage());
        let process = entry.system.process(pid);
        let disk = process.map(|p| p.disk_usage());
        let reading = Reading { memory: process.map(|p| p.memory()),
            cpu: if entry.sampled { process.map(|p| (p.cpu_usage()/cores).clamp(0.0,100.0)) } else { None },
            disk_read: disk.map(|d| d.total_read_bytes), disk_written: disk.map(|d| d.total_written_bytes) };
        entry.sampled = process.is_some();
        reading
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counters_reset_on_restart_even_when_windows_reuses_the_pid() {
        let mut collector = Collector::default();
        let pid = std::process::id();
        let first = collector.sample("one","launch-a",pid);
        assert!(first.memory.is_some()); assert!(first.cpu.is_none());
        assert!(collector.sample("one","launch-a",pid).cpu.is_some());
        assert!(collector.sample("one","launch-b",pid).cpu.is_none());
        assert_eq!(collector.entries.len(),1);
        assert_eq!(collector.entries["one"].system.processes().len(),1);
        collector.forget("one"); assert!(collector.entries.is_empty());
        assert!(collector.sample("one","launch-b",pid).cpu.is_none());
    }
    #[test]
    fn eviction_is_bounded_and_keeps_recent_servers_without_cross_server_resets() {
        let mut collector = Collector::default();
        for index in 0..MAX_SERVERS { collector.entry(&index.to_string(),"a",42).sampled = true; }
        collector.entry("0","a",42);
        collector.entry("new","a",43);
        assert_eq!(collector.entries.len(),MAX_SERVERS);
        assert!(!collector.entries.contains_key("1"));
        assert!(collector.entries["0"].sampled);
        assert!(!collector.entry("0","a",44).sampled);
        for index in 0..1000 { collector.entry("restarts",&index.to_string(),42); }
        assert_eq!(collector.entries.len(),MAX_SERVERS);
        assert!(collector.entries["new"].system.processes().is_empty());
    }
}
