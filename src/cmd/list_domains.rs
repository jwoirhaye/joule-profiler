use anyhow::Result;
use serde::Serialize;

use crate::cli::ListArgs;
use crate::rapl::{RaplDomain, discover_sockets, parse_sockets};

#[derive(Debug, Serialize)]
pub struct DomainInfo {
    socket: u32,
    name: String,
    raw_name: String,
    path: String,
}

pub fn run_list_domains(args: ListArgs, domains: &[RaplDomain]) -> Result<()> {
    let sockets = if let Some(spec) = args.sockets.as_deref() {
        parse_sockets(spec, domains)?
    } else {
        discover_sockets(domains)
    };

    let mut infos: Vec<DomainInfo> = domains
        .iter()
        .filter(|d| sockets.contains(&d.socket))
        .map(|d| DomainInfo {
            socket: d.socket,
            name: d.name.to_uppercase(),
            raw_name: d.name.clone(),
            path: d.path.to_string_lossy().into(),
        })
        .collect();

    infos.sort_by(|a, b| a.socket.cmp(&b.socket).then_with(|| a.name.cmp(&b.name)));

    if args.json {
        print_domains_json(&infos)?;
    } else if args.csv {
        print_domains_csv(&infos)?;
    } else {
        print_domains_table(&infos)?;
    }

    Ok(())
}

fn print_domains_table(infos: &[DomainInfo]) -> Result<()> {
    if infos.is_empty() {
        println!("No RAPL domains found for the selected sockets.");
        return Ok(());
    }

    println!();
    println!("Available RAPL domains:");

    let mut current_socket: Option<u32> = None;

    for info in infos {
        if current_socket != Some(info.socket) {
            current_socket = Some(info.socket);
            println!("\nSocket {}:\n", info.socket);
            println!("  {:<16} {:<20} PATH", "NAME", "RAW_NAME",);
            println!("  {:<16} {:<20} ----", "----", "--------");
        }

        println!("  {:<16} {:<20} {}", info.name, info.raw_name, info.path);
    }

    println!();
    Ok(())
}

fn print_domains_json(infos: &[DomainInfo]) -> Result<()> {
    let value = serde_json::to_value(infos)?;
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

fn print_domains_csv(infos: &[DomainInfo]) -> Result<()> {
    use std::io::{self, Write};

    let mut out = io::stdout();

    writeln!(out, "socket;name;raw_name;path")?;

    for info in infos {
        writeln!(
            out,
            "{};{};{};{}",
            info.socket, info.name, info.raw_name, info.path
        )?;
    }

    Ok(())
}
