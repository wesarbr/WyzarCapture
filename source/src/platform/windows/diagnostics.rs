use windows::Win32::{
    Foundation::FILETIME,
    System::{ProcessStatus::*, Threading::*},
};
#[derive(Clone, Copy, Default, Debug)]
pub struct ProcessSample {
    pub cpu_ticks: u64,
    pub working_set: u64,
    pub private_bytes: u64,
}
pub fn process_sample() -> Option<ProcessSample> {
    unsafe {
        let process = GetCurrentProcess();
        let (mut creation, mut exit, mut kernel, mut user) = (
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
        );
        GetProcessTimes(process, &mut creation, &mut exit, &mut kernel, &mut user).ok()?;
        let mut memory = PROCESS_MEMORY_COUNTERS_EX {
            cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
            ..Default::default()
        };
        GetProcessMemoryInfo(
            process,
            (&mut memory as *mut PROCESS_MEMORY_COUNTERS_EX).cast(),
            memory.cb,
        )
        .ok()?;
        let ticks = |t: FILETIME| ((t.dwHighDateTime as u64) << 32) | t.dwLowDateTime as u64;
        Some(ProcessSample {
            cpu_ticks: ticks(kernel) + ticks(user),
            working_set: memory.WorkingSetSize as u64,
            private_bytes: memory.PrivateUsage as u64,
        })
    }
}
