#![type_length_limit = "2097152"]

use bollard::container::*;
use bollard::errors::Error;
use bollard::exec::*;
use bollard::Docker;

use futures_util::stream::TryStreamExt;
use tokio::runtime::Runtime;

#[macro_use]
pub mod common;
use crate::common::*;

async fn start_exec_test(docker: Docker) -> Result<(), Error> {
    create_daemon(&docker, "integration_test_start_exec_test").await?;

    let message = &docker
        .create_exec(
            "integration_test_start_exec_test",
            CreateExecOptions {
                attach_stdout: Some(true),
                cmd: if cfg!(windows) {
                    Some(vec![
                        "cmd.exe",
                        "/C",
                        "type",
                        "C:\\Windows\\System32\\Inetsrv\\Config\\ApplicationHost.config",
                    ])
                } else {
                    Some(vec!["/bin/cat", "/etc/config/uhttpd"])
                },
                ..Default::default()
            },
        )
        .await?;

    let results = docker
        .start_exec(&message.id, None::<StartExecOptions>)
        .await?;

    assert!(match results {
        StartExecResults::Attached { output, .. } => {
            let log: Vec<_> = output.try_collect().await?;
            assert!(!log.is_empty());
            match &log[0] {
                LogOutput::StdOut { message } => {
                    let (n, expected) = if cfg!(windows) {
                        (0, "<configuration>\r")
                    } else {
                        (1, "config uhttpd main")
                    };

                    let s = String::from_utf8_lossy(message);
                    s.split('\n').nth(n).expect("log exists") == expected
                }
                _ => false,
            }
        }
        _ => false,
    });

    let _ = &docker
        .kill_container(
            "integration_test_start_exec_test",
            None::<KillContainerOptions<String>>,
        )
        .await?;

    let _ = &docker
        .wait_container(
            "integration_test_start_exec_test",
            None::<WaitContainerOptions<String>>,
        )
        .try_collect::<Vec<_>>()
        .await?;

    let _ = &docker
        .remove_container(
            "integration_test_start_exec_test",
            None::<RemoveContainerOptions>,
        )
        .await?;

    Ok(())
}

async fn inspect_exec_test(docker: Docker) -> Result<(), Error> {
    create_daemon(&docker, "integration_test_inspect_exec_test").await?;

    let message = &docker
        .create_exec(
            "integration_test_inspect_exec_test",
            CreateExecOptions {
                attach_stdout: Some(true),
                cmd: if cfg!(windows) {
                    Some(vec![
                        "cmd.exe",
                        "/C",
                        "type",
                        "C:\\Windows\\System32\\Inetsrv\\Config\\ApplicationHost.config",
                    ])
                } else {
                    Some(vec!["/bin/cat", "/etc/config/uhttpd"])
                },
                ..Default::default()
            },
        )
        .await?;

    docker
        .start_exec(
            &message.id,
            Some(StartExecOptions {
                detach: true,
                ..Default::default()
            }),
        )
        .await?;

    let exec_process = &docker.inspect_exec(&message.id).await?;

    assert_eq!(
        if cfg!(windows) { "cmd.exe" } else { "/bin/cat" },
        exec_process
            .process_config
            .as_ref()
            .unwrap()
            .entrypoint
            .as_ref()
            .unwrap()
    );

    let _ = &docker
        .kill_container(
            "integration_test_inspect_exec_test",
            None::<KillContainerOptions<String>>,
        )
        .await?;

    let _ = &docker
        .wait_container(
            "integration_test_inspect_exec_test",
            None::<WaitContainerOptions<String>>,
        )
        .try_collect::<Vec<_>>()
        .await;

    let _ = &docker
        .remove_container(
            "integration_test_inspect_exec_test",
            None::<RemoveContainerOptions>,
        )
        .await?;

    Ok(())
}

async fn start_exec_output_capacity_test(docker: Docker) -> Result<(), Error> {
    create_daemon(&docker, "start_exec_output_capacity_test").await?;

    let text1 = "a".repeat(1024);
    let text2 = "a".repeat(7 * 1024);

    let message = &docker
        .create_exec(
            "start_exec_output_capacity_test",
            CreateExecOptions {
                attach_stdout: Some(true),
                cmd: if cfg!(windows) {
                    Some(vec!["cmd.exe", "/C", "echo", &text1])
                } else {
                    Some(vec!["/bin/echo", &text1])
                },
                ..Default::default()
            },
        )
        .await?;

    let results = docker
        .start_exec(&message.id, None::<StartExecOptions>)
        .await?;

    assert!(match results {
        StartExecResults::Attached { output, .. } => {
            let log: Vec<_> = output.try_collect().await?;
            assert!(!log.is_empty());
            match &log[0] {
                LogOutput::StdOut { message } => {
                    let expected = if cfg!(windows) { text1 + "\r" } else { text1 };

                    let s = String::from_utf8_lossy(message);
                    s.split('\n').next().expect("log exists") == expected
                }
                _ => false,
            }
        }
        _ => false,
    });

    let message = &docker
        .create_exec(
            "start_exec_output_capacity_test",
            CreateExecOptions {
                attach_stdout: Some(true),
                cmd: if cfg!(windows) {
                    Some(vec!["cmd.exe", "/C", "echo", &text2])
                } else {
                    Some(vec!["/bin/echo", &text2])
                },
                ..Default::default()
            },
        )
        .await?;

    let results = docker
        .start_exec(
            &message.id,
            Some(StartExecOptions {
                output_capacity: Some(100 * 1024),
                ..Default::default()
            }),
        )
        .await?;

    assert!(match results {
        StartExecResults::Attached { output, .. } => {
            let log: Vec<_> = output.try_collect().await?;
            assert!(!log.is_empty());
            match &log[0] {
                LogOutput::StdOut { message } => {
                    let expected = if cfg!(windows) { text2 + "\r" } else { text2 };

                    let s = String::from_utf8_lossy(message);
                    s.split('\n').next().expect("log exists") == expected
                }
                _ => false,
            }
        }
        _ => false,
    });

    let _ = &docker
        .kill_container(
            "start_exec_output_capacity_test",
            None::<KillContainerOptions<String>>,
        )
        .await?;

    let _ = &docker
        .wait_container(
            "start_exec_output_capacity_test",
            None::<WaitContainerOptions<String>>,
        )
        .try_collect::<Vec<_>>()
        .await?;

    let _ = &docker
        .remove_container(
            "start_exec_output_capacity_test",
            None::<RemoveContainerOptions>,
        )
        .await?;

    Ok(())
}

#[test]
fn integration_test_start_exec() {
    connect_to_docker_and_run!(start_exec_test);
}

#[test]
fn integration_test_inspect_exec() {
    connect_to_docker_and_run!(inspect_exec_test);
}

#[test]
fn integration_test_start_exec_output_capacity() {
    connect_to_docker_and_run!(start_exec_output_capacity_test);
}
