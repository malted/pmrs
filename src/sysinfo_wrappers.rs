use serde::Serialize;
use sysinfo::{SystemExt, PidExt};
use std::collections::HashMap;
use sysinfo::{ProcessExt, CpuExt, DiskExt, NetworksExt, UserExt};
use parking_lot::RwLock;

#[derive(Serialize, Clone)]
pub struct System {
	process_list: HashMap<u32, Process>,
    mem_total: u64,
    mem_free: u64,
    mem_used: u64,
    mem_available: u64,
    swap_total: u64,
    swap_free: u64,
	swap_used: u64,
    global_cpu: Cpu,
    cpus: Vec<Cpu>,
	physical_core_count: Option<usize>,
    disks: Vec<Disk>,
    interfaces: Vec<String>,
    users: Vec<User>,
    boot_time: u64,
	name: Option<String>,
	long_os_version: Option<String>,
	host_name: Option<String>,
	kernel_version: Option<String>,
	os_version: Option<String>,
	distribution_id: String
}
impl System {
	pub fn init(info: &RwLock<sysinfo::System>) -> Self {
		let value = info.read();

		Self {
			process_list: value.processes().iter().map(|(pid, proc)| (pid.as_u32(), proc.into())).collect(),
			mem_total: value.total_memory(),
			mem_free: value.free_memory(),
			mem_used: value.used_memory(),
			mem_available: value.available_memory(),
			swap_total: value.total_swap(),
			swap_free: value.free_swap(),
			swap_used: value.used_swap(),
			global_cpu: value.global_cpu_info().into(),
			cpus: value.cpus().iter().map(|cpu| cpu.into()).collect(),
			physical_core_count: value.physical_core_count(),
			disks: value.disks().iter().map(|disk| disk.into()).collect(),
			interfaces: value.networks().iter().map(|(name, _)| name.to_owned()).collect(),
			users: value.users().iter().map(|user| user.into()).collect(),
			boot_time: value.boot_time(),
			name: value.name(),
			long_os_version: value.long_os_version(),
			host_name: value.host_name(),
			kernel_version: value.kernel_version(),
			os_version: value.os_version(),
			distribution_id: value.distribution_id()
		}
	}
}


#[derive(Serialize, Clone)]
pub struct Process {
    pub name: String,
    pub cmd: Vec<String>,
    pub exe: String,
    pub parent: Option<i32>,
    pub environ: Vec<String>,
    pub cwd: String,
    pub root: String,
    pub memory: u64,
    pub virtual_memory: u64,
    pub start_time: u64,
    pub run_time: u64,
    pub cpu_usage: f32,
    pub disk_usage: DiskUsage,
}
impl From<&sysinfo::Process> for Process {
	fn from(value: &sysinfo::Process) -> Self {
		Self {
			name: value.name().to_owned(),
			cmd: value.cmd().to_owned(),
			exe: value.exe().to_string_lossy().to_string(),
			parent: value.parent().map(|pid| pid.as_u32() as i32),
			environ: value.environ().to_owned(),
			cwd: value.cwd().to_string_lossy().to_string(),
			root: value.root().to_string_lossy().to_string(),
			memory: value.memory(),
			virtual_memory: value.virtual_memory(),
			start_time: value.start_time(),
			run_time: value.run_time(),
			cpu_usage: value.cpu_usage(),
			disk_usage: value.disk_usage().into(),
		}
	}
}

#[derive(Serialize, Clone)]
pub struct DiskUsage {
    pub total_written_bytes: u64,
    pub written_bytes: u64,
    pub total_read_bytes: u64,
    pub read_bytes: u64,
}
impl From<sysinfo::DiskUsage> for DiskUsage {
	fn from(value: sysinfo::DiskUsage) -> Self {
		Self {
			total_written_bytes: value.total_written_bytes,
			written_bytes: value.written_bytes,
			total_read_bytes: value.total_read_bytes,
			read_bytes: value.read_bytes,
		}
	}
}

#[derive(Serialize, Clone)]
pub struct Cpu {
	pub frequency: u64,
	pub cpu_usage: f32,
	pub name: String,
	pub vendor_id: String,
	pub brand: String,
}
impl From<&sysinfo::Cpu> for Cpu {
	fn from(value: &sysinfo::Cpu) -> Self {
		Self {
			frequency: value.frequency(),
			cpu_usage: value.cpu_usage(),
			name: value.name().to_owned(),
			vendor_id: value.vendor_id().to_owned(),
			brand: value.brand().to_owned(),
		}
	}
}

#[derive(Serialize, Clone)]
pub struct Disk {
    pub r#type: DiskKind,
    pub name: String,
    pub file_system: Option<String>,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub is_removable: bool,
}
impl From<&sysinfo::Disk> for Disk {
	fn from(value: &sysinfo::Disk) -> Self {
		Self {
			r#type: match value.kind() {
				sysinfo::DiskKind::HDD => DiskKind::HDD,
				sysinfo::DiskKind::SSD => DiskKind::SSD,
				sysinfo::DiskKind::Unknown(i) => DiskKind::Unknown(i),
			},
			name: value.name().to_string_lossy().to_string(),
			file_system: std::str::from_utf8(value.file_system()).ok().map(|s| s.to_owned()),
			mount_point: value.mount_point().to_string_lossy().to_string(),
			total_space: value.total_space(),
			available_space: value.available_space(),
			is_removable: value.is_removable(),
		}
	}
}

#[derive(Serialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum DiskKind {
    HDD,
    SSD,
    Unknown(isize),
}

#[derive(Serialize, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct User {
    pub name: String,
    pub groups: Vec<String>,
}
impl From<&sysinfo::User> for User {
	fn from(value: &sysinfo::User) -> Self {
		Self {
			name: value.name().to_owned(),
			groups: value.groups().to_owned(),
		}
	}
}