use anyhow::Result;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::info;

pub struct SchedulerService {
    scheduler: JobScheduler,
}

impl SchedulerService {
    pub async fn new() -> Result<Self> {
        let scheduler = JobScheduler::new().await?;
        Ok(Self { scheduler })
    }

    pub async fn add_job<F>(&mut self, cron: &str, name: &str, job_fn: F) -> Result<()>
    where
        F: Fn() -> futures::future::BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        let job = Job::new_async(cron, move |_, _| {
            let job_fn = job_fn();
            Box::pin(async move {
                job_fn.await;
            })
        })?;

        self.scheduler.add(job).await?;
        info!("Added job: {}", name);
        Ok(())
    }

    pub async fn start(&mut self) -> Result<()> {
        self.scheduler.start().await?;
        info!("Scheduler started");
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        self.scheduler.shutdown().await?;
        info!("Scheduler shutdown");
        Ok(())
    }
}
