use pgrx::prelude::*;

pgrx::pg_module_magic!();

#[pg_guard]
#[no_mangle]
pub extern "C" fn bgworker_sleep() {
    use pgrx::bgworkers::*;
    BackgroundWorker::attach_signal_handlers(SignalWakeFlags::SIGHUP | SignalWakeFlags::SIGTERM);
    while BackgroundWorker::wait_latch(None) {}
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;
    use pgrx::bgworkers::*;

    #[pg_test]
    fn test_dynamic_worker_allocation_failure() {
        let max_proc = Spi::get_one::<i32>("SELECT current_setting('max_worker_processes')::int")
            .expect("failed to get max_worker_processes")
            .expect("got null for max_worker_processes");
        let available_proc = max_proc - 1; // One worker process for logical replication launcher

        let results = (0..available_proc+1).map(|_| {
            BackgroundWorkerBuilder::new("dynamic_bgworker")
                .set_library("pgrx_tests")
                .set_function("bgworker_sleep")
                .enable_shmem_access(None)
                .set_notify_pid(unsafe { pg_sys::MyProcPid })
                .load_dynamic()
        }).collect::<Vec<_>>();
        for worker in results {
            let handle = worker.terminate();
            handle.wait_for_shutdown().expect("aborted shutdown");
        }
    }

}



/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
