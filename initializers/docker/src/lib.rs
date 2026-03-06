use std::io::{BufReader, Read, Write};

use bollard::{
    Docker, query_parameters::{AttachContainerOptions, WaitContainerOptions}, secret::ContainerCreateBody
};
use futures_util::StreamExt;

pub struct DockerInitializer;

impl DockerInitializer {
    pub async fn spawn(cmd: &[String]) -> anyhow::Result<SpawnedContainer> {
        spawn_profiled_command(cmd).await
    }
}

pub struct SpawnedContainer {
    pub cid: String,
    pub pid: i32,
    pub stdout: BufReader<Box<dyn Read + Send>>,
    docker: Docker,
}

impl SpawnedContainer {
    pub async fn wait(&self) -> anyhow::Result<i64> {
        let opts = WaitContainerOptions {
            condition: "not-running".to_string(),
            ..Default::default()
        };

        let mut stream = self.docker.wait_container(&self.cid, Some(opts));

        match stream.next().await {
            Some(Ok(response)) => Ok(response.status_code),
            Some(Err(e)) => Err(e.into()),
            None => anyhow::bail!("wait stream closed without response"),
        }
    }
}


pub async fn spawn_profiled_command(cmd: &[String]) -> anyhow::Result<SpawnedContainer> {
    let docker = Docker::connect_with_socket_defaults()?;

    let image = cmd.last().cloned();

    let create = docker
        .create_container(
            None,
            ContainerCreateBody {
                image,
                env: extract_env(&cmd),
                ..Default::default()
            },
        )
        .await?;

    let cid = create.id;
    docker.start_container(&cid, None).await?;

    let pid = wait_for_pid(&docker, &cid).await?;

    let attach_opts = AttachContainerOptions {
        logs: true,
        stdout: true,
        stream: true,
        ..Default::default()
    };

    let mut attach = docker.attach_container(&cid, Some(attach_opts)).await?;

    let (pipe_reader, mut pipe_writer) = std::io::pipe()?;

    tokio::spawn(async move {
        while let Some(chunk) = attach.output.next().await {
            match chunk {
                Ok(log_output) => {
                    if pipe_writer.write_all(&log_output.into_bytes()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let stdout = BufReader::new(Box::new(pipe_reader) as Box<dyn Read + Send>);

    Ok(SpawnedContainer {
        cid,
        pid: pid as i32,
        stdout,
        docker,
    })
}

async fn wait_for_pid(docker: &Docker, cid: &str) -> anyhow::Result<i64> {
    loop {
        let inspect = docker.inspect_container(cid, None).await?;
        if let Some(pid) = inspect.state.and_then(|s| s.pid) {
            if pid > 0 {
                return Ok(pid);
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    }
}

fn extract_env(cmd: &[String]) -> Option<Vec<String>> {
    let env: Vec<_> = cmd.windows(2)
        .filter(|w| w[0] == "-e")
        .map(|w| w[1].clone())
        .collect();
    if env.is_empty() {
        None
    } else {
        Some(env)
    }
         
}